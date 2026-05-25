use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent};

pub fn poll(timeout: Duration) -> std::io::Result<bool> {
    crossterm::event::poll(timeout)
}

pub fn read() -> std::io::Result<Event> {
    crossterm::event::read()
}

use super::app::App;

pub fn handle_key(app: &mut App, key: &KeyEvent) -> bool {
    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) && key.code == KeyCode::Char('t') {
        app.toggle_help();
        return true;
    }

    let shift = key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT);

    match key.code {
        KeyCode::Char('q') => return false,
        KeyCode::Char(' ') => app.play_selected(),
        KeyCode::Char('x') => app.toggle_pause(),
        KeyCode::Char('e') => app.cycle_mode(),
        KeyCode::Char('o') => app.volume_down(),
        KeyCode::Char('p') => app.volume_up(),
        KeyCode::Char('v') => app.toggle_lyrics(),
        KeyCode::Char('a') => {
            if shift {
                app.prev_song();
            } else {
                app.seek_backward();
            }
        }
        KeyCode::Char('d') => {
            if shift {
                app.next_song();
            } else {
                app.seek_forward();
            }
        }
        KeyCode::Char('h') | KeyCode::Left => app.prev_album(),
        KeyCode::Char('l') | KeyCode::Right => app.next_album(),
        KeyCode::Char('k') | KeyCode::Up => app.prev_song(),
        KeyCode::Char('j') | KeyCode::Down => app.next_song(),
        _ => {}
    }
    true
}
