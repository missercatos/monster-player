use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::api::client::Client;
use crate::api::types::*;
use crate::player::Player;

#[derive(Clone, Copy, PartialEq)]
pub enum PlayMode {
    /// 专辑内列表循环
    AlbumList,
    /// 专辑内随机播放
    AlbumRandom,
    /// 全局列表循环
    GlobalList,
    /// 全局随机播放
    GlobalRandom,
    /// 单曲播放后停止
    Single,
    /// 收藏歌曲列表循环
    LoveList,
    /// 收藏歌曲随机播放
    LoveRandom,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LovedEntry {
    /// 歌曲CID
    pub cid: String,
    /// 歌曲名
    pub name: String,
    /// 艺术家列表
    pub artists: Vec<String>,
}

pub struct Engine {
    pub albums: Vec<Album>, // 全局专辑列表
    pub album_index: usize, // 当前专辑索引
    pub songs: Vec<AlbumSong>, // 当前专辑的歌曲列表
    pub songs_loaded: bool, // 歌曲列表是否已加载
    pub album_name: Option<String>, // 当前专辑名称
    pub album_artist: Option<String>, // 当前专辑艺术家
    pub album_total: usize, // 专辑总数

    pub playing: bool, // 是否正在播放
    pub volume: u8, // 音量 0-100
    pub play_mode: PlayMode, // 当前播放模式
    pub current_song_name: Option<String>, // 当前播放曲目名称
    pub current_song_cid: Option<String>, // 当前播放曲目的 CID
    pub current_song_index: Option<usize>, // 当前播放曲目在列表中的索引
    pub song_info: Option<String>, // 当前歌曲展示信息
    pub album_intro: Option<String>, // 当前播放歌曲所属专辑的简介
    pub buffering: bool, // 是否正在缓冲音频数据
    pub buffering_msg: Option<String>, // 缓冲提示文本

    pub lyrics: Vec<(f64, String)>, // 已解析的 LRC 歌词 (时间秒, 文本)
    pub lyric_index: usize, // 当前匹配的歌词行索引
    pub progress: Option<f64>, // 播放进度 0.0-1.0

    pub loved_cids: HashMap<String, LovedEntry>, // 收藏的歌曲 (CID → LovedEntry)
    pub loved_list: Vec<LovedEntry>, // 收藏歌曲列表 (用于展示/播放)

    wav_data: Option<Vec<u8>>, // 当前播放曲目的原始 WAV 字节缓存（用于跳转/重播）
    stream_buffer: Option<Arc<Mutex<Vec<u8>>>>, // 渐进下载缓冲区（后台写，主线程读）
    stream_done: Option<Arc<Mutex<bool>>>, // 渐进下载是否完成
    stream_started: bool, // 首个 chunk 是否已开始播放
    stream_switched: bool, // 是否已完成从 chunk 到全曲的无缝切换
    loved_path: PathBuf, // loved.json 持久化文件路径

    detail_cache: Arc<Mutex<HashMap<String, AlbumDetail>>>, // 专辑详情缓存（线程安全 HashMap）
    detail_pending: Option<Arc<Mutex<Option<Result<AlbumDetail, String>>>>>, // 待处理的专辑详情异步请求句柄
    albums_pending: Option<Arc<Mutex<Option<Result<Vec<Album>, String>>>>>, // 待处理的专辑列表异步请求句柄
    song_cache: Arc<Mutex<HashMap<String, SongDetail>>>, // 歌曲详情缓存（线程安全 HashMap, CID→SongDetail）
    song_pending: Option<Arc<Mutex<Option<Result<SongDetail, String>>>>>, // 待处理的歌曲详情异步请求句柄
    lyric_cache: HashMap<String, Vec<(f64, String)>>, // 已解析歌词缓存 (URL→LRC行列表)
    player: Option<Player>, // rodio 音频播放器实例
    wav_pending: Option<Arc<Mutex<Option<Result<Vec<u8>, String>>>>>, // 待处理的 WAV 音频下载异步请求句柄
    pending_song: Option<SongDetail>, // 正在等待缓冲的歌曲详情
}

impl Engine {
    /// 初始化引擎：创建配置目录，从磁盘恢复收藏数据
    pub fn new() -> Self {
        let proj_dirs =
            directories::ProjectDirs::from("com", "msplayer", "msplayer")
                .expect("failed to get project dirs");
        let mut config_dir = proj_dirs.config_dir().to_path_buf();
        std::fs::create_dir_all(&config_dir).ok();
        config_dir.push("loved.json");
        let loved_path = config_dir;

        let loved_entries = Self::load_loved_entries(&loved_path);
        let loved_list: Vec<LovedEntry> = loved_entries.values().cloned().collect();

        Self {
            albums: Vec::new(),
            album_index: 0,
            songs: Vec::new(),
            songs_loaded: false,
            album_name: None,
            album_artist: None,
            album_total: 0,
            playing: false,
            volume: 50,
            play_mode: PlayMode::AlbumList,
            current_song_name: None,
            current_song_cid: None,
            current_song_index: None,
            song_info: None,
            album_intro: None,
            buffering: false,
            buffering_msg: None,
            lyrics: Vec::new(),
            lyric_index: 0,
            progress: None,
            wav_data: None,
            stream_buffer: None,
            stream_done: None,
            stream_started: false,
            stream_switched: false,
            loved_cids: loved_entries,
            loved_list,
            loved_path,
            detail_cache: Arc::new(Mutex::new(HashMap::new())),
            detail_pending: None,
            albums_pending: None,
            song_cache: Arc::new(Mutex::new(HashMap::new())),
            song_pending: None,
            lyric_cache: HashMap::new(),
            player: None,
            wav_pending: None,
            pending_song: None,
        }
    }

    /// 每帧更新：轮询异步请求结果 + 歌词进度 + 自动切歌检测
    pub fn update(&mut self) {
        if self.albums.is_empty() && self.albums_pending.is_none() {
            self.fetch_albums();
        }
        self.check_albums();
        self.check_detail();
        self.check_song();
        self.check_wav();
        self.check_stream();
        self.update_lyric_index();
        self.auto_advance();
    }

    /// 播放列表中指定位置的歌曲（根据模式从专辑或收藏列表选取）
    pub fn play_song_at(&mut self, index: usize) {
        let is_love = matches!(self.play_mode, PlayMode::LoveList | PlayMode::LoveRandom);

        if is_love {
            if self.loved_list.is_empty() {
                return;
            }
            self.current_song_index = Some(index);
            let cid = self.loved_list[index].cid.clone();
            self.play_cid(&cid);
        } else {
            if self.songs.is_empty() {
                return;
            }
            self.current_song_index = Some(index);
            let cid = self.songs[index].cid.clone();
            self.play_cid(&cid);
        }
    }

    /// 通过 CID 播放歌曲：缓存命中直接播，否则异步获取详情
    fn play_cid(&mut self, cid: &str) {
        let cached = self.song_cache.lock().unwrap().get(cid).cloned();
        if let Some(song) = cached {
            self.start_playback(&song);
            return;
        }
        // 允许新请求覆盖旧的 pending，避免快速切歌时静默丢弃
        let cid_owned = cid.to_string();
        let pending = Arc::new(Mutex::new(None));
        let p = pending.clone();
        std::thread::spawn(move || {
            let client = Client::new();
            let result = client.song_detail(&cid_owned).map_err(|e| e.to_string());
            *p.lock().unwrap() = Some(result);
        });
        self.song_pending = Some(pending);
    }

    /// 切换 暂停/播放 状态
    pub fn toggle_pause(&mut self) {
        if let Some(ref player) = self.player {
            player.toggle();
            self.playing = !player.is_paused();
        }
    }

    /// 切换到下一张专辑
    pub fn next_album(&mut self) {
        if self.albums.is_empty() {
            return;
        }
        self.album_index = (self.album_index + 1) % self.albums.len();
        self.fetch_album_detail();
    }

    /// 切换到上一张专辑（循环）
    pub fn prev_album(&mut self) {
        if self.albums.is_empty() {
            return;
        }
        self.album_index = self
            .album_index
            .checked_sub(1)
            .unwrap_or(self.albums.len() - 1);
        self.fetch_album_detail();
    }

    /// 循环切换播放模式 (7种)
    pub fn cycle_mode(&mut self) {
        self.play_mode = match self.play_mode {
            PlayMode::AlbumList => PlayMode::AlbumRandom,
            PlayMode::AlbumRandom => PlayMode::GlobalList,
            PlayMode::GlobalList => PlayMode::GlobalRandom,
            PlayMode::GlobalRandom => PlayMode::Single,
            PlayMode::Single => PlayMode::LoveList,
            PlayMode::LoveList => PlayMode::LoveRandom,
            PlayMode::LoveRandom => PlayMode::AlbumList,
        };
    }

    /// 音量 +5%（上限 100），立即应用到 rodio 输出
    pub fn volume_up(&mut self) {
        self.volume = (self.volume + 5).min(100);
        self.apply_volume();
    }

    /// 音量 -5%（下限 0）
    pub fn volume_down(&mut self) {
        self.volume = self.volume.saturating_sub(5);
        self.apply_volume();
    }

    /// 进度前进 5%（重新解码跳转）
    pub fn seek_forward(&mut self) {
        if let (Some(player), Some(data)) = (self.player.as_ref(), self.wav_data.as_ref()) {
            let dur = player.duration().unwrap_or(0.0);
            if dur <= 0.0 {
                return;
            }
            let cur = player.elapsed();
            let target = (cur + dur * 0.05).min(dur - 0.5);
            if let Err(e) = player.play_bytes_at(data.clone(), target) {
                eprintln!("seek: {e}");
            }
        }
    }

    /// 进度后退 5%

    /// 当前播放进度（秒）
    pub fn elapsed_secs(&self) -> f64 {
        self.player.as_ref().map_or(0.0, |p| p.elapsed())
    }

    /// 当前曲目总时长（秒），未解码则返回 None
    pub fn duration_secs(&self) -> Option<f64> {
        self.player.as_ref().and_then(|p| p.duration())
    }

    pub fn seek_backward(&mut self) {
        if let (Some(player), Some(data)) = (self.player.as_ref(), self.wav_data.as_ref()) {
            let cur = player.elapsed();
            let dur = player.duration().unwrap_or(0.0);
            let target = (cur - dur * 0.05).max(0.0);
            if let Err(e) = player.play_bytes_at(data.clone(), target) {
                eprintln!("seek: {e}");
            }
        }
    }

    /// 从头重新播放当前歌曲
    pub fn restart_song(&mut self) {
        if let (Some(player), Some(data)) = (self.player.as_ref(), self.wav_data.as_ref()) {
            if let Err(e) = player.play_bytes_at(data.clone(), 0.0) {
                eprintln!("restart: {e}");
            }
        }
    }

    /// 检查指定 CID 是否已收藏
    pub fn is_loved(&self, cid: &str) -> bool {
        self.loved_cids.contains_key(cid)
    }

    /// 收藏/取消收藏，并写入 loved.json
    pub fn toggle_love(&mut self, cid: &str, name: &str, artists: &[String]) {
        if self.loved_cids.contains_key(cid) {
            self.loved_cids.remove(cid);
        } else {
            self.loved_cids.insert(
                cid.to_string(),
                LovedEntry {
                    cid: cid.to_string(),
                    name: name.to_string(),
                    artists: artists.to_vec(),
                },
            );
        }
        self.save_loved();
        self.loved_list = self.loved_cids.values().cloned().collect();
    }

    /// 从缓存和当前专辑重建收藏列表条目信息
    pub fn rebuild_loved_list(&mut self) {
        // Update loved entries with fresh data from cache
        let cache = self.song_cache.lock().unwrap();
        for entry in self.loved_cids.values_mut() {
            if let Some(song) = cache.get(&entry.cid) {
                entry.name = song.name.clone();
                entry.artists = song.artists.clone();
            }
        }
        // Also check current album songs
        for song in &self.songs {
            if let Some(entry) = self.loved_cids.get_mut(&song.cid) {
                entry.name = song.name.clone();
                entry.artists = song.artistes.clone();
            }
        }
        self.loved_list = self.loved_cids.values().cloned().collect();
    }

    /// 从 JSON 文件加载收藏数据
    fn load_loved_entries(path: &PathBuf) -> HashMap<String, LovedEntry> {
        let data = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    }

    /// 将收藏数据序列化写入 JSON 文件
    fn save_loved(&self) {
        if let Ok(json) = serde_json::to_string(&self.loved_cids) {
            std::fs::write(&self.loved_path, json).ok();
        }
    }

    /// 将 volume 值 (0-100) 转换为 f32 (0.0-1.0) 并应用到 rodio
    fn apply_volume(&self) {
        if let Some(ref player) = self.player {
            player.set_volume(self.volume as f32 / 100.0);
        }
    }

    /// 启动异步线程：GET /api/albums 获取全量专辑列表
    fn fetch_albums(&mut self) {
        if self.albums_pending.is_some() {
            return;
        }
        let pending = Arc::new(Mutex::new(None));
        let p = pending.clone();
        std::thread::spawn(move || {
            let client = Client::new();
            let result = client.albums().map_err(|e| e.to_string());
            *p.lock().unwrap() = Some(result);
        });
        self.albums_pending = Some(pending);
    }

    /// 轮询专辑列表异步结果：完成后触发首张专辑详情加载
    fn check_albums(&mut self) {
        let completed = if let Some(ref pending) = self.albums_pending {
            pending.lock().unwrap().take()
        } else {
            None
        };
        if let Some(result) = completed {
            match result {
                Ok(list) => {
                    self.album_total = list.len();
                    self.albums = list;
                    self.fetch_album_detail();
                }
                Err(e) => eprintln!("albums: {e}"),
            }
            self.albums_pending = None;
        }
    }

    /// 启动异步线程：GET /api/album/{cid}/detail 获取专辑详情+歌曲列表
    fn fetch_album_detail(&mut self) {
        if self.albums.is_empty() {
            return;
        }
        let album = &self.albums[self.album_index];
        let cid = album.cid.clone();

        self.album_name = Some(album.name.clone());
        self.album_artist = Some(album.artistes.join(", "));
        self.songs_loaded = false;

        {
            let cache = self.detail_cache.lock().unwrap();
            if let Some(detail) = cache.get(&cid) {
                self.songs = detail.songs.clone();
                self.songs_loaded = true;
                drop(cache);
                self.rebuild_loved_list();
                self.preload_adjacent();
                self.preload_song_details();
                return;
            }
        }
        if self.detail_pending.is_some() {
            return;
        }
        let pending = Arc::new(Mutex::new(None));
        let p = pending.clone();
        std::thread::spawn(move || {
            let client = Client::new();
            let result = client.album_detail(&cid).map_err(|e| e.to_string());
            *p.lock().unwrap() = Some(result);
        });
        self.detail_pending = Some(pending);
        self.preload_adjacent();
    }

    /// 轮询专辑详情异步结果：设定歌曲列表 + 重建收藏 + 预取前3首歌
    fn check_detail(&mut self) {
        let completed = if let Some(ref pending) = self.detail_pending {
            pending.lock().unwrap().take()
        } else {
            None
        };
        if let Some(result) = completed {
            match result {
                Ok(detail) => {
                    self.songs = detail.songs.clone();
                    self.songs_loaded = true;
                    self.rebuild_loved_list();
                    self.detail_cache
                        .lock()
                        .unwrap()
                        .insert(detail.cid.clone(), detail);
                    self.preload_song_details();
                }
                Err(e) => eprintln!("detail: {e}"),
            }
            self.detail_pending = None;
        }
    }

    /// 后台预取当前专辑前后各2张专辑的详情（减少切换等待）
    fn preload_adjacent(&mut self) {
        let total = self.albums.len();
        if total <= 1 {
            return;
        }
        for offset in 1..=2i32 {
            for &dir in &[-1i32, 1i32] {
                let idx =
                    (self.album_index as i32 + dir * offset).rem_euclid(total as i32) as usize;
                let cid = self.albums[idx].cid.clone();
                if self.detail_cache.lock().unwrap().contains_key(&cid) {
                    continue;
                }
                let cache = self.detail_cache.clone();
                std::thread::spawn(move || {
                    let client = Client::new();
                    if let Ok(detail) = client.album_detail(&cid) {
                        cache.lock().unwrap().insert(cid, detail);
                    }
                });
            }
        }
    }

    /// 后台预取当前专辑前3首歌的详情（加速 Space 播放）
    fn preload_song_details(&mut self) {
        for song in self.songs.iter().take(3) {
            let cid = song.cid.clone();
            if self.song_cache.lock().unwrap().contains_key(&cid) {
                continue;
            }
            let cache = self.song_cache.clone();
            std::thread::spawn(move || {
                let client = Client::new();
                if let Ok(detail) = client.song_detail(&cid) {
                    cache.lock().unwrap().insert(cid, detail);
                }
            });
        }
    }

    /// 轮询歌曲详情异步结果：缓存并启动播放
    fn check_song(&mut self) {
        let completed = if let Some(ref pending) = self.song_pending {
            pending.lock().unwrap().take()
        } else {
            None
        };
        if let Some(result) = completed {
            match result {
                Ok(song) => {
                    self.song_cache.lock().unwrap().insert(song.cid.clone(), song.clone());
                    self.start_playback(&song);
                }
                Err(e) => eprintln!("song fetch: {e}"),
            }
            self.song_pending = None;
        }
    }

    /// 启动渐进播放流程：后台下载 WAV，首个 chunk 到达即开始播放
    fn start_playback(&mut self, song: &SongDetail) {
        self.pending_song = Some(song.clone());

        self.song_info = Some(format!("Song: {} - {}", song.name, song.artists.join(", ")));
        self.album_intro = self
            .detail_cache
            .lock()
            .unwrap()
            .get(&song.album_cid)
            .map(|d| d.intro.clone());
        self.lyrics.clear();
        self.lyric_index = 0;
        if let Some(ref lyric_url) = song.lyric_url {
            self.fetch_lyrics(lyric_url);
        }

        self.buffering = true;
        self.buffering_msg =
            Some(format!("Buffering: {} - {}", song.name, song.artists.join(", ")));

        if self.player.is_none() {
            self.player = Player::new().ok();
            self.apply_volume();
        }

        // 渐进下载：缓冲区 + 完成标志
        let buf = Arc::new(Mutex::new(Vec::new()));
        let done = Arc::new(Mutex::new(false));
        let b = buf.clone();
        let d = done.clone();
        let source_url = song.source_url.clone();

        std::thread::spawn(move || {
            match ureq::get(&source_url).call() {
                Ok(mut resp) => {
                    // 只读第一个 chunk 知道大小，然后逐步写 buffer
                    let mut chunk = [0u8; 65536];
                    loop {
                        match resp.body_mut().as_reader().read(&mut chunk) {
                            Ok(0) => break,
                            Ok(n) => b.lock().unwrap().extend_from_slice(&chunk[..n]),
                            Err(e) => {
                                eprintln!("stream chunk: {e}");
                                break;
                            }
                        }
                    }
                }
                Err(e) => eprintln!("stream fetch: {e}"),
            }
            *d.lock().unwrap() = true;
        });

        self.stream_buffer = Some(buf);
        self.stream_done = Some(done);
        self.stream_started = false;
        self.stream_switched = false;
        self.wav_pending = None;
    }

    /// 渐进下载帧检查：首个 chunk 到达 → 播放；全曲下载完 → 无缝切换以支持 seek
    fn check_stream(&mut self) {
        if self.stream_buffer.is_none() || !self.buffering {
            return;
        }
        let Some(ref buf) = self.stream_buffer else { return };

        let is_done = self.stream_done.as_ref().map_or(false, |d| *d.lock().unwrap());
        let data_len = buf.lock().unwrap().len();

        // 首个 chunk 到达（4MB 或下载完成）且未开始播放
        if (data_len >= 4 * 1024 * 1024 || is_done) && !self.stream_started {
            self.stream_started = true;
            let data = buf.lock().unwrap().clone();
            if let Some(ref player) = self.player {
                if let Err(e) = player.play_bytes(data) {
                    eprintln!("stream start: {e}");
                    self.buffering = false;
                    self.buffering_msg = None;
                    self.advance_to_next();
                    return;
                }
                self.playing = true;
                if let Some(ref song) = self.pending_song {
                    self.current_song_name =
                        Some(format!("{} - {}", song.name, song.artists.join(", ")));
                    self.current_song_cid = Some(song.cid.clone());
                }
            }
        }

        // 全曲下载完毕 且 首个 chunk 已在播放 且 尚未切换到全曲
        if is_done && self.stream_started && !self.stream_switched {
            self.stream_switched = true;
            let data = buf.lock().unwrap().clone();
            self.wav_data = Some(data.clone());
            self.pending_song = None;

            if let Some(ref player) = self.player {
                let cur = player.elapsed();
                if cur > 0.5 {
                    if let Err(e) = player.play_bytes_at(data, cur) {
                        eprintln!("stream switch: {e}");
                    }
                }
            }
            self.stream_done = None;
            self.buffering = false;
            self.buffering_msg = None;
        }
    }

    /// 轮询 WAV 下载结果（保留兼容，渐进模式不走此路径）
    fn check_wav(&mut self) {
        let completed = if let Some(ref pending) = self.wav_pending {
            pending.lock().unwrap().take()
        } else {
            None
        };
        let Some(result) = completed else { return };
        self.wav_pending = None;

        match result {
            Ok(data) => {
                if let Some(ref player) = self.player {
                    if let Some(ref song) = self.pending_song {
                        if let Err(e) = player.play_bytes(data.clone()) {
                            eprintln!("playback: {e}");
                            self.buffering = false;
                            self.buffering_msg = None;
                            self.advance_to_next();
                            return;
                        }
                        self.playing = true;
                        self.current_song_name = Some(format!(
                            "{} - {}",
                            song.name,
                            song.artists.join(", ")
                        ));
                        self.current_song_cid = Some(song.cid.clone());
                        self.wav_data = Some(data);
                        self.pending_song = None;
                    }
                }
                self.buffering = false;
                self.buffering_msg = None;
            }
            Err(e) => {
                eprintln!("wav download: {e}");
                self.buffering = false;
                self.buffering_msg = None;
                self.advance_to_next();
            }
        }
    }

    /// 下载 .lrc 歌词文件并解析为时间戳列表
    fn fetch_lyrics(&mut self, url: &str) {
        if self.lyric_cache.contains_key(url) {
            self.lyrics = self.lyric_cache.get(url).cloned().unwrap_or_default();
            return;
        }
        match ureq::get(url).call() {
            Ok(mut resp) => {
                let mut data = Vec::new();
                if resp.body_mut().as_reader().read_to_end(&mut data).is_ok() {
                    if let Ok(text) = String::from_utf8(data) {
                        let parsed = Self::parse_lrc(&text);
                        self.lyric_cache.insert(url.to_string(), parsed.clone());
                        self.lyrics = parsed;
                    }
                }
            }
            Err(e) => eprintln!("lyrics fetch: {e}"),
        }
    }

    /// 根据当前播放进度计算匹配的歌词行，同时更新进度条百分比
    fn update_lyric_index(&mut self) {
        self.progress = None;
        if let Some(ref player) = self.player {
            let elapsed = player.elapsed();
            if let Some(dur) = player.duration() {
                if dur > 0.0 {
                    self.progress = Some(elapsed / dur);
                }
            }
            if !self.lyrics.is_empty() {
                let mut idx = 0;
                for (i, (t, _)) in self.lyrics.iter().enumerate() {
                    if *t <= elapsed {
                        idx = i;
                    } else {
                        break;
                    }
                }
                self.lyric_index = idx;
            }
        }
    }

    /// 检测歌曲播放结束：根据播放模式自动切到下一首（Single则停止）
    fn auto_advance(&mut self) {
        if !self.playing || self.buffering {
            return;
        }
        let is_empty = self.player.as_ref().map_or(false, |p| p.is_empty());
        if !is_empty {
            return;
        }
        self.advance_to_next();
    }

    /// 强制跳到下一首（专辑列表循环/随机），下载失败或解码失败时回调
    fn advance_to_next(&mut self) {
        let is_love = matches!(self.play_mode, PlayMode::LoveList | PlayMode::LoveRandom);

        match self.play_mode {
            PlayMode::Single => {
                self.playing = false;
                self.current_song_name = None;
                self.current_song_cid = None;
                self.current_song_index = None;
                self.song_info = None;
                self.album_intro = None;
                self.lyrics.clear();
            }
            PlayMode::LoveRandom | PlayMode::AlbumRandom | PlayMode::GlobalRandom => {
                let len = if is_love {
                    self.loved_list.len()
                } else {
                    self.songs.len()
                };
                if len > 0 {
                    let next = fastrand::usize(..len);
                    self.play_song_at(next);
                }
            }
            _ => {
                let current = self.current_song_index.unwrap_or(0);
                let len = if is_love {
                    self.loved_list.len()
                } else {
                    self.songs.len()
                };
                if len > 0 {
                    let next = (current + 1) % len;
                    self.play_song_at(next);
                }
            }
        }
    }

    /// 解析 LRC 歌词格式 ([mm:ss.xx]文本 → Vec<(秒数, 文本)>)
    fn parse_lrc(text: &str) -> Vec<(f64, String)> {
        let mut result = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some(tag_end) = line.rfind(']') else { continue };
            let tag = &line[1..tag_end];
            let text = line[tag_end + 1..].trim().to_string();
            let parts: Vec<&str> = tag.split(':').collect();
            if parts.len() >= 2 {
                if let (Ok(min), Ok(sec)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    result.push((min * 60.0 + sec, text));
                }
            }
        }
        result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        result
    }
}
