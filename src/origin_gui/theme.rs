use eframe::egui::Color32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeName {
    Origin,
    Tty,
    Tokyonight,
}

impl ThemeName {
    pub fn label(&self) -> &'static str {
        match self {
            ThemeName::Origin => "origin",
            ThemeName::Tty => "tty",
            ThemeName::Tokyonight => "tokyonight",
        }
    }

    pub fn next(&self) -> ThemeName {
        match self {
            ThemeName::Origin => ThemeName::Tty,
            ThemeName::Tty => ThemeName::Tokyonight,
            ThemeName::Tokyonight => ThemeName::Origin,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ThemeColors {
    pub bg_fill: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub accent: Color32,
    pub cursor: Color32,
    pub loved: Color32,
    pub progress_bar: Color32,
    pub progress_bg: Color32,
    pub search_icon: Color32,
    pub search_bg: Color32,
    pub search_text: Color32,
    pub settings_bg: Color32,
    pub settings_border: Color32,
    pub card_bg: Color32,
    pub card_border: Color32,
}

impl ThemeColors {
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Origin => Self::origin(),
            ThemeName::Tty => Self::tty(),
            ThemeName::Tokyonight => Self::tokyonight(),
        }
    }

    fn origin() -> Self {
        Self {
            bg_fill: Color32::from_rgba_premultiplied(0, 0, 0, 120),
            border: Color32::WHITE,
            text_primary: Color32::WHITE,
            text_secondary: Color32::GRAY,
            accent: Color32::from_rgb(0, 200, 200),
            cursor: Color32::WHITE,
            loved: Color32::RED,
            progress_bar: Color32::from_rgb(0, 200, 200),
            progress_bg: Color32::from_rgb(60, 60, 60),
            search_icon: Color32::WHITE,
            search_bg: Color32::from_rgba_premultiplied(0, 0, 0, 220),
            search_text: Color32::WHITE,
            settings_bg: Color32::from_rgba_premultiplied(0, 0, 0, 200),
            settings_border: Color32::WHITE,
            card_bg: Color32::from_rgba_premultiplied(0, 0, 0, 120),
            card_border: Color32::WHITE,
        }
    }

    fn tty() -> Self {
        Self {
            bg_fill: Color32::from_rgba_premultiplied(0, 0, 0, 255),
            border: Color32::WHITE,
            text_primary: Color32::from_gray(180),
            text_secondary: Color32::from_gray(100),
            accent: Color32::WHITE,
            cursor: Color32::BLACK,
            loved: Color32::from_gray(200),
            progress_bar: Color32::WHITE,
            progress_bg: Color32::from_gray(60),
            search_icon: Color32::WHITE,
            search_bg: Color32::from_gray(20),
            search_text: Color32::from_gray(180),
            settings_bg: Color32::from_gray(10),
            settings_border: Color32::WHITE,
            card_bg: Color32::from_rgba_premultiplied(0, 0, 0, 255),
            card_border: Color32::WHITE,
        }
    }

    fn tokyonight() -> Self {
        Self {
            bg_fill: Color32::from_rgba_premultiplied(26, 27, 38, 240),
            border: Color32::from_rgb(122, 162, 247),
            text_primary: Color32::from_rgb(192, 202, 247),
            text_secondary: Color32::from_rgb(137, 142, 165),
            accent: Color32::from_rgb(122, 162, 247),
            cursor: Color32::from_rgb(187, 154, 247),
            loved: Color32::from_rgb(247, 118, 142),
            progress_bar: Color32::from_rgb(122, 162, 247),
            progress_bg: Color32::from_rgb(50, 50, 70),
            search_icon: Color32::from_rgb(122, 162, 247),
            search_bg: Color32::from_rgba_premultiplied(26, 27, 38, 240),
            search_text: Color32::from_rgb(192, 202, 247),
            settings_bg: Color32::from_rgba_premultiplied(26, 27, 38, 240),
            settings_border: Color32::from_rgb(122, 162, 247),
            card_bg: Color32::from_rgba_premultiplied(26, 27, 38, 240),
            card_border: Color32::from_rgb(122, 162, 247),
        }
    }

}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LyricAnimation {
    FontGrow,
    ColorChange,
}

impl LyricAnimation {
    pub fn label(&self) -> &'static str {
        match self {
            LyricAnimation::FontGrow => "字体放大",
            LyricAnimation::ColorChange => "颜色变化",
        }
    }

    pub fn next(&self) -> LyricAnimation {
        match self {
            LyricAnimation::FontGrow => LyricAnimation::ColorChange,
            LyricAnimation::ColorChange => LyricAnimation::FontGrow,
        }
    }
}
