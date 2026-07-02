use std::collections::HashMap;
use std::io::Read;

use eframe::egui;
use monster_player::kernel::PlayMode;

use super::app::App;
use super::settings::draw_settings;
use super::theme::{ThemeColors};

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

/// 主帧函数
pub fn update(app: &mut App, gui: &mut GuiState, ctx: &egui::Context, frame: &mut eframe::Frame) {
    // 播放时持续重绘
    if app.engine.playing {
        ctx.request_repaint();
    }

    app.update();
    gui.check_cover(app);

    let colors = app.colors();

    // 自定义标题栏
    draw_title_bar(ctx, frame, colors);

    // 设置弹窗渲染（在最顶层）
    draw_settings(ctx, app, colors);

    // 搜索弹窗渲染（顶部下拉）
    draw_search_popup(ctx, app, colors);

    // 键盘处理
    handle_input(app, ctx);

    // 鼠标处理
    handle_mouse(app, ctx);

    // 主布局
    render_main(app, gui, ctx, colors);
}

fn handle_input(app: &mut App, ctx: &egui::Context) {
    // 设置弹窗打开时拦截按键
    if app.settings.show {
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, .. } = event {
                    match key {
                        egui::Key::Escape => { app.settings.toggle(); return; }
                        egui::Key::K | egui::Key::ArrowUp => {
                            if app.settings.focus_left {
                                app.settings.move_up();
                            } else {
                                match app.settings.tab {
                                    super::settings::SettingsTab::Themes => app.cycle_theme(),
                                    super::settings::SettingsTab::Extra => app.toggle_download(),
                                    _ => {}
                                }
                            }
                            return;
                        }
                        egui::Key::J | egui::Key::ArrowDown => {
                            if app.settings.focus_left {
                                app.settings.move_down();
                            } else {
                                match app.settings.tab {
                                    super::settings::SettingsTab::Themes => app.cycle_theme(),
                                    super::settings::SettingsTab::Extra => app.toggle_download(),
                                    _ => {}
                                }
                            }
                            return;
                        }
                        egui::Key::Tab => { app.settings.tab_switch(); return; }
                        egui::Key::H | egui::Key::ArrowLeft => {
                            match app.settings.tab {
                                super::settings::SettingsTab::Themes => app.cycle_theme(),
                                super::settings::SettingsTab::Extra => app.toggle_download(),
                                _ => {}
                            }
                            return;
                        }
                        egui::Key::L | egui::Key::ArrowRight => {
                            match app.settings.tab {
                                super::settings::SettingsTab::Themes => {
                                    app.cycle_theme();
                                }
                                super::settings::SettingsTab::Extra => {
                                    app.toggle_download();
                                }
                                _ => {}
                            }
                            return;
                        }
                        _ => {}
                    }
                }
            }
        });
        return;
    }

    if app.search_mode {
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    for c in text.chars() {
                        if c.is_control() { continue; }
                        app.search_input(c);
                    }
                }
                if let egui::Event::Ime(ime) = event {
                    if let egui::ImeEvent::Commit(text) = ime {
                        app.search_input_str(text);
                    }
                }
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    match key {
                        egui::Key::Escape => { app.exit_search(); return; }
                        egui::Key::Enter => { app.search_confirm(); return; }
                        egui::Key::Backspace => { app.search_backspace(); return; }
                        egui::Key::K | egui::Key::ArrowUp if *modifiers == egui::Modifiers::NONE => { app.search_prev(); return; }
                        egui::Key::J | egui::Key::ArrowDown if *modifiers == egui::Modifiers::NONE => { app.search_next(); return; }
                        _ => {}
                    }
                }
            }
        });
    } else {
        if ctx.input(|i| i.key_pressed(egui::Key::Slash)) {
            app.enter_search();
        }
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

        let pressed_up = ctx.input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp));
        let pressed_down = ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown));
        let pressed_left = ctx.input(|i| i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft));
        let pressed_right = ctx.input(|i| i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight));

        if pressed_up { app.prev_song(); }
        if pressed_down { app.next_song(); }
        if pressed_left { app.prev_album(); }
        if pressed_right { app.next_album(); }

        if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.ctrl) {
            app.settings.toggle();
        }
    }
}

fn handle_mouse(app: &mut App, ctx: &egui::Context) {
    if app.settings.show || app.search_mode {
        return;
    }

    let screen = ctx.screen_rect();

    ctx.input(|i| {
        if i.pointer.any_click() {
            let pos = i.pointer.interact_pos().unwrap_or_default();

            // 搜索图标区域（右上角）
            let search_icon_rect = egui::Rect::from_min_size(
                egui::pos2(screen.right() - 32.0, screen.top() + 35.0),
                egui::vec2(24.0, 24.0),
            );
            if search_icon_rect.contains(pos) {
                app.enter_search();
                return;
            }

            // 右键 → 收藏
            if i.pointer.secondary_pressed() {
                app.toggle_love();
                return;
            }
        }

        // 右侧面板滚轮切歌
        let pos = i.pointer.interact_pos().unwrap_or_default();
        if pos.x > screen.width() * 0.35 {
            for event in &i.events {
                match event {
                    egui::Event::MouseWheel { unit: _, delta, modifiers: _ } if delta.y > 0.0 => app.next_song(),
                    egui::Event::MouseWheel { unit: _, delta, modifiers: _ } if delta.y < 0.0 => app.prev_song(),
                    _ => {}
                }
            }
        }
    });
}

/// 自定义标题栏：拖拽移动 + 关闭按钮
fn draw_title_bar(ctx: &egui::Context, _frame: &mut eframe::Frame, colors: ThemeColors) {
    let screen = ctx.screen_rect();
    let bar_h = 24.0;
    let bar_rect = egui::Rect::from_min_size(screen.min, egui::vec2(screen.width(), bar_h));

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("title_bar_layer"),
    ));

    painter.rect_filled(bar_rect, egui::CornerRadius::same(0), colors.card_bg);
    painter.rect_stroke(
        egui::Rect::from_min_size(bar_rect.min, egui::vec2(bar_rect.width(), bar_h)),
        egui::CornerRadius::same(0),
        egui::Stroke::new(1.0, colors.card_border),
        egui::StrokeKind::Inside,
    );

    // 标题文字
    painter.text(
        bar_rect.center(),
        egui::Align2::CENTER_CENTER,
        "monster-player",
        egui::FontId::proportional(13.0),
        colors.text_secondary,
    );

    // 关闭按钮
    let btn_r = 8.0;
    let close_center = egui::pos2(screen.right() - 16.0, bar_rect.center().y);
    let close_rect = egui::Rect::from_center_size(close_center, egui::vec2(btn_r * 2.0, btn_r * 2.0));
    let stroke = egui::Stroke::new(1.5, colors.text_secondary);
    painter.line_segment(
        [egui::pos2(close_center.x - 4.0, close_center.y - 4.0),
         egui::pos2(close_center.x + 4.0, close_center.y + 4.0)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(close_center.x + 4.0, close_center.y - 4.0),
         egui::pos2(close_center.x - 4.0, close_center.y + 4.0)],
        stroke,
    );

    // 拖拽 + 关闭交互（通过 ui 层处理）
    let ui = &ctx.layer_painter(egui::LayerId::new(egui::Order::Debug, egui::Id::new("title_interact")));
    let _ = ui;
    ctx.input(|i| {
        let pos = i.pointer.interact_pos().unwrap_or_default();
        if i.pointer.button_pressed(egui::PointerButton::Primary) {
            if close_rect.contains(pos) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if bar_rect.contains(pos) {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
        }
    });
}

fn render_main(app: &mut App, gui: &mut GuiState, ctx: &egui::Context, colors: ThemeColors) {
    let bg = colors.bg_fill;
    let border = colors.border;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(bg)
                .stroke(egui::Stroke::new(3.0, border))
                .outer_margin(egui::Margin { left: 11, right: 11, top: 35, bottom: 11 })
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
                egui::Stroke::new(line_w, colors.border),
            );

            // 搜索图标（右上角）
            draw_search_icon(ui, colors, size);

            // 左侧面板
            ui.allocate_ui_at_rect(left_rect, |ui| {
                let lsize = ui.available_size();
                if shrink {
                    ui.centered_and_justified(|ui| {
                        draw_bottom(app, colors, ui);
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
                        egui::Stroke::new(line_h, colors.border),
                    );

                    // 封面（可点击暂停）
                    ui.allocate_ui_at_rect(top_rect, |ui| {
                        let tsize = ui.available_size();
                        let side = tsize.x.min(tsize.y) * 0.5;
                        ui.centered_and_justified(|ui| {
                            draw_cover(gui, ui, side);
                        });
                        let resp = ui.interact(top_rect, egui::Id::new("cover_click"), egui::Sense::click());
                        if resp.clicked() {
                            app.toggle_pause();
                        }
                    });

                    // 底部状态：进度条 + 信息 + 播放按钮
                    ui.allocate_ui_at_rect(bot_rect, |ui| {
                        ui.centered_and_justified(|ui| {
                            draw_bottom(app, colors, ui);
                        });
                    });
                }
            });

            // 右侧面板
            ui.allocate_ui_at_rect(right_rect, |ui| {
                draw_right(app, colors, ui);
            });
        });
}

/// 搜索弹窗：顶部下拉（Spotlight 风格），不覆盖整个界面
fn draw_search_popup(ctx: &egui::Context, app: &mut App, colors: ThemeColors) {
    if !app.search_mode {
        return;
    }
    let screen = ctx.screen_rect();
    let popup_w = (screen.width() * 0.55).min(420.0);
    let max_results = 6.min(app.search_results.len());
    let popup_h = 36.0 + max_results as f32 * 22.0 + 12.0;
    let popup_rect = egui::Rect::from_center_size(
        egui::pos2(screen.center().x, screen.top() + popup_h * 0.5 + 4.0),
        egui::vec2(popup_w, popup_h),
    );

    egui::Area::new(egui::Id::new("search_popup"))
        .fixed_pos(popup_rect.min)
        .interactable(true)
        .show(ctx, |ui| {
            ui.painter().rect_filled(popup_rect, egui::CornerRadius::same(8), colors.search_bg);
            ui.painter().rect_stroke(popup_rect, egui::CornerRadius::same(8),
                egui::Stroke::new(1.5, colors.border), egui::StrokeKind::Inside);

            let inner = popup_rect.shrink(8.0);
            // 输入栏
            let input_rect = egui::Rect::from_min_size(inner.min, egui::vec2(inner.width(), 28.0));
            let input_inner = input_rect.shrink(4.0);
            let query_display = if app.search_confirmed {
                format!("{} ✓", app.search_query)
            } else if app.search_query.is_empty() {
                "Type to search...".to_string()
            } else {
                format!("{}█", app.search_query)
            };
            ui.painter().text(input_inner.left_center(), egui::Align2::LEFT_CENTER,
                &query_display, egui::FontId::proportional(14.0), colors.search_text);

            // 分割线
            let div_y = input_rect.bottom();
            ui.painter().line_segment(
                [egui::pos2(inner.left(), div_y), egui::pos2(inner.right(), div_y)],
                egui::Stroke::new(1.0, colors.border),
            );

            // 结果列表
            if !app.search_results.is_empty() {
                for i in 0..max_results {
                    let song = &app.search_results[i];
                    let is_selected = i == app.search_index;
                    let prefix = if is_selected { "> " } else { "  " };
                    let color = if is_selected { colors.accent } else { colors.text_secondary };
                    let text = format!("{}{} - {}", prefix, song.name, song.artists.join(", "));
                    let item_y = div_y + 4.0 + i as f32 * 22.0;
                    let item_rect = egui::Rect::from_min_size(
                        egui::pos2(inner.left(), item_y), egui::vec2(inner.width(), 22.0),
                    );
                    ui.painter().text(item_rect.left_center(), egui::Align2::LEFT_CENTER,
                        &text, egui::FontId::proportional(12.0), color);

                    let resp = ui.interact(item_rect, egui::Id::new(("sp", i)), egui::Sense::click());
                    if resp.clicked() { app.search_index = i; }
                    if resp.double_clicked() { app.search_index = i; app.search_confirm(); }
                }
            } else if !app.search_query.is_empty() {
                let ny = div_y + 8.0;
                ui.painter().text(egui::pos2(inner.center().x, ny), egui::Align2::CENTER_TOP,
                    "No results", egui::FontId::proportional(12.0), colors.text_secondary);
            }
        });
}

fn draw_search_icon(ui: &mut egui::Ui, colors: ThemeColors, size: egui::Vec2) {
    let cx = size.x - 20.0;
    let cy = 28.0;
    let r = 6.0;

    let painter = ui.painter();
    let center = egui::pos2(
        ui.clip_rect().left() + cx,
        ui.clip_rect().top() + cy,
    );
    let stroke = egui::Stroke::new(2.0, colors.search_icon);

    // 圆形
    painter.circle_stroke(center, r, stroke);
    // 斜线（手柄）
    let handle_start = egui::pos2(center.x + r * 0.7, center.y + r * 0.7);
    let handle_end = egui::pos2(center.x + r * 1.4, center.y + r * 1.4);
    painter.line_segment([handle_start, handle_end], stroke);
}

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
    ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::from_gray(60));
}

fn draw_bottom(app: &mut App, colors: ThemeColors, ui: &mut egui::Ui) {
    let big_font = egui::FontId::proportional(15.0);
    let small_font = egui::FontId::proportional(13.0);
    let avail = ui.available_rect_before_wrap();
    let card_pad = 4.0;

    // 卡片背景
    let card_rect = avail.expand(card_pad);
    ui.painter().rect_filled(card_rect, egui::CornerRadius::same(6), colors.card_bg);
    ui.painter().rect_stroke(card_rect, egui::CornerRadius::same(6),
        egui::Stroke::new(1.0, colors.card_border), egui::StrokeKind::Inside);

    let inner = card_rect.shrink(card_pad);

    let play_state = if app.engine.playing { "\u{25b6} 播放中" } else { "O 已暂停" };
    let mode_text = match app.engine.play_mode {
        PlayMode::AlbumList => "专辑列表",
        PlayMode::AlbumRandom => "专辑随机",
        PlayMode::GlobalList => "全局列表",
        PlayMode::GlobalRandom => "全局随机",
        PlayMode::Single => "单曲循环",
        PlayMode::LoveList => "收藏列表",
        PlayMode::LoveRandom => "收藏随机",
    };
    let volume_str = format!("音量: {}%", app.engine.volume);

    if app.show_help {
        let help_items = [
            ("Ctrl+T", "帮助"), ("h/l", "上/下专辑"), ("j/k", "上/下歌曲"),
            ("Space", "播放选中"), ("x", "暂停/恢复"), ("e", "切换模式"),
            ("o/p", "音量"), ("s", "收藏"), ("v", "歌词"),
            ("a/d", "快进/退"), ("/", "搜索"), ("Shift+A/D", "Shift切歌"),
        ];
        let row_h = 16.0;
        for (i, (key, desc)) in help_items.iter().enumerate() {
            let y = inner.top() + i as f32 * row_h;
            ui.painter().text(egui::pos2(inner.left() + inner.width() * 0.35, y + row_h * 0.5),
                egui::Align2::RIGHT_CENTER, key, small_font.clone(), colors.accent);
            ui.painter().text(egui::pos2(inner.left() + inner.width() * 0.38, y + row_h * 0.5),
                egui::Align2::LEFT_CENTER, desc, small_font.clone(), colors.text_primary);
        }
        return;
    }

    // 进度条行
    let bar_h = 20.0;
    let bar_y = inner.top() + 4.0;
    if let Some(pct) = app.engine.progress {
        let bar_pad = 8.0;
        let bar_left = inner.left() + bar_pad;
        let bar_right = inner.right() - bar_pad;
        let bar_width = bar_right - bar_left;
        let elapsed = app.engine.elapsed_secs();
        let time_text = if let Some(dur) = app.engine.duration_secs() {
            format!("{:02}:{:02} / {:02}:{:02}", elapsed as u64/60, elapsed as u64%60, dur as u64/60, dur as u64%60)
        } else {
            format!("{:02}:{:02}", elapsed as u64/60, elapsed as u64%60)
        };
        ui.painter().text(egui::pos2(inner.center().x, bar_y),
            egui::Align2::CENTER_TOP, &time_text, egui::FontId::proportional(11.0), colors.text_secondary);

        let line_y = bar_y + 14.0;
        let line_start = egui::pos2(bar_left, line_y);
        let line_end = egui::pos2(bar_right, line_y);
        ui.painter().line_segment([line_start, line_end], egui::Stroke::new(3.0, colors.progress_bg));
        let filled_right = bar_left + bar_width * pct as f32;
        ui.painter().line_segment([line_start, egui::pos2(filled_right, line_y)], egui::Stroke::new(3.0, colors.progress_bar));
        ui.painter().circle_filled(egui::pos2(filled_right, line_y), 3.5, colors.progress_bar);

        let drag_rect = egui::Rect::from_min_size(egui::pos2(bar_left - 6.0, line_y - 8.0), egui::vec2(bar_width + 12.0, 16.0));
        let resp = ui.interact(drag_rect, egui::Id::new("progress_bar"), egui::Sense::click_and_drag());
        if resp.dragged() || resp.clicked() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let new_pct = ((pos.x - bar_left) / bar_width).clamp(0.0, 1.0) as f64;
                app.seek_to_progress(new_pct);
            }
        }
    }

    // 状态行：两列布局（左侧状态+歌曲，右侧模式+音量）
    let status_y = bar_y + bar_h + 4.0;
    let col_w = inner.width() * 0.5;

    let song_name = if app.engine.buffering {
        app.engine.buffering_msg.clone().unwrap_or_default()
    } else {
        app.engine.current_song_name.clone().unwrap_or_default()
    };
    let short_name = if song_name.len() > 15 {
        let end = song_name.char_indices()
            .take(12)
            .last()
            .map_or(0, |(i, c)| i + c.len_utf8());
        format!("{}...", &song_name[..end])
    } else {
        song_name.clone()
    };

    ui.painter().text(egui::pos2(inner.left() + 4.0, status_y + 8.0),
        egui::Align2::LEFT_CENTER, &play_state, big_font.clone(), colors.accent);
    if !short_name.is_empty() {
        ui.painter().text(egui::pos2(inner.left() + 4.0, status_y + 28.0),
            egui::Align2::LEFT_CENTER, &short_name, small_font.clone(), colors.text_primary);
    }
    ui.painter().text(egui::pos2(inner.left() + col_w, status_y + 8.0),
        egui::Align2::LEFT_CENTER, mode_text, big_font.clone(), colors.text_primary);
    // 点击模式文字切换模式
    let mode_rect = egui::Rect::from_min_size(
        egui::pos2(inner.left() + col_w, status_y),
        egui::vec2(col_w, 22.0),
    );
    if ui.interact(mode_rect, egui::Id::new("mode_click"), egui::Sense::click()).clicked() {
        app.cycle_mode();
    }
    ui.painter().text(egui::pos2(inner.left() + col_w, status_y + 28.0),
        egui::Align2::LEFT_CENTER, &volume_str, small_font.clone(), colors.text_secondary);

    // 播放按钮行（使用 egui 原生 Button）
    let btn_y = status_y + 52.0;
    let btn_rect = egui::Rect::from_min_size(egui::pos2(inner.left(), btn_y),
        egui::vec2(inner.width(), 28.0));
    ui.allocate_ui_at_rect(btn_rect, |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(4.0);
            if ui.button(egui::RichText::new("<").font(small_font.clone()).color(colors.accent)).clicked() {
                app.play_prev();
            }
            let pause_label = if app.engine.playing { "||" } else { ">" };
            if ui.button(egui::RichText::new(pause_label).font(small_font.clone()).color(colors.accent)).clicked() {
                app.toggle_pause();
            }
            if ui.button(egui::RichText::new(">").font(small_font.clone()).color(colors.accent)).clicked() {
                app.play_next();
            }
            ui.add_space(4.0);
        });
    });
}

fn draw_right(app: &App, colors: ThemeColors, ui: &mut egui::Ui) {
    // 卡片背景
    let avail = ui.available_rect_before_wrap();
    let card = avail.expand(4.0);
    ui.painter().rect_filled(card, egui::CornerRadius::same(6), colors.card_bg);
    ui.painter().rect_stroke(card, egui::CornerRadius::same(6),
        egui::Stroke::new(1.0, colors.card_border), egui::StrokeKind::Inside);

    if app.show_lyrics {
        draw_lyrics(app, colors, ui);
        return;
    }
    if matches!(app.engine.play_mode, PlayMode::LoveList | PlayMode::LoveRandom) {
        draw_loved_view(app, colors, ui);
        return;
    }
    if app.engine.is_global_mode() {
        draw_global_view(app, colors, ui);
        return;
    }
    if app.engine.albums.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("Loading...").color(colors.text_primary));
        });
        return;
    }

    let size = ui.available_size();
    let top_h = size.y * 0.2;
    let line_h = 2.0;
    let bot_h = size.y - top_h - line_h;

    let top_rect = egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(size.x, top_h));
    let line_rect = egui::Rect::from_min_size(egui::pos2(top_rect.left(), top_rect.bottom()), egui::vec2(size.x, line_h));
    let bot_rect = egui::Rect::from_min_size(egui::pos2(line_rect.left(), line_rect.bottom()), egui::vec2(size.x, bot_h));

    ui.painter().line_segment(
        [line_rect.left_center(), line_rect.right_center()],
        egui::Stroke::new(line_h, colors.border),
    );

    ui.allocate_ui_at_rect(top_rect, |ui| {
        ui.vertical_centered(|ui| {
            if let Some(ref name) = app.engine.album_name {
                ui.label(egui::RichText::new(name.clone()).font(egui::FontId::proportional(24.0)).color(colors.text_primary));
            }
            if let Some(ref artist) = app.engine.album_artist {
                ui.label(egui::RichText::new(artist.clone()).font(egui::FontId::proportional(12.0)).color(colors.text_secondary));
            }
            if let Some(ref intro) = app.engine.album_intro {
                ui.label(egui::RichText::new(intro.clone()).font(egui::FontId::proportional(12.0)).color(colors.text_secondary));
            }
            if let Some(ref info) = app.engine.song_info {
                ui.label(egui::RichText::new(info.clone()).font(egui::FontId::proportional(12.0)).color(colors.text_primary));
            }
            if app.engine.album_total > 1 {
                ui.label(egui::RichText::new(format!("[{}/{}]", app.engine.album_index + 1, app.engine.album_total)).font(egui::FontId::proportional(14.0)).color(colors.text_secondary));
            }
        });
    });

    ui.allocate_ui_at_rect(bot_rect, |ui| {
        if app.engine.songs_loaded {
            ui.vertical_centered(|ui| {
                let total = app.engine.songs.len();
                let max_disp = (bot_h as usize / 20).max(5);
                let start = if total <= max_disp { 0 } else {
                    let half = max_disp / 2;
                    app.selected_song.saturating_sub(half).min(total.saturating_sub(max_disp))
                };
                let end = (start + max_disp).min(total);

                for i in start..end {
                    let song = &app.engine.songs[i];
                    let prefix = if i == app.selected_song { "> " } else { "  " };
                    let is_playing = app.engine.current_song_cid.as_ref().map_or(false, |cid| cid == &song.cid);
                    let color = if is_playing { colors.accent } else if i == app.selected_song { colors.cursor } else { colors.text_secondary };

                    let text = format!("{}{} - {}", prefix, song.name, song.artistes.join(", "));
                    if app.engine.is_loved(&song.cid) {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(text).font(egui::FontId::proportional(16.0)).color(color));
                            ui.label(egui::RichText::new(" *").font(egui::FontId::proportional(16.0)).color(colors.loved));
                        });
                    } else {
                        ui.label(egui::RichText::new(text).font(egui::FontId::proportional(16.0)).color(color));
                    }
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("Loading songs...").color(colors.text_secondary));
            });
        }
    });
}

fn draw_lyrics(app: &App, colors: ThemeColors, ui: &mut egui::Ui) {
    if app.engine.lyrics.is_empty() {
        let msg = if app.engine.current_song_name.is_some() { "No lyrics available" } else { "No song playing" };
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new(msg).color(colors.text_secondary));
        });
        return;
    }
    let max_disp = 20usize;
    let total = app.engine.lyrics.len();
    let half = max_disp / 2;
    let start = if total <= max_disp { 0 } else {
        app.engine.lyric_index.saturating_sub(half).min(total.saturating_sub(max_disp))
    };
    let end = (start + max_disp).min(total);

    ui.vertical_centered(|ui| {
        let avail_h = ui.available_size().y;
        ui.add_space(avail_h * 0.15);
        for i in start..end {
            let (_, ref text) = app.engine.lyrics[i];
            let is_current = i == app.engine.lyric_index;
            if is_current {
                match app.lyric_animation {
                    super::theme::LyricAnimation::FontGrow => {
                        ui.label(egui::RichText::new(format!("> {}", text)).font(egui::FontId::proportional(20.0)).color(colors.accent).strong());
                    }
                    super::theme::LyricAnimation::ColorChange => {
                        ui.label(egui::RichText::new(format!("> {}", text)).font(egui::FontId::proportional(14.0)).color(colors.accent).strong());
                    }
                }
            } else {
                ui.label(egui::RichText::new(format!("  {}", text)).font(egui::FontId::proportional(14.0)).color(colors.text_primary));
            }
        }
    });
}

fn draw_global_view(app: &App, colors: ThemeColors, ui: &mut egui::Ui) {
    let size = ui.available_size();
    let top_h = size.y * 0.2;
    let line_h = 2.0;
    let bot_h = size.y - top_h - line_h;

    let top_rect = egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(size.x, top_h));
    let line_rect = egui::Rect::from_min_size(egui::pos2(top_rect.left(), top_rect.bottom()), egui::vec2(size.x, line_h));
    let bot_rect = egui::Rect::from_min_size(egui::pos2(line_rect.left(), line_rect.bottom()), egui::vec2(size.x, bot_h));

    ui.painter().line_segment(
        [line_rect.left_center(), line_rect.right_center()],
        egui::Stroke::new(line_h, colors.border),
    );

    ui.allocate_ui_at_rect(top_rect, |ui| {
        ui.vertical_centered(|ui| {
            let mode_name = match app.engine.play_mode {
                PlayMode::GlobalList => "Global List",
                PlayMode::GlobalRandom => "Global Random",
                _ => "Global",
            };
            ui.label(egui::RichText::new(format!("{} ({})", mode_name, app.engine.global_playlist.len())).font(egui::FontId::proportional(20.0)).color(colors.accent));
        });
    });

    ui.allocate_ui_at_rect(bot_rect, |ui| {
        if app.engine.global_playlist.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("Loading global playlist...").color(colors.text_secondary));
            });
            return;
        }
        ui.vertical_centered(|ui| {
            let total = app.engine.global_playlist.len();
            let max_disp = (bot_h as usize / 20).max(5);
            let start = if total <= max_disp { 0 } else {
                let half = max_disp / 2;
                app.engine.global_index.saturating_sub(half).min(total.saturating_sub(max_disp))
            };
            let end = (start + max_disp).min(total);
            for i in start..end {
                let song = &app.engine.global_playlist[i];
                let prefix = if i == app.engine.global_index { "> " } else { "  " };
                let is_playing = app.engine.current_song_cid.as_ref().map_or(false, |cid| cid == &song.cid);
                let color = if is_playing { colors.accent } else if i == app.engine.global_index { colors.cursor } else { colors.text_secondary };
                let text = format!("{}{} - {}", prefix, song.name, song.artists.join(", "));
                if app.engine.is_loved(&song.cid) {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(text).font(egui::FontId::proportional(16.0)).color(color));
                        ui.label(egui::RichText::new(" *").font(egui::FontId::proportional(16.0)).color(colors.loved));
                    });
                } else {
                    ui.label(egui::RichText::new(text).font(egui::FontId::proportional(16.0)).color(color));
                }
            }
        });
    });
}

fn draw_loved_view(app: &App, colors: ThemeColors, ui: &mut egui::Ui) {
    let size = ui.available_size();
    let top_h = size.y * 0.2;
    let line_h = 2.0;
    let bot_h = size.y - top_h - line_h;

    let top_rect = egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(size.x, top_h));
    let line_rect = egui::Rect::from_min_size(egui::pos2(top_rect.left(), top_rect.bottom()), egui::vec2(size.x, line_h));
    let bot_rect = egui::Rect::from_min_size(egui::pos2(line_rect.left(), line_rect.bottom()), egui::vec2(size.x, bot_h));

    ui.painter().line_segment(
        [line_rect.left_center(), line_rect.right_center()],
        egui::Stroke::new(line_h, colors.border),
    );

    ui.allocate_ui_at_rect(top_rect, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(format!("Loved Songs ({})", app.engine.loved_list.len())).font(egui::FontId::proportional(20.0)).color(colors.accent));
        });
    });

    ui.allocate_ui_at_rect(bot_rect, |ui| {
        if app.engine.loved_list.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("No loved songs. Press S to love.").color(colors.text_secondary));
            });
            return;
        }
        ui.vertical_centered(|ui| {
            let total = app.engine.loved_list.len();
            let max_disp = (bot_h as usize / 20).max(5);
            let start = if total <= max_disp { 0 } else {
                let half = max_disp / 2;
                app.selected_song.saturating_sub(half).min(total.saturating_sub(max_disp))
            };
            let end = (start + max_disp).min(total);
            for i in start..end {
                let entry = &app.engine.loved_list[i];
                let prefix = if i == app.selected_song { "> " } else { "  " };
                let is_playing = app.engine.current_song_cid.as_ref().map_or(false, |cid| cid == &entry.cid);
                let color = if is_playing { colors.accent } else if i == app.selected_song { colors.cursor } else { colors.text_secondary };
                let text = format!("{}{} - {}", prefix, entry.name, entry.artists.join(", "));
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(text).font(egui::FontId::proportional(16.0)).color(color));
                    ui.label(egui::RichText::new(" *").font(egui::FontId::proportional(16.0)).color(colors.loved));
                });
            }
        });
    });
}
