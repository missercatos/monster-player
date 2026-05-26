use std::collections::HashMap;
use std::io::Read;

use eframe::egui;
use monster_player::kernel::PlayMode;

use super::app::App;

/// GUI 专属渲染状态：窗口尺寸参考 + 封面缓存
pub struct GuiState {
    pub ref_width: Option<f32>,
    pub ref_height: Option<f32>,
    pub cover_cache: HashMap<String, egui::ColorImage>,
    pub current_cover_url: Option<String>,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            ref_width: None,
            ref_height: None,
            cover_cache: HashMap::new(),
            current_cover_url: None,
        }
    }

    /// 下载封面图片，解码为 egui ColorImage 纹理
    pub fn load_cover(&mut self, url: &str) {
        if self.cover_cache.contains_key(url) {
            return;
        }
        if let Ok(mut resp) = ureq::get(url).call() {
            let mut data = Vec::new();
            if resp.body_mut().as_reader().read_to_end(&mut data).is_ok() {
                if let Ok(img) = image::load_from_memory(&data) {
                    let rgba = img.to_rgba8();
                    let size = rgba.dimensions();
                    let color = egui::ColorImage::from_rgba_unmultiplied(
                        [size.0 as usize, size.1 as usize],
                        rgba.as_raw(),
                    );
                    self.cover_cache.insert(url.to_string(), color);
                }
            }
        }
    }

    /// 检测专辑切换，触发封面加载
    pub fn check_cover(&mut self, app: &App) {
        if app.engine.albums.is_empty() {
            return;
        }
        let url = &app.engine.albums[app.engine.album_index].cover_url;
        if self.current_cover_url.as_deref() != Some(url) {
            self.current_cover_url = Some(url.clone());
            self.load_cover(url);
        }
    }
}

/// 主帧函数：键盘处理 + 左右分栏布局
pub fn update(app: &mut App, gui: &mut GuiState, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    app.update();
    gui.check_cover(app);

    // Keyboard handling
    if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
        app.play_selected();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::X)) {
        app.toggle_pause();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::E)) {
        app.cycle_mode();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::O)) {
        app.volume_down();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::P)) {
        app.volume_up();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::V)) {
        app.toggle_lyrics();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::S)) {
        app.toggle_love();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::A)) {
        if ctx.input(|i| i.modifiers.shift) {
            app.play_prev();
        } else {
            app.seek_backward();
        }
    }
    if ctx.input(|i| i.key_pressed(egui::Key::D)) {
        if ctx.input(|i| i.modifiers.shift) {
            app.play_next();
        } else {
            app.seek_forward();
        }
    }

    let pressed_up =
        ctx.input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp));
    let pressed_down =
        ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown));
    let pressed_left =
        ctx.input(|i| i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft));
    let pressed_right =
        ctx.input(|i| i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight));

    if pressed_up {
        app.prev_song();
    }
    if pressed_down {
        app.next_song();
    }
    if pressed_left {
        app.prev_album();
    }
    if pressed_right {
        app.next_album();
    }

    if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.ctrl) {
        app.toggle_help();
    }

    // Layout
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 120))
                .stroke(egui::Stroke::new(3.0, egui::Color32::WHITE))
                .outer_margin(egui::Margin::same(11))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(egui::CornerRadius::same(4)),
        )
        .show(ctx, |ui| {
            let size = ui.available_size();

            if gui.ref_width.is_none() {
                gui.ref_width = Some(size.x);
                gui.ref_height = Some(size.y);
            } else {
                let ref_w = gui.ref_width.unwrap();
                let ref_h = gui.ref_height.unwrap();
                if size.x > ref_w || size.y > ref_h {
                    gui.ref_width = Some(size.x.max(ref_w));
                    gui.ref_height = Some(size.y.max(ref_h));
                }
            }

            let shrink = size.x < gui.ref_width.unwrap() * 0.5
                || size.y < gui.ref_height.unwrap() * 0.5;

            let left_w = size.x * 0.3;
            let line_w = 2.0;
            let right_w = size.x - left_w - line_w;

            let left_rect = egui::Rect::from_min_size(
                ui.next_widget_position(),
                egui::vec2(left_w, size.y),
            );
            let line_rect = egui::Rect::from_min_size(
                egui::pos2(left_rect.right(), left_rect.top()),
                egui::vec2(line_w, size.y),
            );
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(line_rect.right(), line_rect.top()),
                egui::vec2(right_w, size.y),
            );

            ui.painter().line_segment(
                [line_rect.center_top(), line_rect.center_bottom()],
                egui::Stroke::new(line_w, egui::Color32::WHITE),
            );

            ui.allocate_ui_at_rect(left_rect, |ui| {
                let lsize = ui.available_size();

                if shrink {
                    ui.centered_and_justified(|ui| {
                        draw_bottom(app, ui);
                    });
                } else {
                    let top_h = lsize.y * 0.5;
                    let line_h = 2.0;
                    let bot_h = lsize.y - top_h - line_h;

                    let top_rect = egui::Rect::from_min_size(
                        ui.next_widget_position(),
                        egui::vec2(lsize.x, top_h),
                    );
                    let line_rect = egui::Rect::from_min_size(
                        egui::pos2(top_rect.left(), top_rect.bottom()),
                        egui::vec2(lsize.x, line_h),
                    );
                    let bot_rect = egui::Rect::from_min_size(
                        egui::pos2(line_rect.left(), line_rect.bottom()),
                        egui::vec2(lsize.x, bot_h),
                    );

                    ui.painter().line_segment(
                        [line_rect.left_center(), line_rect.right_center()],
                        egui::Stroke::new(line_h, egui::Color32::WHITE),
                    );

                    ui.allocate_ui_at_rect(top_rect, |ui| {
                        let tsize = ui.available_size();
                        let side = tsize.x.min(tsize.y) * 0.5;
                        ui.centered_and_justified(|ui| {
                            draw_cover(gui, ui, side);
                        });
                    });

                    ui.allocate_ui_at_rect(bot_rect, |ui| {
                        ui.centered_and_justified(|ui| {
                            draw_bottom(app, ui);
                        });
                    });
                }
            });

            ui.allocate_ui_at_rect(right_rect, |ui| {
                draw_right(app, ui);
            });
        });
}

/// 渲染封面纹理或灰色占位正方形
fn draw_cover(gui: &GuiState, ui: &mut egui::Ui, side: f32) {
    if let Some(url) = &gui.current_cover_url {
        if let Some(img) = gui.cover_cache.get(url) {
            let tex = ui.ctx().load_texture(url, img.clone(), egui::TextureOptions::LINEAR);
            let tex_size = tex.size_vec2();
            let scale = (side / tex_size.x).min(side / tex_size.y);
            let fit_w = tex_size.x * scale;
            let fit_h = tex_size.y * scale;
            ui.add(
                egui::Image::from_texture(egui::load::SizedTexture::from_handle(&tex))
                    .fit_to_exact_size(egui::vec2(fit_w, fit_h)),
            );
            return;
        }
    }

    let rect = egui::Rect::from_center_size(
        ui.available_rect_before_wrap().center(),
        egui::vec2(side, side),
    );
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::from_gray(60));
}

/// 底部信息栏：播放状态/模式/音量/帮助
fn draw_bottom(app: &App, ui: &mut egui::Ui) {
    let font_id = egui::FontId::proportional(16.0);

    let play_state = if app.engine.playing {
        "O Playing"
    } else {
        "X Paused"
    };

    let mode_text = match app.engine.play_mode {
        PlayMode::AlbumList => "Album List",
        PlayMode::AlbumRandom => "Album Random",
        PlayMode::GlobalList => "Global List",
        PlayMode::GlobalRandom => "Global Random",
        PlayMode::Single => "Single",
        PlayMode::LoveList => "Love List",
        PlayMode::LoveRandom => "Love Random",
    };

    let volume_str = format!("{}%", app.engine.volume);

    let mut items: Vec<String> = if app.show_help {
        vec![
            "Ctrl+T  Toggle help".into(),
            "h/l     Prev/Next album".into(),
            "j/k     Prev/Next song".into(),
            "Space   Play selected".into(),
            "x       Pause / Resume".into(),
            "e       Cycle mode".into(),
            "o/p     Volume".into(),
            "s       Toggle love".into(),
            "v       Toggle lyrics".into(),
            "a/d     Seek".into(),
        ]
    } else {
        vec![
            play_state.into(),
            mode_text.into(),
            volume_str,
            "Ctrl+T for help".into(),
        ]
    };

    if !app.show_help {
        if app.engine.buffering {
            if let Some(ref msg) = app.engine.buffering_msg {
                items.insert(1, msg.clone());
            }
        } else if let Some(ref name) = app.engine.current_song_name {
            items.insert(1, name.clone());
        }
    }

    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let n = item_refs.len() as f32;
    let avail = ui.available_rect_before_wrap();
    let row_h = avail.height() / n;

    for (i, text) in item_refs.iter().enumerate() {
        let row_rect = egui::Rect::from_min_size(
            egui::pos2(avail.left(), avail.top() + i as f32 * row_h),
            egui::vec2(avail.width(), row_h),
        );
        ui.allocate_ui_at_rect(row_rect, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new(*text)
                        .font(font_id.clone())
                        .color(egui::Color32::WHITE),
                );
            });
        });
    }
}

/// 右侧分区：歌词/收藏/专辑+歌曲
fn draw_right(app: &App, ui: &mut egui::Ui) {
    if app.show_lyrics {
        draw_lyrics(app, ui);
        return;
    }

    let is_love = matches!(
        app.engine.play_mode,
        PlayMode::LoveList | PlayMode::LoveRandom
    );

    if is_love {
        draw_loved_view(app, ui);
        return;
    }

    if app.engine.albums.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("Loading...").color(egui::Color32::WHITE));
        });
        return;
    }

    let size = ui.available_size();
    let top_h = size.y * 0.2;
    let line_h = 2.0;
    let bot_h = size.y - top_h - line_h;

    let top_rect = egui::Rect::from_min_size(
        ui.next_widget_position(),
        egui::vec2(size.x, top_h),
    );
    let line_rect = egui::Rect::from_min_size(
        egui::pos2(top_rect.left(), top_rect.bottom()),
        egui::vec2(size.x, line_h),
    );
    let bot_rect = egui::Rect::from_min_size(
        egui::pos2(line_rect.left(), line_rect.bottom()),
        egui::vec2(size.x, bot_h),
    );

    ui.painter().line_segment(
        [line_rect.left_center(), line_rect.right_center()],
        egui::Stroke::new(line_h, egui::Color32::WHITE),
    );

    ui.allocate_ui_at_rect(top_rect, |ui| {
        ui.vertical_centered(|ui| {
            if let Some(ref name) = app.engine.album_name {
                ui.label(
                    egui::RichText::new(name.clone())
                        .font(egui::FontId::proportional(24.0))
                        .color(egui::Color32::WHITE),
                );
            }
            if let Some(ref artist) = app.engine.album_artist {
                ui.label(
                    egui::RichText::new(artist.clone())
                        .font(egui::FontId::proportional(17.0))
                        .color(egui::Color32::LIGHT_GRAY),
                );
            }
            if app.engine.album_total > 1 {
                ui.label(
                    egui::RichText::new(format!(
                        "[{}/{}]",
                        app.engine.album_index + 1,
                        app.engine.album_total
                    ))
                    .font(egui::FontId::proportional(14.0))
                    .color(egui::Color32::GRAY),
                );
            }
        });
    });

    ui.allocate_ui_at_rect(bot_rect, |ui| {
        if app.engine.songs_loaded {
            ui.vertical_centered(|ui| {
                let total = app.engine.songs.len();
                let max_disp = (bot_h as usize / 20).max(5);
                let start = if total <= max_disp {
                    0
                } else {
                    let half = max_disp / 2;
                    app.selected_song
                        .saturating_sub(half)
                        .min(total.saturating_sub(max_disp))
                };
                let end = (start + max_disp).min(total);

                for i in start..end {
                    let song = &app.engine.songs[i];
                    let prefix = if i == app.selected_song { "> " } else { "  " };
                    let is_playing = app
                        .engine
                        .current_song_cid
                        .as_ref()
                        .map_or(false, |cid| cid == &song.cid);

                    let color = if is_playing {
                        egui::Color32::from_rgb(0, 200, 200)
                    } else if i == app.selected_song {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::GRAY
                    };

                    let text = format!(
                        "{}{} - {}",
                        prefix,
                        song.name,
                        song.artistes.join(", ")
                    );
                    if app.engine.is_loved(&song.cid) {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(text)
                                    .font(egui::FontId::proportional(16.0))
                                    .color(color),
                            );
                            ui.label(
                                egui::RichText::new(" *")
                                    .font(egui::FontId::proportional(16.0))
                                    .color(egui::Color32::RED),
                            );
                        });
                    } else {
                        ui.label(
                            egui::RichText::new(text)
                                .font(egui::FontId::proportional(16.0))
                                .color(color),
                        );
                    }
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("Loading songs...").color(egui::Color32::GRAY),
                );
            });
        }
    });
}

/// 歌词视图：当前行放大加粗（20pt）
fn draw_lyrics(app: &App, ui: &mut egui::Ui) {
    if app.engine.lyrics.is_empty() {
        let msg = if app.engine.current_song_name.is_some() {
            "No lyrics available"
        } else {
            "No song playing"
        };
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new(msg).color(egui::Color32::DARK_GRAY));
        });
        return;
    }

    let max_disp = 20usize;
    let total = app.engine.lyrics.len();
    let half = max_disp / 2;
    let start = if total <= max_disp {
        0
    } else {
        app.engine
            .lyric_index
            .saturating_sub(half)
            .min(total.saturating_sub(max_disp))
    };
    let end = (start + max_disp).min(total);

    ui.vertical_centered(|ui| {
        for i in start..end {
            let (_, ref text) = app.engine.lyrics[i];
            let is_current = i == app.engine.lyric_index;

            if is_current {
                ui.label(
                    egui::RichText::new(format!("> {}", text))
                        .font(egui::FontId::proportional(20.0))
                        .color(egui::Color32::from_rgb(0, 200, 200))
                        .strong(),
                );
            } else {
                ui.label(
                    egui::RichText::new(format!("  {}", text))
                        .font(egui::FontId::proportional(14.0))
                        .color(egui::Color32::WHITE),
                );
            }
        }
    });
}

/// 收藏歌曲视图
fn draw_loved_view(app: &App, ui: &mut egui::Ui) {
    let size = ui.available_size();
    let top_h = size.y * 0.2;
    let line_h = 2.0;
    let bot_h = size.y - top_h - line_h;

    let top_rect = egui::Rect::from_min_size(
        ui.next_widget_position(),
        egui::vec2(size.x, top_h),
    );
    let line_rect = egui::Rect::from_min_size(
        egui::pos2(top_rect.left(), top_rect.bottom()),
        egui::vec2(size.x, line_h),
    );
    let bot_rect = egui::Rect::from_min_size(
        egui::pos2(line_rect.left(), line_rect.bottom()),
        egui::vec2(size.x, bot_h),
    );

    ui.painter().line_segment(
        [line_rect.left_center(), line_rect.right_center()],
        egui::Stroke::new(line_h, egui::Color32::WHITE),
    );

    ui.allocate_ui_at_rect(top_rect, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(format!("Loved Songs ({})", app.engine.loved_list.len()))
                    .font(egui::FontId::proportional(20.0))
                    .color(egui::Color32::from_rgb(0, 200, 200)),
            );
        });
    });

    ui.allocate_ui_at_rect(bot_rect, |ui| {
        if app.engine.loved_list.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No loved songs. Press S to love.")
                        .color(egui::Color32::DARK_GRAY),
                );
            });
            return;
        }

        ui.vertical_centered(|ui| {
            let total = app.engine.loved_list.len();
            let max_disp = (bot_h as usize / 20).max(5);
            let start = if total <= max_disp {
                0
            } else {
                let half = max_disp / 2;
                app.selected_song
                    .saturating_sub(half)
                    .min(total.saturating_sub(max_disp))
            };
            let end = (start + max_disp).min(total);

            for i in start..end {
                let entry = &app.engine.loved_list[i];
                let prefix = if i == app.selected_song { "> " } else { "  " };
                let is_playing = app
                    .engine
                    .current_song_cid
                    .as_ref()
                    .map_or(false, |cid| cid == &entry.cid);

                let color = if is_playing {
                    egui::Color32::from_rgb(0, 200, 200)
                } else if i == app.selected_song {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::GRAY
                };

                let text = format!(
                    "{}{} - {}",
                    prefix,
                    entry.name,
                    entry.artists.join(", ")
                );
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(text)
                            .font(egui::FontId::proportional(16.0))
                            .color(color),
                    );
                    ui.label(
                        egui::RichText::new(" *")
                            .font(egui::FontId::proportional(16.0))
                            .color(egui::Color32::RED),
                    );
                });
            }
        });
    });
}
