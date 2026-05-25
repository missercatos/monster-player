use std::collections::HashMap;
use std::io::Read;
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
    pub song_info: Option<String>,
    pub album_intro: Option<String>,
    pub buffering: bool,
    pub buffering_msg: Option<String>,

    pub lyrics: Vec<(f64, String)>,
    pub lyric_index: usize,
    pub progress: Option<f64>,

    wav_data: Option<Vec<u8>>,

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
            song_info: None,
            album_intro: None,
            buffering: false,
            buffering_msg: None,
            lyrics: Vec::new(),
            lyric_index: 0,
            progress: None,
            wav_data: None,
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
    }

    pub fn play_song_at(&mut self, index: usize) {
        if self.songs.is_empty() {
            return;
        }
        let cid = self.songs[index].cid.clone();

        let cached = self.song_cache.lock().unwrap().get(&cid).cloned();
        if let Some(song) = cached {
            self.start_playback(&song);
            return;
        }
        if self.song_pending.is_some() {
            return;
        }
        let pending = Arc::new(Mutex::new(None));
        let p = pending.clone();
        std::thread::spawn(move || {
            let client = Client::new();
            let result = client.song_detail(&cid).map_err(|e| e.to_string());
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
            PlayMode::Single => PlayMode::AlbumList,
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
