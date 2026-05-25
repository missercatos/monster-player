use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use monster_player::kernel::PlayMode;

use super::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let inner = outer.inner(size);
    f.render_widget(outer, size);

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(inner);

    draw_left(f, app, h_chunks[0]);
    draw_right(f, app, h_chunks[1]);
}

fn draw_left(f: &mut Frame, app: &App, area: Rect) {
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_info(f, app, v_chunks[0]);
    draw_bottom(f, app, v_chunks[1]);
}

fn draw_info(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = vec![];

    if let Some(ref intro) = app.engine.album_intro {
        lines.push(Line::from(Span::styled(
            intro.clone(),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(Color::DarkGray),
        )));
    }

    if let Some(ref info) = app.engine.song_info {
        lines.push(Line::from(Span::styled(
            info.clone(),
            Style::default().fg(Color::White),
        )));
    }

    if let Some(pct) = app.engine.progress {
        let bar_w = 10usize;
        let filled = ((pct * bar_w as f64) as usize).min(bar_w);
        let unfilled = bar_w - filled;
        lines.push(Line::from(vec![
            Span::styled("#".repeat(filled), Style::default().fg(Color::Cyan)),
            Span::styled(" ".repeat(unfilled), Style::default().fg(Color::DarkGray)),
        ]));
    }

    if lines.is_empty() {
        return;
    }

    let line_count = lines.len();
    let v_pad = area.height.saturating_sub(line_count as u16) / 2;
    let display_area = Rect {
        y: area.y + v_pad,
        height: line_count as u16,
        ..area
    };

    if lines.len() >= 2 {
        lines[1] = Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(Color::DarkGray),
        ));
    }

    let p = Paragraph::new(lines);
    f.render_widget(p, display_area);
}

fn draw_bottom(f: &mut Frame, app: &App, area: Rect) {
    let play_state = if app.engine.playing {
        "O Playing"
    } else {
        "X Paused"
    };

    let mode_text = match app.engine.play_mode {
        PlayMode::AlbumList => "Album List",
        PlayMode::AlbumRandom => "Album Random",
        PlayMode::GlobalList => "Global List",
        PlayMode::GlobalRandom => "Global Random",
        PlayMode::Single => "Single",
        PlayMode::LoveList => "Love List",
        PlayMode::LoveRandom => "Love Random",
    };

    let volume_str = format!("Volume: {}%", app.engine.volume);

    let mut items: Vec<String> = if app.show_help {
        vec![
            "q         Quit".into(),
            "h/l  ←/→  Prev/Next album".into(),
            "j/k  ↓/↑  Prev/Next song".into(),
            "A/D  Shift  Prev/Next song".into(),
            "a/d         Seek backward/forward 5%".into(),
            "Space       Play selected song".into(),
            "x           Pause / Resume".into(),
            "e           Cycle play mode".into(),
            "o           Volume -5%".into(),
            "p           Volume +5%".into(),
            "v           Toggle lyrics".into(),
            "s           Toggle love on selected song".into(),
            "Ctrl+T      Toggle this help".into(),
        ]
    } else {
        vec![
            play_state.into(),
            mode_text.into(),
            volume_str,
            "Ctrl+T for help".into(),
        ]
    };

    if !app.show_help {
        if app.engine.buffering {
            if let Some(ref msg) = app.engine.buffering_msg {
                items.insert(1, msg.clone());
            }
        } else if let Some(ref name) = app.engine.current_song_name {
            items.insert(1, name.clone());
        }
    }

    let text: Vec<Line> = items
        .iter()
        .map(|s| Line::from(Span::styled(s.clone(), Style::default().fg(Color::White))))
        .collect();
    let line_count = text.len() as u16;
    let p = Paragraph::new(text).centered();
    let v_center = area.height.saturating_sub(line_count) / 2;
    let centered_area = Rect {
        y: area.y + v_center,
        height: line_count,
        ..area
    };
    f.render_widget(p, centered_area);
}

fn draw_loved_view(f: &mut Frame, app: &App, area: Rect) {
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(area);

    let header_lines = vec![Line::from(Span::styled(
        format!("Loved Songs ({})", app.engine.loved_list.len()),
        Style::default().fg(Color::Cyan),
    ))];
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let p = Paragraph::new(header_lines).block(block).centered();
    f.render_widget(p, v_chunks[0]);

    if app.engine.loved_list.is_empty() {
        let text = vec![Line::from(Span::styled(
            "No loved songs. Press S on a song to love it.",
            Style::default().fg(Color::DarkGray),
        ))];
        let p = Paragraph::new(text).centered();
        f.render_widget(p, v_chunks[1]);
        return;
    }

    let max_display = (v_chunks[1].height as usize).saturating_sub(2);
    let total = app.engine.loved_list.len();
    let start = if total <= max_display {
        0
    } else {
        let half = max_display / 2;
        app.selected_song
            .saturating_sub(half)
            .min(total.saturating_sub(max_display))
    };
    let end = (start + max_display).min(total);

    let mut lns: Vec<Line> = vec![];
    for i in start..end {
        let entry = &app.engine.loved_list[i];
        let prefix = if i == app.selected_song { "> " } else { "  " };
        let is_playing = app
            .engine
            .current_song_cid
            .as_ref()
            .map_or(false, |cid| cid == &entry.cid);

        let color = if is_playing {
            Color::Cyan
        } else if i == app.selected_song {
            Color::White
        } else {
            Color::Gray
        };

        let text = format!(
            "{}{} - {}",
            prefix,
            entry.name,
            entry.artists.join(", ")
        );
        lns.push(Line::from(vec![
            Span::styled(text, Style::default().fg(color)),
            Span::styled(" *", Style::default().fg(Color::Red)),
        ]));
    }

    let p = Paragraph::new(lns);
    f.render_widget(p, v_chunks[1]);
}

fn draw_right(f: &mut Frame, app: &App, area: Rect) {
    if app.show_lyrics {
        draw_lyrics(f, app, area);
        return;
    }

    let is_love = matches!(
        app.engine.play_mode,
        PlayMode::LoveList | PlayMode::LoveRandom
    );

    if is_love {
        draw_loved_view(f, app, area);
        return;
    }

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(area);

    draw_album_info(f, app, v_chunks[0]);
    draw_song_list(f, app, v_chunks[1]);
}

fn draw_album_info(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = vec![];

    if let Some(ref name) = app.engine.album_name {
        lines.push(Line::from(Span::styled(
            name.clone(),
            Style::default().fg(Color::Cyan),
        )));
    }
    if let Some(ref artist) = app.engine.album_artist {
        lines.push(Line::from(Span::styled(
            artist.clone(),
            Style::default().fg(Color::Gray),
        )));
    }
    if app.engine.album_total > 1 {
        lines.push(Line::from(Span::styled(
            format!(
                "[{}/{}]",
                app.engine.album_index + 1,
                app.engine.album_total
            ),
            Style::default().fg(Color::DarkGray),
        )));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Loading...",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let p = Paragraph::new(lines).block(block).centered();
    f.render_widget(p, area);
}

fn draw_song_list(f: &mut Frame, app: &App, area: Rect) {
    if !app.engine.songs_loaded {
        let text = vec![Line::from(Span::styled(
            "Loading songs...",
            Style::default().fg(Color::DarkGray),
        ))];
        let p = Paragraph::new(text).centered();
        f.render_widget(p, area);
        return;
    }

    let max_display = (area.height as usize).saturating_sub(2);
    let total = app.engine.songs.len();
    let start = if total <= max_display {
        0
    } else {
        let half = max_display / 2;
        app.selected_song
            .saturating_sub(half)
            .min(total.saturating_sub(max_display))
    };
    let end = (start + max_display).min(total);

    let mut lns: Vec<Line> = vec![];
    for i in start..end {
        let song = &app.engine.songs[i];
        let prefix = if i == app.selected_song { "> " } else { "  " };
        let is_playing = app
            .engine
            .current_song_cid
            .as_ref()
            .map_or(false, |cid| cid == &song.cid);

        let color = if is_playing {
            Color::Cyan
        } else if i == app.selected_song {
            Color::White
        } else {
            Color::Gray
        };

        let text = format!(
            "{}{} - {}",
            prefix,
            song.name,
            song.artistes.join(", ")
        );
        if app.engine.is_loved(&song.cid) {
            lns.push(Line::from(vec![
                Span::styled(text, Style::default().fg(color)),
                Span::styled(" *", Style::default().fg(Color::Red)),
            ]));
        } else {
            lns.push(Line::from(Span::styled(text, Style::default().fg(color))));
        }
    }

    let p = Paragraph::new(lns);
    f.render_widget(p, area);
}

fn draw_lyrics(f: &mut Frame, app: &App, area: Rect) {
    if app.engine.lyrics.is_empty() {
        let text = vec![Line::from(Span::styled(
            if app.engine.current_song_name.is_some() {
                "No lyrics available"
            } else {
                "No song playing"
            },
            Style::default().fg(Color::DarkGray),
        ))];
        let p = Paragraph::new(text).centered();
        f.render_widget(p, area);
        return;
    }

    let max_display = (area.height as usize).saturating_sub(2);
    let total = app.engine.lyrics.len();
    let half = max_display / 2;
    let start = if total <= max_display {
        0
    } else {
        app.engine
            .lyric_index
            .saturating_sub(half)
            .min(total.saturating_sub(max_display))
    };
    let end = (start + max_display).min(total);

    let mut lines: Vec<Line> = vec![];
    for i in start..end {
        let (_, ref text) = app.engine.lyrics[i];
        let is_current = i == app.engine.lyric_index;

        let span = if is_current {
            Span::styled(
                format!("> {}", text),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
        } else {
            Span::styled(format!("  {}", text), Style::default().fg(Color::White))
        };
        lines.push(Line::from(span));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Lyrics (V to toggle) ");
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}
