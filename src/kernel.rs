use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::api::client::Client;
use crate::api::types::*;
use crate::player::Player;

#[derive(Clone, Copy, PartialEq)]
pub enum PlayMode {
    AlbumList,
    AlbumRandom,
    GlobalList,
    GlobalRandom,
    Single,
    LoveList,
    LoveRandom,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LovedEntry {
    pub cid: String,
    pub name: String,
    pub artists: Vec<String>,
}

pub struct Engine {
    pub albums: Vec<Album>,
    pub album_index: usize,
    pub songs: Vec<AlbumSong>,
    pub songs_loaded: bool,
    pub album_name: Option<String>,
    pub album_artist: Option<String>,
    pub album_total: usize,

    pub playing: bool,
    pub volume: u8,
    pub play_mode: PlayMode,
    pub current_song_name: Option<String>,
    pub current_song_cid: Option<String>,
    pub current_song_index: Option<usize>,
    pub song_info: Option<String>,
    pub album_intro: Option<String>,
    pub buffering: bool,
    pub buffering_msg: Option<String>,

    pub lyrics: Vec<(f64, String)>,
    pub lyric_index: usize,
    pub progress: Option<f64>,

    pub loved_cids: HashMap<String, LovedEntry>,
    pub loved_list: Vec<LovedEntry>,

    wav_data: Option<Vec<u8>>,
    loved_path: PathBuf,

    detail_cache: Arc<Mutex<HashMap<String, AlbumDetail>>>,
    detail_pending: Option<Arc<Mutex<Option<Result<AlbumDetail, String>>>>>,
    albums_pending: Option<Arc<Mutex<Option<Result<Vec<Album>, String>>>>>,
    song_cache: Arc<Mutex<HashMap<String, SongDetail>>>,
    song_pending: Option<Arc<Mutex<Option<Result<SongDetail, String>>>>>,
    lyric_cache: HashMap<String, Vec<(f64, String)>>,
    player: Option<Player>,
    wav_pending: Option<Arc<Mutex<Option<Result<Vec<u8>, String>>>>>,
    pending_song: Option<SongDetail>,
}

impl Engine {
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

    pub fn update(&mut self) {
        if self.albums.is_empty() && self.albums_pending.is_none() {
            self.fetch_albums();
        }
        self.check_albums();
        self.check_detail();
        self.check_song();
        self.check_wav();
        self.update_lyric_index();
        self.auto_advance();
    }

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

    fn play_cid(&mut self, cid: &str) {
        let cached = self.song_cache.lock().unwrap().get(cid).cloned();
        if let Some(song) = cached {
            self.start_playback(&song);
            return;
        }
        if self.song_pending.is_some() {
            return;
        }
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

    pub fn toggle_pause(&mut self) {
        if let Some(ref player) = self.player {
            player.toggle();
            self.playing = !player.is_paused();
        }
    }

    pub fn next_album(&mut self) {
        if self.albums.is_empty() {
            return;
        }
        self.album_index = (self.album_index + 1) % self.albums.len();
        self.fetch_album_detail();
    }

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

    pub fn volume_up(&mut self) {
        self.volume = (self.volume + 5).min(100);
        self.apply_volume();
    }

    pub fn volume_down(&mut self) {
        self.volume = self.volume.saturating_sub(5);
        self.apply_volume();
    }

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

    pub fn restart_song(&mut self) {
        if let (Some(player), Some(data)) = (self.player.as_ref(), self.wav_data.as_ref()) {
            if let Err(e) = player.play_bytes_at(data.clone(), 0.0) {
                eprintln!("restart: {e}");
            }
        }
    }

    pub fn is_loved(&self, cid: &str) -> bool {
        self.loved_cids.contains_key(cid)
    }

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

    fn load_loved_entries(path: &PathBuf) -> HashMap<String, LovedEntry> {
        let data = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    }

    fn save_loved(&self) {
        if let Ok(json) = serde_json::to_string(&self.loved_cids) {
            std::fs::write(&self.loved_path, json).ok();
        }
    }

    fn apply_volume(&self) {
        if let Some(ref player) = self.player {
            player.set_volume(self.volume as f32 / 100.0);
        }
    }

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

        let source_url = song.source_url.clone();
        let pending = Arc::new(Mutex::new(None));
        let p = pending.clone();
        std::thread::spawn(move || {
            let result = match ureq::get(&source_url).call() {
                Ok(mut resp) => {
                    let mut data = Vec::new();
                    match resp.body_mut().as_reader().read_to_end(&mut data) {
                        Ok(_) => Ok(data),
                        Err(e) => Err(format!("download: {e}")),
                    }
                }
                Err(e) => Err(format!("fetch: {e}")),
            };
            *p.lock().unwrap() = Some(result);
        });
        self.wav_pending = Some(pending);
    }

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
            }
        }
    }

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

    fn auto_advance(&mut self) {
        if !self.playing || self.buffering {
            return;
        }
        let is_empty = self.player.as_ref().map_or(false, |p| p.is_empty());
        if !is_empty {
            return;
        }

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
