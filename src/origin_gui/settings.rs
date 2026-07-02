use eframe::egui;

use super::app::App;
use super::theme::{LyricAnimation, ThemeColors, ThemeName};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    About,
    Shortcuts,
    Themes,
    Extra,
}

impl SettingsTab {
    pub fn all() -> &'static [SettingsTab] {
        &[
            SettingsTab::About,
            SettingsTab::Shortcuts,
            SettingsTab::Themes,
            SettingsTab::Extra,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingsTab::About => "项目简介",
            SettingsTab::Shortcuts => "快捷键信息",
            SettingsTab::Themes => "主题设置",
            SettingsTab::Extra => "额外功能",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            SettingsTab::About => 0,
            SettingsTab::Shortcuts => 1,
            SettingsTab::Themes => 2,
            SettingsTab::Extra => 3,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => SettingsTab::About,
            1 => SettingsTab::Shortcuts,
            2 => SettingsTab::Themes,
            3 => SettingsTab::Extra,
            _ => SettingsTab::About,
        }
    }
}

pub struct SettingsState {
    pub show: bool,
    pub tab: SettingsTab,
    pub focus_left: bool,
    pub selected_index: usize,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            show: false,
            tab: SettingsTab::About,
            focus_left: true,
            selected_index: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.show = !self.show;
        if self.show {
            self.focus_left = true;
            self.selected_index = 0;
            self.tab = SettingsTab::About;
        }
    }

    pub fn move_up(&mut self) {
        if self.focus_left {
            self.selected_index = self.selected_index.checked_sub(1)
                .unwrap_or(SettingsTab::all().len() - 1);
            self.tab = SettingsTab::from_index(self.selected_index);
        }
    }

    pub fn move_down(&mut self) {
        if self.focus_left {
            self.selected_index = (self.selected_index + 1) % SettingsTab::all().len();
            self.tab = SettingsTab::from_index(self.selected_index);
        }
    }

    pub fn tab_switch(&mut self) {
        self.focus_left = !self.focus_left;
    }
}

pub fn draw_settings(ctx: &egui::Context, app: &mut App, colors: ThemeColors) {
    if !app.settings.show {
        return;
    }

    let popup_w = 500.0f32;
    let popup_h = 360.0f32;
    let center = ctx.screen_rect().center();
    let rect = egui::Rect::from_center_size(center, egui::vec2(popup_w, popup_h));

    egui::Area::new(egui::Id::new("settings_popup"))
        .fixed_pos(rect.min)
        .interactable(true)
        .show(ctx, |ui| {
            let bg = egui::Color32::from_rgba_premultiplied(
                colors.settings_bg.r(),
                colors.settings_bg.g(),
                colors.settings_bg.b(),
                colors.settings_bg.a(),
            );
            ui.painter().rect_filled(
                rect,
                egui::CornerRadius::same(6),
                bg,
            );
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(6),
                egui::Stroke::new(2.0, colors.settings_border),
                egui::StrokeKind::Inside,
            );

            let left_w = popup_w * 0.25;
            let right_w = popup_w * 0.75;
            let padding = 8.0;

            let left_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(padding, padding),
                egui::vec2(left_w - padding * 1.5, popup_h - padding * 2.0),
            );
            let divider_x = rect.left() + left_w;
            let divider_rect = egui::Rect::from_min_size(
                egui::pos2(divider_x, rect.top() + padding),
                egui::vec2(1.0, popup_h - padding * 2.0),
            );
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(divider_x + 1.0, rect.top() + padding),
                egui::vec2(right_w - padding * 1.5, popup_h - padding * 2.0),
            );

            ui.painter().line_segment(
                [divider_rect.center_top(), divider_rect.center_bottom()],
                egui::Stroke::new(1.0, colors.text_secondary),
            );

            ui.allocate_ui_at_rect(left_rect, |ui| {
                draw_left_panel(app, colors, ui);
            });

            ui.allocate_ui_at_rect(right_rect, |ui| {
                draw_right_panel(app, colors, ui);
            });

            // 焦点指示：高亮当前活动面板
            let focus_rect = if app.settings.focus_left { left_rect } else { right_rect };
            ui.painter().rect_stroke(focus_rect.shrink(-2.0), egui::CornerRadius::same(2),
                egui::Stroke::new(1.5, colors.accent), egui::StrokeKind::Inside);
        });
}

fn draw_left_panel(app: &App, colors: ThemeColors, ui: &mut egui::Ui) {
    let tabs = SettingsTab::all();
    let font_small = egui::FontId::proportional(13.0);

    for (i, tab) in tabs.iter().enumerate() {
        let is_selected = app.settings.selected_index == i && app.settings.focus_left;
        let text_color = if is_selected {
            colors.accent
        } else if app.settings.selected_index == i {
            colors.cursor
        } else {
            colors.text_primary
        };

        let prefix = if is_selected { "> " } else { "  " };
        let label = format!("{}{}", prefix, tab.label());

        ui.label(
            egui::RichText::new(label)
                .font(font_small.clone())
                .color(text_color),
        );
    }
}

fn draw_right_panel(app: &App, colors: ThemeColors, ui: &mut egui::Ui) {
    match app.settings.tab {
        SettingsTab::About => draw_about(colors, ui),
        SettingsTab::Shortcuts => draw_shortcuts(colors, ui),
        SettingsTab::Themes => draw_themes(app, colors, ui),
        SettingsTab::Extra => draw_extra(app, colors, ui),
    }
}

fn draw_about(colors: ThemeColors, ui: &mut egui::Ui) {
    let font_body = egui::FontId::proportional(12.0);
    let font_title = egui::FontId::proportional(14.0);

    ui.label(
        egui::RichText::new("monster-player")
            .font(font_title)
            .color(colors.accent),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Monster Siren Records 音乐播放器")
            .font(font_body.clone())
            .color(colors.text_primary),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("播放器内核: rodio + 渐进式缓冲流")
            .font(font_body.clone())
            .color(colors.text_secondary),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("架构: 内核共享 (TUI + GUI)")
            .font(font_body.clone())
            .color(colors.text_secondary),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("GitHub:")
            .font(font_body.clone())
            .color(colors.text_secondary),
    );
    ui.label(
        egui::RichText::new("github.com/missercatos/monster-player")
            .font(font_body)
            .color(colors.accent),
    );
}

fn draw_shortcuts(colors: ThemeColors, ui: &mut egui::Ui) {
    let font_key = egui::FontId::proportional(11.0);
    let font_desc = egui::FontId::proportional(11.0);

    let shortcuts = [
        ("Ctrl+T", "设置/帮助"),
        ("h/l", "上/下专辑"),
        ("j/k", "上/下歌曲"),
        ("Space", "播放选中"),
        ("x", "暂停/恢复"),
        ("e", "切换模式"),
        ("o/p", "音量 -/+"),
        ("s", "收藏歌曲"),
        ("v", "歌词显示"),
        ("A/D", "快退/快进"),
        ("/", "搜索"),
    ];

    for (key, desc) in shortcuts {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("{:<8}", key))
                    .font(font_key.clone())
                    .color(colors.accent),
            );
            ui.label(
                egui::RichText::new(desc)
                    .font(font_desc.clone())
                    .color(colors.text_primary),
            );
        });
    }
}

fn draw_themes(app: &App, colors: ThemeColors, ui: &mut egui::Ui) {
    let font_label = egui::FontId::proportional(12.0);
    let font_value = egui::FontId::proportional(12.0);

    ui.label(
        egui::RichText::new("主题 (h/l 切换)")
            .font(font_label.clone())
            .color(colors.text_secondary),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(format!("[{}]", app.theme_name.label()))
            .font(font_value.clone())
            .color(colors.accent),
    );

    ui.add_space(8.0);

    ui.label(
        egui::RichText::new("歌词动画 (h/l 切换)")
            .font(font_label.clone())
            .color(colors.text_secondary),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(format!("[{}]", app.lyric_animation.label()))
            .font(font_value)
            .color(colors.accent),
    );
}

fn draw_extra(app: &App, colors: ThemeColors, ui: &mut egui::Ui) {
    let font_label = egui::FontId::proportional(12.0);
    let font_value = egui::FontId::proportional(12.0);

    ui.label(
        egui::RichText::new("下载模块 (h/l 切换)")
            .font(font_label.clone())
            .color(colors.text_secondary),
    );
    ui.add_space(2.0);
    let download_text = if app.download_enabled { "on" } else { "off" };
    ui.label(
        egui::RichText::new(format!("[{}]", download_text))
            .font(font_value)
            .color(colors.accent),
    );

    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("注: 下载模块属于内核模块")
            .font(egui::FontId::proportional(8.0))
            .color(colors.text_secondary),
    );
    ui.label(
        egui::RichText::new("暂未实现，后续版本开放")
            .font(egui::FontId::proportional(8.0))
            .color(colors.text_secondary),
    );
}
