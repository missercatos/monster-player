// 条件编译：TUI 模块仅在 "tui" feature 启用时编译
#[cfg(feature = "tui")]
mod tui;

// 条件编译：GUI 模块仅在 "gui" feature 启用时编译
#[cfg(feature = "gui")]
mod origin_gui;

fn main() {
    env_logger::init();

    // GUI 优先：如果 gui feature 启用，只启动 GUI
    #[cfg(feature = "gui")]
    {
        origin_gui::run();
        return;
    }

    // TUI 入口：仅在 gui 未启用时运行
    #[cfg(feature = "tui")]
    if let Err(e) = tui::run() {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}
