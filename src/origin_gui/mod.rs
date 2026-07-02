mod app;
mod settings;
mod theme;
mod ui;

use app::App;
use ui::GuiState;

/// GUI 入口：创建 frameless 透明窗口，加载 CJK 字体
pub fn run() {
    let opts = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([420.0, 340.0])
            .with_transparent(true)
            .with_decorations(false)
            .with_resizable(true)
            .with_min_inner_size([300.0, 200.0]),
        ..Default::default()
    };

    eframe::run_native(
        "origin-gui",
        opts,
        Box::new(|cc| {
            setup_cjk_fonts(&cc.egui_ctx);
            let app = App::new();
            let gui = GuiState::new();
            Ok(Box::new(OriginApp { app, gui }))
        }),
    )
    .expect("failed to launch GUI");
}

/// eframe App 容器：持有 App + GuiState
struct OriginApp {
    app: App,
    gui: GuiState,
}

impl eframe::App for OriginApp {
    /// 透明窗口背景
    fn clear_color(&self, _visuals: &eframe::egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ui::update(&mut self.app, &mut self.gui, ctx, frame);
    }
}

/// 加载系统 Noto Sans CJK 中文字体
fn setup_cjk_fonts(ctx: &eframe::egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();

    if let Some(cjk) = find_cjk_font() {
        fonts.font_data.insert("cjk".into(), cjk);
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

/// 按优先级搜索系统 CJK 字体文件
fn find_cjk_font() -> Option<std::sync::Arc<eframe::egui::FontData>> {
    #[cfg(target_os = "windows")]
    {
        // 先尝试已知路径
        let known: &[&str] = &[
            r"C:\Windows\Fonts\msyh.ttc",
            r"C:\Windows\Fonts\msyhbd.ttc",
            r"C:\Windows\Fonts\msyhl.ttc",
            r"C:\Windows\Fonts\SIMKAI.ttf",
            r"C:\Windows\Fonts\simsun.ttc",
        ];
        for path in known {
            if let Ok(data) = std::fs::read(path) {
                return Some(std::sync::Arc::new(eframe::egui::FontData::from_owned(data)));
            }
        }
        // 遍历 Windows 字体目录查找任意 CJK 字体
        if let Ok(entries) = std::fs::read_dir(r"C:\Windows\Fonts") {
            let cjk_keywords = ["msyh", "simsun", "simkai", "simhei", "mingliu",
                "yumin", "meiryo", "msgothic", "msmincho", "noto", "han", "cjk"];
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.ends_with(".ttc") || name.ends_with(".ttf") || name.ends_with(".otf") {
                    if cjk_keywords.iter().any(|k| name.contains(k)) {
                        if let Ok(data) = std::fs::read(entry.path()) {
                            return Some(std::sync::Arc::new(eframe::egui::FontData::from_owned(data)));
                        }
                    }
                }
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    let paths: &[&str] = &[
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Light.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Medium.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/wps-office/FZFSK.TTF",
    ];
    #[cfg(target_os = "macos")]
    let paths: &[&str] = &[
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/PingFang.ttf",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
    ];

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    for path in paths {
        if let Ok(data) = std::fs::read(path) {
            return Some(std::sync::Arc::new(eframe::egui::FontData::from_owned(data)));
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    None
}
