use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent};

/// 轮询键盘事件，非阻塞
pub fn poll(timeout: Duration) -> std::io::Result<bool> {
    crossterm::event::poll(timeout)
}

/// 读取下一个事件
pub fn read() -> std::io::Result<Event> {
    crossterm::event::read()
}

use super::app::App;

/// 按键分发：将 crossterm KeyCode 映射到 App 方法
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
        KeyCode::Char('s') => app.toggle_love(),
        KeyCode::Char('a') | KeyCode::Char('A') => {
            if shift {
                app.play_prev();
            } else {
                app.seek_backward();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if shift {
                app.play_next();
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
