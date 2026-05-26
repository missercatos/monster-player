use monster_player::kernel::{Engine, LovedEntry, PlayMode};

pub struct App {
    pub engine: Engine,
    pub show_help: bool,
    pub show_lyrics: bool,
    pub selected_song: usize,
    is_love_view: bool,
}

impl App {
    /// 初始化播放引擎，默认隐藏帮助与歌词，选中第一首歌
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
            show_help: false,
            show_lyrics: false,
            selected_song: 0,
            is_love_view: false,
        }
    }

    /// 驱动引擎更新，检测收藏列表/普通列表切换，同步 selected_song 归零
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

    /// 播放当前选中的歌曲
    pub fn play_selected(&mut self) {
        self.engine.play_song_at(self.selected_song);
    }

    /// 暂停 / 恢复播放
    pub fn toggle_pause(&mut self) {
        self.engine.toggle_pause();
    }

    /// 切换到下一张专辑，选中复位到第一首歌
    pub fn next_album(&mut self) {
        self.selected_song = 0;
        self.engine.next_album();
    }

    /// 切换到上一张专辑，选中复位到第一首歌
    pub fn prev_album(&mut self) {
        self.selected_song = 0;
        self.engine.prev_album();
    }

    /// 根据当前视图返回歌曲数量（收藏视图取 loved_list，否则取 songs）
    fn song_count(&self) -> usize {
        if self.is_love_view {
            self.engine.loved_list.len()
        } else {
            self.engine.songs.len()
        }
    }

    /// 选中下一首歌，Single 模式则直接重播当前歌曲
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

    /// 选中上一首歌，Single 模式则直接重播当前歌曲
    pub fn prev_song(&mut self) {
        if matches!(self.engine.play_mode, PlayMode::Single) {
            self.engine.restart_song();
            return;
        }
        let len = self.song_count();
        if len > 0 {
            self.selected_song = self
                .selected_song
                .checked_sub(1)
                .unwrap_or(len - 1);
        }
    }

    /// Shift+A：根据模式切歌并立即播放（列表=上一首，随机=随机抽，单曲=重播）
    pub fn play_prev(&mut self) {
        let rand = matches!(
            self.engine.play_mode,
            PlayMode::AlbumRandom | PlayMode::GlobalRandom | PlayMode::LoveRandom
        );
        if matches!(self.engine.play_mode, PlayMode::Single) {
            self.engine.restart_song();
            return;
        }
        let len = self.song_count();
        if len > 0 {
            if rand {
                self.selected_song = fastrand::usize(..len);
            } else {
                self.selected_song = self.selected_song.checked_sub(1).unwrap_or(len - 1);
            }
            self.play_selected();
        }
    }

    /// Shift+D：根据模式切歌并立即播放（列表=下一首，随机=随机抽，单曲=重播）
    pub fn play_next(&mut self) {
        let rand = matches!(
            self.engine.play_mode,
            PlayMode::AlbumRandom | PlayMode::GlobalRandom | PlayMode::LoveRandom
        );
        if matches!(self.engine.play_mode, PlayMode::Single) {
            self.engine.restart_song();
            return;
        }
        let len = self.song_count();
        if len > 0 {
            if rand {
                self.selected_song = fastrand::usize(..len);
            } else {
                self.selected_song = (self.selected_song + 1) % len;
            }
            self.play_selected();
        }
    }

    /// 循环切换播放模式（列表 → 随机 → 单曲 → …）
    pub fn cycle_mode(&mut self) {
        self.engine.cycle_mode();
    }

    /// 增加音量
    pub fn volume_up(&mut self) {
        self.engine.volume_up();
    }

    /// 降低音量
    pub fn volume_down(&mut self) {
        self.engine.volume_down();
    }

    /// 快进 5 秒
    pub fn seek_forward(&mut self) {
        self.engine.seek_forward();
    }

    /// 快退 5 秒
    pub fn seek_backward(&mut self) {
        self.engine.seek_backward();
    }

    /// 切换帮助面板的显示 / 隐藏
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// 切换歌词视图的显示 / 隐藏
    pub fn toggle_lyrics(&mut self) {
        self.show_lyrics = !self.show_lyrics;
    }

    /// 切换当前选中歌曲的收藏状态
    pub fn toggle_love(&mut self) {
        let entry = if self.is_love_view {
            self.engine.loved_list.get(self.selected_song).cloned()
        } else {
            self.engine.songs.get(self.selected_song).map(|s| LovedEntry {
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
