use std::io::{self, stdout};
use std::time::Duration;

use crossterm::{
    cursor,
    event::Event,
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

mod app;
mod event;
mod ui;

use app::App;

pub fn run() -> io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let mut app = App::new();
    let mut term = Terminal::new(CrosstermBackend::new(stdout))?;

    loop {
        app.update();

        term.draw(|f| ui::draw(f, &mut app))?;

        while event::poll(Duration::from_millis(1))? {
            match event::read()? {
                Event::Key(key) => {
                    use crossterm::event::KeyEventKind;
                    if key.kind == KeyEventKind::Press {
                        if !event::handle_key(&mut app, &key) {
                            execute!(term.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
                            terminal::disable_raw_mode()?;
                            return Ok(());
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
