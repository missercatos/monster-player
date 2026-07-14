use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

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
    // 搜索模式：拦截所有键盘事件
    if app.search_mode {
        match key.code {
            KeyCode::Esc => {
                app.exit_search();
            }
            KeyCode::Enter => {
                app.search_confirm();
            }
            KeyCode::Backspace => {
                app.search_backspace();
            }
            KeyCode::Up | KeyCode::Char('k')
                if !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                app.search_prev();
            }
            KeyCode::Down | KeyCode::Char('j')
                if !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                app.search_next();
            }
            KeyCode::Char(c) => {
                app.search_input(c);
            }
            _ => {}
        }
        return true;
    }

    // 非搜索模式
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('t') {
        app.toggle_help();
        return true;
    }

    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    match key.code {
        KeyCode::Char('q') => return false,
        KeyCode::Char('/') => app.enter_search(),
        KeyCode::Enter => app.play_selected(),
        KeyCode::Char(' ') => app.toggle_pause(),
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
