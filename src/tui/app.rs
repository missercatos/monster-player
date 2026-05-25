use monster_player::kernel::{Engine, LovedEntry, PlayMode};

pub struct App {
    pub engine: Engine,
    pub show_help: bool,
    pub show_lyrics: bool,
    pub selected_song: usize,
    is_love_view: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
            show_help: false,
            show_lyrics: false,
            selected_song: 0,
            is_love_view: false,
        }
    }

    pub fn update(&mut self) {
        let now_love = matches!(
            self.engine.play_mode,
            PlayMode::LoveList | PlayMode::LoveRandom
        );
        if now_love != self.is_love_view {
            self.is_love_view = now_love;
            self.selected_song = 0;
        }
        self.engine.update();
    }

    pub fn play_selected(&mut self) {
        self.engine.play_song_at(self.selected_song);
    }

    pub fn toggle_pause(&mut self) {
        self.engine.toggle_pause();
    }

    pub fn next_album(&mut self) {
        self.selected_song = 0;
        self.engine.next_album();
    }

    pub fn prev_album(&mut self) {
        self.selected_song = 0;
        self.engine.prev_album();
    }

    fn song_count(&self) -> usize {
        if self.is_love_view {
            self.engine.loved_list.len()
        } else {
            self.engine.songs.len()
        }
    }

    pub fn next_song(&mut self) {
        if matches!(self.engine.play_mode, PlayMode::Single) {
            self.engine.restart_song();
            return;
        }
        let len = self.song_count();
        if len > 0 {
            self.selected_song = (self.selected_song + 1) % len;
        }
    }

    pub fn prev_song(&mut self) {
        if matches!(self.engine.play_mode, PlayMode::Single) {
            self.engine.restart_song();
            return;
        }
        let len = self.song_count();
        if len > 0 {
            self.selected_song = self.selected_song.checked_sub(1).unwrap_or(len - 1);
        }
    }

    pub fn cycle_mode(&mut self) {
        self.engine.cycle_mode();
    }

    pub fn volume_up(&mut self) {
        self.engine.volume_up();
    }

    pub fn volume_down(&mut self) {
        self.engine.volume_down();
    }

    pub fn seek_forward(&mut self) {
        self.engine.seek_forward();
    }

    pub fn seek_backward(&mut self) {
        self.engine.seek_backward();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_lyrics(&mut self) {
        self.show_lyrics = !self.show_lyrics;
    }

    pub fn toggle_love(&mut self) {
        let entry = if self.is_love_view {
            self.engine.loved_list.get(self.selected_song).cloned()
        } else {
            self.engine
                .songs
                .get(self.selected_song)
                .map(|s| LovedEntry {
                    cid: s.cid.clone(),
                    name: s.name.clone(),
                    artists: s.artistes.clone(),
                })
        };
        if let Some(e) = entry {
            self.engine.toggle_love(&e.cid, &e.name, &e.artists);
        }
    }
}
