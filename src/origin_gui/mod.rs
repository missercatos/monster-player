use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};

use monster_player::api::types::*;
use monster_player::api::client::Client;
use monster_player::player::Player;

pub fn run() {
    let opts = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_transparent(true)
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "origin-gui",
        opts,
        Box::new(|cc| {
            setup_cjk_fonts(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
    .expect("failed to launch GUI");
}

fn setup_cjk_fonts(ctx: &eframe::egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();

    if let Some(cjk) = find_cjk_font() {
        fonts
            .font_data
            .insert("cjk".into(), cjk);
        fonts
            .families
            .entry(eframe::egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "cjk".into());
        fonts
            .families
            .entry(eframe::egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "cjk".into());
    }

    ctx.set_fonts(fonts);
}

fn find_cjk_font() -> Option<std::sync::Arc<eframe::egui::FontData>> {
    let paths = [
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Light.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Medium.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/wps-office/FZFSK.TTF",
    ];

    for path in paths {
        if let Ok(data) = std::fs::read(path) {
            return Some(std::sync::Arc::new(eframe::egui::FontData::from_owned(data)));
        }
    }
    None
}

#[derive(Clone, Copy, PartialEq)]
enum PlayMode {
    AlbumList,
    AlbumRandom,
    GlobalList,
    GlobalRandom,
    Single,
}

struct App {
    ref_width: Option<f32>,
    ref_height: Option<f32>,
    cover_cache: HashMap<String, eframe::egui::ColorImage>,
    playing: bool,
    play_mode: PlayMode,
    volume: u8,
    show_help: bool,
    albums: Vec<Album>,
    album_index: usize,
    album_detail: Option<AlbumDetail>,
    detail_cache: Arc<Mutex<HashMap<String, AlbumDetail>>>,
    detail_pending: Option<Arc<Mutex<Option<Result<AlbumDetail, String>>>>>,
    albums_pending: Option<Arc<Mutex<Option<Result<Vec<Album>, String>>>>>,
    current_cover_url: Option<String>,
    selected_song: usize,
    current_song: Option<SongDetail>,
    song_cache: HashMap<String, SongDetail>,
    song_pending: Option<Arc<Mutex<Option<Result<SongDetail, String>>>>>,
    player: Option<Player>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            ref_width: None,
            ref_height: None,
            cover_cache: HashMap::new(),
            playing: false,
            play_mode: PlayMode::AlbumList,
            volume: 50,
            show_help: false,
            albums: Vec::new(),
            album_index: 0,
            album_detail: None,
            detail_cache: Arc::new(Mutex::new(HashMap::new())),
            detail_pending: None,
            albums_pending: None,
            current_cover_url: None,
            selected_song: 0,
            current_song: None,
            song_cache: HashMap::new(),
            song_pending: None,
            player: None,
        }
    }
}

impl App {
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
        let cid = self.albums[self.album_index].cid.clone();
        let cover_url = self.albums[self.album_index].cover_url.clone();

        self.selected_song = 0;
        if self.current_cover_url.as_deref() != Some(&cover_url) {
            self.current_cover_url = Some(cover_url.clone());
            self.load_cover(&cover_url);
        }

        {
            let cache = self.detail_cache.lock().unwrap();
            if let Some(detail) = cache.get(&cid) {
                self.album_detail = Some(detail.clone());
                drop(cache);
                self.preload_adjacent();
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
        self.album_detail = None;
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
                    self.detail_cache.lock().unwrap().insert(detail.cid.clone(), detail.clone());
                    self.album_detail = Some(detail);
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
                let idx = (self.album_index as i32 + dir * offset).rem_euclid(total as i32) as usize;
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

    fn prev_album(&mut self) {
        if self.albums.is_empty() {
            return;
        }
        self.album_index = self.album_index.checked_sub(1).unwrap_or(self.albums.len() - 1);
        self.fetch_album_detail();
    }

    fn next_album(&mut self) {
        if self.albums.is_empty() {
            return;
        }
        self.album_index = (self.album_index + 1) % self.albums.len();
        self.fetch_album_detail();
    }

    fn prev_song(&mut self) {
        if let Some(ref detail) = self.album_detail {
            if detail.songs.is_empty() {
                return;
            }
            self.selected_song = self
                .selected_song
                .checked_sub(1)
                .unwrap_or(detail.songs.len() - 1);
        }
    }

    fn next_song(&mut self) {
        if let Some(ref detail) = self.album_detail {
            if detail.songs.is_empty() {
                return;
            }
            self.selected_song = (self.selected_song + 1) % detail.songs.len();
        }
    }

    fn play_selected(&mut self) {
        let song_cid = match self.album_detail.as_ref() {
            Some(detail) => detail.songs.get(self.selected_song).map(|s| s.cid.clone()),
            None => return,
        };
        let Some(cid) = song_cid else { return };

        if let Some(song) = self.song_cache.get(&cid).cloned() {
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

    fn check_song(&mut self) {
        let completed = if let Some(ref pending) = self.song_pending {
            pending.lock().unwrap().take()
        } else {
            None
        };
        if let Some(result) = completed {
            match result {
                Ok(song) => {
                    self.song_cache.insert(song.cid.clone(), song.clone());
                    self.start_playback(&song);
                }
                Err(e) => eprintln!("song fetch: {e}"),
            }
            self.song_pending = None;
        }
    }

    fn start_playback(&mut self, song: &SongDetail) {
        if self.player.is_none() {
            self.player = Player::new().ok();
        }
        if let Some(ref player) = self.player {
            if let Err(e) = player.play_song(song) {
                eprintln!("playback: {e}");
            } else {
                self.playing = true;
                self.current_song = Some(song.clone());
            }
        }
    }

    fn toggle_pause(&mut self) {
        if let Some(ref player) = self.player {
            player.toggle();
            self.playing = !player.is_paused();
        }
    }

    fn cycle_mode(&mut self) {
        self.play_mode = match self.play_mode {
            PlayMode::AlbumList => PlayMode::AlbumRandom,
            PlayMode::AlbumRandom => PlayMode::GlobalList,
            PlayMode::GlobalList => PlayMode::GlobalRandom,
            PlayMode::GlobalRandom => PlayMode::Single,
            PlayMode::Single => PlayMode::AlbumList,
        };
    }

    fn volume_down(&mut self) {
        self.volume = self.volume.saturating_sub(5);
    }

    fn volume_up(&mut self) {
        self.volume = (self.volume + 5).min(100);
    }

    fn load_cover(&mut self, url: &str) {
        if self.cover_cache.contains_key(url) {
            return;
        }
        match ureq::get(url).call() {
            Ok(mut resp) => {
                let mut data = Vec::new();
                if resp.body_mut().as_reader().read_to_end(&mut data).is_ok() {
                    if let Ok(img) = image::load_from_memory(&data) {
                        let rgba = img.to_rgba8();
                        let size = rgba.dimensions();
                        let color = eframe::egui::ColorImage::from_rgba_unmultiplied(
                            [size.0 as usize, size.1 as usize],
                            rgba.as_raw(),
                        );
                        self.cover_cache.insert(url.to_string(), color);
                    }
                }
            }
            Err(e) => eprintln!("cover download failed: {e}"),
        }
    }

    fn render_cover(&self, ui: &mut eframe::egui::Ui, side: f32) {
        if let Some(url) = &self.current_cover_url {
            if let Some(img) = self.cover_cache.get(url) {
                let tex = ui.ctx().load_texture(
                    url,
                    img.clone(),
                    eframe::egui::TextureOptions::LINEAR,
                );
                let tex_size = tex.size_vec2();
                let scale = (side / tex_size.x).min(side / tex_size.y);
                let fit_w = tex_size.x * scale;
                let fit_h = tex_size.y * scale;
                ui.add(
                    eframe::egui::Image::from_texture(eframe::egui::load::SizedTexture::from_handle(&tex))
                        .fit_to_exact_size(eframe::egui::vec2(fit_w, fit_h)),
                );
                return;
            }
        }

        let rect = eframe::egui::Rect::from_center_size(
            ui.available_rect_before_wrap().center(),
            eframe::egui::vec2(side, side),
        );
        ui.painter().rect_filled(
            rect,
            eframe::egui::CornerRadius::ZERO,
            eframe::egui::Color32::from_gray(60),
        );
    }

    fn render_bottom(&self, ui: &mut eframe::egui::Ui) {
        let font_id = eframe::egui::FontId::proportional(16.0);

        let play_state = if self.playing { "O Playing" } else { "X Paused" };

        let mode_text = match self.play_mode {
            PlayMode::AlbumList => "Album List",
            PlayMode::AlbumRandom => "Album Random",
            PlayMode::GlobalList => "Global List",
            PlayMode::GlobalRandom => "Global Random",
            PlayMode::Single => "Single",
        };

        let volume_str = format!("{}%", self.volume);

        let mut items: Vec<String> = if self.show_help {
            vec!["Ctrl+T  Toggle help".into()]
        } else {
            vec![
                play_state.into(),
                mode_text.into(),
                volume_str,
                "Press Ctrl+T for help".into(),
            ]
        };

        if !self.show_help {
            if let Some(ref song) = self.current_song {
                let song_info = format!("{} - {}", song.name, song.artists.join(", "));
                items.insert(1, song_info);
            }
        }

        let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

        let n = item_refs.len() as f32;
        let avail = ui.available_rect_before_wrap();
        let row_h = avail.height() / n;

        for (i, text) in item_refs.iter().enumerate() {
            let row_rect = eframe::egui::Rect::from_min_size(
                eframe::egui::pos2(avail.left(), avail.top() + i as f32 * row_h),
                eframe::egui::vec2(avail.width(), row_h),
            );
            ui.allocate_ui_at_rect(row_rect, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        eframe::egui::RichText::new(*text)
                            .font(font_id.clone())
                            .color(eframe::egui::Color32::WHITE),
                    );
                });
            });
        }
    }

    fn render_right(&mut self, ui: &mut eframe::egui::Ui) {
        if self.albums.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    eframe::egui::RichText::new("Loading...")
                        .color(eframe::egui::Color32::WHITE),
                );
            });
            return;
        }

        let size = ui.available_size();
        let top_h = size.y * 0.2;
        let line_h = 2.0;
        let bot_h = size.y - top_h - line_h;

        let top_rect = eframe::egui::Rect::from_min_size(
            ui.next_widget_position(),
            eframe::egui::vec2(size.x, top_h),
        );
        let line_rect = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(top_rect.left(), top_rect.bottom()),
            eframe::egui::vec2(size.x, line_h),
        );
        let bot_rect = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(line_rect.left(), line_rect.bottom()),
            eframe::egui::vec2(size.x, bot_h),
        );

        ui.painter().line_segment(
            [line_rect.left_center(), line_rect.right_center()],
            eframe::egui::Stroke::new(line_h, eframe::egui::Color32::WHITE),
        );

        ui.allocate_ui_at_rect(top_rect, |ui| {
            let album = &self.albums[self.album_index];
            ui.vertical_centered(|ui| {
                ui.label(
                    eframe::egui::RichText::new(&album.name)
                        .font(eframe::egui::FontId::proportional(24.0))
                        .color(eframe::egui::Color32::WHITE),
                );
                let artists = album.artistes.join(", ");
                ui.label(
                    eframe::egui::RichText::new(artists)
                        .font(eframe::egui::FontId::proportional(17.0))
                        .color(eframe::egui::Color32::LIGHT_GRAY),
                );
                if self.albums.len() > 1 {
                    ui.label(
                        eframe::egui::RichText::new(format!(
                            "[{}/{}]",
                            self.album_index + 1,
                            self.albums.len()
                        ))
                        .font(eframe::egui::FontId::proportional(14.0))
                        .color(eframe::egui::Color32::GRAY),
                    );
                }
            });
        });

        ui.allocate_ui_at_rect(bot_rect, |ui| {
            if let Some(ref detail) = self.album_detail {
                ui.vertical_centered(|ui| {
                    if !detail.intro.is_empty() {
                        ui.label(
                            eframe::egui::RichText::new(&detail.intro)
                                .font(eframe::egui::FontId::proportional(15.0))
                                .color(eframe::egui::Color32::GRAY),
                        );
                        ui.add_space(4.0);
                    }
                    for (i, song) in detail.songs.iter().enumerate() {
                        let is_playing = self
                            .current_song
                            .as_ref()
                            .map_or(false, |cs| cs.cid == song.cid);
                        let prefix = if i == self.selected_song { "> " } else { "  " };
                        let color = if is_playing {
                            eframe::egui::Color32::from_rgb(0, 200, 200)
                        } else {
                            eframe::egui::Color32::WHITE
                        };
                        ui.label(
                            eframe::egui::RichText::new(format!(
                                "{}{} - {}",
                                prefix,
                                song.name,
                                song.artistes.join(", ")
                            ))
                            .font(eframe::egui::FontId::proportional(16.0))
                            .color(color),
                        );
                    }
                });
            } else if self.detail_pending.is_some() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        eframe::egui::RichText::new("Loading songs...")
                            .color(eframe::egui::Color32::GRAY),
                    );
                });
            }
        });
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &eframe::egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.albums.is_empty() && self.detail_pending.is_none() {
            self.fetch_albums();
        }

        self.check_albums();
        self.check_detail();
        self.check_song();

        if ctx.input(|i| i.key_pressed(eframe::egui::Key::Space)) {
            self.play_selected();
        }
        if ctx.input(|i| i.key_pressed(eframe::egui::Key::X)) {
            self.toggle_pause();
        }
        if ctx.input(|i| i.key_pressed(eframe::egui::Key::E)) {
            self.cycle_mode();
        }
        if ctx.input(|i| i.key_pressed(eframe::egui::Key::O)) {
            self.volume_down();
        }
        if ctx.input(|i| i.key_pressed(eframe::egui::Key::P)) {
            self.volume_up();
        }

        let pressed_up = ctx.input(|i| {
            i.key_pressed(eframe::egui::Key::K)
                || i.key_pressed(eframe::egui::Key::ArrowUp)
        });
        let pressed_down = ctx.input(|i| {
            i.key_pressed(eframe::egui::Key::J)
                || i.key_pressed(eframe::egui::Key::ArrowDown)
        });
        let pressed_left = ctx.input(|i| {
            i.key_pressed(eframe::egui::Key::H)
                || i.key_pressed(eframe::egui::Key::ArrowLeft)
        });
        let pressed_right = ctx.input(|i| {
            i.key_pressed(eframe::egui::Key::L)
                || i.key_pressed(eframe::egui::Key::ArrowRight)
        });

        if pressed_up {
            self.prev_song();
        }
        if pressed_down {
            self.next_song();
        }
        if pressed_left {
            self.prev_album();
        }
        if pressed_right {
            self.next_album();
        }

        if ctx.input(|i| i.key_pressed(eframe::egui::Key::T) && i.modifiers.ctrl) {
            self.show_help = !self.show_help;
        }

        eframe::egui::CentralPanel::default()
            .frame(
                eframe::egui::Frame::new()
                    .fill(eframe::egui::Color32::from_rgba_premultiplied(0, 0, 0, 120))
                    .stroke(eframe::egui::Stroke::new(3.0, eframe::egui::Color32::WHITE))
                    .outer_margin(eframe::egui::Margin::same(11))
                    .inner_margin(eframe::egui::Margin::same(8))
                    .corner_radius(eframe::egui::CornerRadius::same(4)),
            )
            .show(ctx, |ui| {
                let size = ui.available_size();

                if self.ref_width.is_none() {
                    self.ref_width = Some(size.x);
                    self.ref_height = Some(size.y);
                } else {
                    let ref_w = self.ref_width.unwrap();
                    let ref_h = self.ref_height.unwrap();
                    if size.x > ref_w || size.y > ref_h {
                        self.ref_width = Some(size.x.max(ref_w));
                        self.ref_height = Some(size.y.max(ref_h));
                    }
                }

                let shrink = size.x < self.ref_width.unwrap() * 0.5
                    || size.y < self.ref_height.unwrap() * 0.5;

                let left_w = size.x * 0.3;
                let line_w = 2.0;
                let right_w = size.x - left_w - line_w;

                let left_rect = eframe::egui::Rect::from_min_size(
                    ui.next_widget_position(),
                    eframe::egui::vec2(left_w, size.y),
                );
                let line_rect = eframe::egui::Rect::from_min_size(
                    eframe::egui::pos2(left_rect.right(), left_rect.top()),
                    eframe::egui::vec2(line_w, size.y),
                );
                let right_rect = eframe::egui::Rect::from_min_size(
                    eframe::egui::pos2(line_rect.right(), line_rect.top()),
                    eframe::egui::vec2(right_w, size.y),
                );

                ui.painter().line_segment(
                    [line_rect.center_top(), line_rect.center_bottom()],
                    eframe::egui::Stroke::new(line_w, eframe::egui::Color32::WHITE),
                );

                ui.allocate_ui_at_rect(left_rect, |ui| {
                    let lsize = ui.available_size();

                    if shrink {
                        ui.centered_and_justified(|ui| {
                            self.render_bottom(ui);
                        });
                    } else {
                        let top_h = lsize.y * 0.5;
                        let line_h = 2.0;
                        let bot_h = lsize.y - top_h - line_h;

                        let top_rect = eframe::egui::Rect::from_min_size(
                            ui.next_widget_position(),
                            eframe::egui::vec2(lsize.x, top_h),
                        );
                        let line_rect = eframe::egui::Rect::from_min_size(
                            eframe::egui::pos2(top_rect.left(), top_rect.bottom()),
                            eframe::egui::vec2(lsize.x, line_h),
                        );
                        let bot_rect = eframe::egui::Rect::from_min_size(
                            eframe::egui::pos2(line_rect.left(), line_rect.bottom()),
                            eframe::egui::vec2(lsize.x, bot_h),
                        );

                        ui.painter().line_segment(
                            [line_rect.left_center(), line_rect.right_center()],
                            eframe::egui::Stroke::new(line_h, eframe::egui::Color32::WHITE),
                        );

                        ui.allocate_ui_at_rect(top_rect, |ui| {
                            let tsize = ui.available_size();
                            let side = tsize.x.min(tsize.y) * 0.5;
                            ui.centered_and_justified(|ui| {
                                self.render_cover(ui, side);
                            });
                        });

                        ui.allocate_ui_at_rect(bot_rect, |ui| {
                            ui.centered_and_justified(|ui| {
                                self.render_bottom(ui);
                            });
                        });
                    }
                });

                ui.allocate_ui_at_rect(right_rect, |ui| {
                    self.render_right(ui);
                });
            });
    }
}
