// 条件编译：TUI 模块仅在 "tui" feature 启用时编译
#[cfg(feature = "tui")]
mod tui;

// 条件编译：GUI 模块仅在 "gui" feature 启用时编译
#[cfg(feature = "gui")]
mod origin_gui;

fn main() {
    env_logger::init();

    // TUI 入口：feature = "tui" 时编译运行，出错则 panic 退出
    #[cfg(feature = "tui")]
    if let Err(e) = tui::run() {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }

    // GUI 入口：feature = "gui" 时编译运行，阻塞主线程直到窗口关闭
    #[cfg(feature = "gui")]
    origin_gui::run();
}
