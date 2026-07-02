use monster_player::kernel::{Engine, LovedEntry, PlayMode};
use monster_player::api::types::Song;

/// TUI 应用状态：包装 Engine 内核 + UI 专属状态
pub struct App {
    pub engine: Engine,
    pub show_help: bool,
    pub show_lyrics: bool,
    pub selected_song: usize,
    is_love_view: bool,
    pub search_mode: bool,
    pub search_query: String,
    pub search_results: Vec<Song>,
    pub search_index: usize,
    pub search_confirmed: bool,
}

impl App {
    /// 创建 Engine 实例
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
            show_help: false,
            show_lyrics: false,
            selected_song: 0,
            is_love_view: false,
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            search_confirmed: false,
        }
    }

    /// 检测播放模式切换 + 调用 engine.update()
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

    /// → engine.play_song_at(selected_song)
    pub fn play_selected(&mut self) {
        let index = if self.engine.is_global_mode() {
            self.engine.global_index
        } else {
            self.selected_song
        };
        self.engine.play_song_at(index);
    }

    /// → engine.toggle_pause()
    pub fn toggle_pause(&mut self) {
        self.engine.toggle_pause();
    }

    /// → engine.next_album()
    pub fn next_album(&mut self) {
        self.selected_song = 0;
        self.engine.next_album();
    }

    /// → engine.prev_album()
    pub fn prev_album(&mut self) {
        self.selected_song = 0;
        self.engine.prev_album();
    }

    fn song_count(&self) -> usize {
        if self.is_love_view {
            self.engine.loved_list.len()
        } else if self.engine.is_global_mode() {
            self.engine.global_playlist.len()
        } else {
            self.engine.songs.len()
        }
    }

    /// 选中下移（支持回绕），单曲模式→ engine.restart_song()
    pub fn next_song(&mut self) {
        if matches!(self.engine.play_mode, PlayMode::Single) {
            self.engine.restart_song();
            return;
        }
        // 全局模式：仅移动 global_index，不自动播放
        if self.engine.is_global_mode() {
            let len = self.engine.global_playlist.len();
            if len > 0 {
                self.engine.global_index = (self.engine.global_index + 1) % len;
            }
            return;
        }
        let len = self.song_count();
        if len > 0 {
            self.selected_song = (self.selected_song + 1) % len;
        }
    }

    /// 选中上移（支持回绕），单曲模式→ engine.restart_song()
    pub fn prev_song(&mut self) {
        if matches!(self.engine.play_mode, PlayMode::Single) {
            self.engine.restart_song();
            return;
        }
        // 全局模式：仅移动 global_index，不自动播放
        if self.engine.is_global_mode() {
            let len = self.engine.global_playlist.len();
            if len > 0 {
                self.engine.global_index = self.engine.global_index.checked_sub(1)
                    .unwrap_or(len - 1);
            }
            return;
        }
        let len = self.song_count();
        if len > 0 {
            self.selected_song = self.selected_song.checked_sub(1).unwrap_or(len - 1);
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

        // 全局模式：直接调用 engine 的 advance 逻辑
        if self.engine.is_global_mode() {
            self.engine.play_prev_global();
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

        // 全局模式：直接调用 engine 的 advance 逻辑
        if self.engine.is_global_mode() {
            self.engine.play_next_global();
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

    /// → engine.cycle_mode()
    pub fn cycle_mode(&mut self) {
        self.engine.cycle_mode();
    }

    /// → engine.volume_up()
    pub fn volume_up(&mut self) {
        self.engine.volume_up();
    }

    /// → engine.volume_down()
    pub fn volume_down(&mut self) {
        self.engine.volume_down();
    }

    /// → engine.seek_forward()
    pub fn seek_forward(&mut self) {
        self.engine.seek_forward();
    }

    /// → engine.seek_backward()
    pub fn seek_backward(&mut self) {
        self.engine.seek_backward();
    }

    /// 切换帮助面板显示
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// 切换歌词面板显示
    pub fn toggle_lyrics(&mut self) {
        self.show_lyrics = !self.show_lyrics;
    }

    /// → engine.toggle_love(cid, name, artists)
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

    /// 进入搜索模式
    pub fn enter_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_results.clear();
        self.search_index = 0;
        self.search_confirmed = false;
        self.engine.fetch_all_songs();
    }

    /// 退出搜索模式
    pub fn exit_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_results.clear();
        self.search_index = 0;
        self.search_confirmed = false;
    }

    /// 向搜索框追加字符
    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.search_confirmed = false;
        self.update_search_results();
    }

    /// 删除搜索框最后一个字符
    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.search_confirmed = false;
        self.update_search_results();
    }

    /// 选择上一个搜索结果
    pub fn search_prev(&mut self) {
        if !self.search_results.is_empty() {
            self.search_index = self.search_index.checked_sub(1)
                .unwrap_or(self.search_results.len() - 1);
        }
    }

    /// 选择下一个搜索结果
    pub fn search_next(&mut self) {
        if !self.search_results.is_empty() {
            self.search_index = (self.search_index + 1) % self.search_results.len();
        }
    }

    /// 确认搜索结果：第一次选中并补全，第二次跳转
    pub fn search_confirm(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        if !self.search_confirmed {
            // 第一次回车：选中结果，补全查询
            if let Some(song) = self.search_results.get(self.search_index) {
                self.search_query = song.name.clone();
                self.search_confirmed = true;
                self.update_search_results();
                // 如果只有一个结果，直接跳转
                if self.search_results.len() == 1 {
                    self.search_jump();
                }
            }
        } else {
            // 第二次回车：跳转到专辑
            self.search_jump();
        }
    }

    /// 跳转到选中的搜索结果所在专辑
    fn search_jump(&mut self) {
        if let Some(song) = self.search_results.get(self.search_index) {
            let cid = song.cid.clone();
            if self.engine.jump_to_song(&cid) {
                self.selected_song = self.engine.current_song_index.unwrap_or(0);
                self.exit_search();
            }
        }
    }

    /// 根据当前查询更新搜索结果
    fn update_search_results(&mut self) {
        self.search_results = self.engine.search_songs(&self.search_query);
        self.search_index = 0;
    }
}
