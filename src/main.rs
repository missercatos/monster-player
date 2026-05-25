#[cfg(feature = "tui")]
mod tui;

#[cfg(feature = "gui")]
mod origin_gui;

fn main() {
    env_logger::init();

    #[cfg(feature = "tui")]
    if let Err(e) = tui::run() {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }

    #[cfg(feature = "gui")]
    origin_gui::run();
}
