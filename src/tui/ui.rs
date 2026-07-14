use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use monster_player::kernel::PlayMode;

use super::app::App;

/// 将文本按指定字符宽度拆分为多行
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    chars
        .chunks(width)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

/// 搜索栏：顶部白色横线 + 输入框 + 匹配结果列表
fn draw_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let max_results = 5usize;
    let bar_height = 3 + max_results as u16; // 横线 + 输入框 + 空行 + 结果列表
    let bar_height = bar_height.min(area.height);

    let bar_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: bar_height,
    };

    // 背景遮罩（深色半透明效果）
    let bg = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(bg, bar_area);

    // 白色横线
    let line_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    let separator = Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::White),
    ));
    let sep_p = Paragraph::new(separator);
    f.render_widget(sep_p, line_area);

    // 输入框
    let input_y = area.y + 1;
    let input_area = Rect {
        x: area.x,
        y: input_y,
        width: area.width,
        height: 1,
    };
    let input_text = format!("/ {}█", app.search_query);
    let input_color = if app.search_confirmed {
        Color::Cyan
    } else {
        Color::White
    };
    let input_p = Paragraph::new(Line::from(Span::styled(
        input_text,
        Style::default().fg(input_color),
    )));
    f.render_widget(input_p, input_area);

    // 搜索结果列表
    let results_y = input_y + 2;
    let mut results_lines: Vec<Line> = vec![];
    let show_count = app.search_results.len().min(max_results);
    for i in 0..show_count {
        let song = &app.search_results[i];
        let prefix = if i == app.search_index { "> " } else { "  " };
        let color = if i == app.search_index {
            Color::White
        } else {
            Color::Gray
        };
        let text = format!("{}{} - {}", prefix, song.name, song.artists.join(", "));
        results_lines.push(Line::from(Span::styled(
            text,
            Style::default().fg(color),
        )));
    }
    if app.search_results.is_empty() && !app.search_query.is_empty() {
        results_lines.push(Line::from(Span::styled(
            "  No results",
            Style::default().fg(Color::DarkGray),
        )));
    }

    if !results_lines.is_empty() {
        let results_area = Rect {
            x: area.x,
            y: results_y,
            width: area.width,
            height: (show_count as u16).min(area.height.saturating_sub(3)),
        };
        let results_p = Paragraph::new(results_lines);
        f.render_widget(results_p, results_area);
    }
}

/// 主渲染入口：外层边框 + 左右分栏布局
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

    // 搜索模式：在顶部渲染搜索栏（覆盖）
    if app.search_mode {
        draw_search_bar(f, app, size);
    }
}

/// 左侧区域：信息栏 + 底部状态栏
fn draw_left(f: &mut Frame, app: &App, area: Rect) {
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_info(f, app, v_chunks[0]);
    draw_bottom(f, app, v_chunks[1]);
}

/// 显示专辑简介 + 歌曲信息 + 进度条
fn draw_info(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = vec![];
    let char_width = (area.width as usize).saturating_sub(2);

    if let Some(ref intro) = app.engine.album_intro {
        for line_text in wrap_text(intro, char_width) {
            lines.push(Line::from(Span::styled(
                line_text,
                Style::default().fg(Color::Gray),
            )));
        }
        lines.push(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(Color::DarkGray),
        )));
    }

    if let Some(ref info) = app.engine.song_info {
        for line_text in wrap_text(info, (area.width as usize).saturating_sub(2)) {
            lines.push(Line::from(Span::styled(
                line_text,
                Style::default().fg(Color::White),
            )));
        }
    }

    if let Some(pct) = app.engine.progress {
        let bar_w = 10usize;
        let filled = ((pct * bar_w as f64) as usize).min(bar_w);
        let unfilled = bar_w - filled;
        lines.push(Line::from(vec![
            Span::styled("#".repeat(filled), Style::default().fg(Color::Cyan)),
            Span::styled(" ".repeat(unfilled), Style::default().fg(Color::DarkGray)),
        ]));

        let elapsed = app.engine.elapsed_secs();
        let time_text = if let Some(dur) = app.engine.duration_secs() {
            format!("{:02}:{:02}/{:02}:{:02}",
                elapsed as u64 / 60, elapsed as u64 % 60,
                dur as u64 / 60, dur as u64 % 60)
        } else {
            format!("{:02}:{:02}", elapsed as u64 / 60, elapsed as u64 % 60)
        };
        lines.push(Line::from(Span::styled(
            time_text,
            Style::default().fg(Color::DarkGray),
        )));
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

    let p = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center);
    f.render_widget(p, display_area);
}

/// 底部状态栏：播放状态/模式/音量/快捷键提示
fn draw_bottom(f: &mut Frame, app: &App, area: Rect) {
    let play_state = if app.engine.playing {
        "O Playing"
    } else {
        "O Paused"
    };

    let mode_text = match app.engine.play_mode {
        PlayMode::AlbumList => "专辑列表",
        PlayMode::AlbumRandom => "专辑随机",
        PlayMode::GlobalList => "全局列表",
        PlayMode::GlobalRandom => "全局随机",
        PlayMode::Single => "单曲循环",
        PlayMode::LoveList => "收藏列表",
        PlayMode::LoveRandom => "收藏随机",
    };

    let volume_str = format!("Volume: {}%", app.engine.volume);

    if app.show_help {
        let shortcuts: Vec<(&str, &str)> = vec![
            ("q", "Quit"),
            ("/", "Search songs"),
            ("h/l, ←/→", "Prev/Next album"),
            ("j/k, ↓/↑", "Prev/Next song"),
            ("A/D (Shift)", "Prev/Next song"),
            ("a/d", "Seek backward/forward 5%"),
            ("Enter", "Play selected song"),
            ("Space", "Pause / Resume"),
            ("e", "Cycle play mode"),
            ("o", "Volume -5%"),
            ("p", "Volume +5%"),
            ("v", "Toggle lyrics"),
            ("s", "Toggle love on selected song"),
            ("Ctrl+T", "Toggle this help"),
        ];

        let rows: Vec<Row> = shortcuts
            .iter()
            .map(|(key, desc)| {
                Row::new(vec![
                    Cell::from(*key).style(Style::default().fg(Color::White)),
                    Cell::from(*desc).style(Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let widths = [Constraint::Length(18), Constraint::Min(0)];
        let help_table = Table::new(rows, widths);

        let line_count = shortcuts.len() as u16;
        let v_center = area.height.saturating_sub(line_count) / 2;
        let centered_area = Rect {
            y: area.y + v_center,
            height: line_count,
            ..area
        };
        f.render_widget(help_table, centered_area);
    } else {
        let mut items: Vec<String> = vec![
            play_state.into(),
            mode_text.into(),
            volume_str,
            "Ctrl+T for help".into(),
        ];

        if app.engine.buffering {
            if let Some(ref msg) = app.engine.buffering_msg {
                items.insert(1, msg.clone());
            }
        } else if let Some(ref name) = app.engine.current_song_name {
            items.insert(1, name.clone());
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
}

/// 收藏视图：显示所有收藏歌曲列表
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

/// 右侧区域：根据状态分发到 歌词/收藏/专辑+歌曲 视图
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

/// 专辑名称 + 艺术家 + 页码
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

/// 歌曲列表：选中标记 > / 播放中青色 / 收藏红色 *
fn draw_song_list(f: &mut Frame, app: &App, area: Rect) {
    // 全局模式：从 global_playlist 渲染
    if app.engine.is_global_mode() {
        if app.engine.global_playlist.is_empty() {
            let text = vec![Line::from(Span::styled(
                "Loading global playlist...",
                Style::default().fg(Color::DarkGray),
            ))];
            let p = Paragraph::new(text).centered();
            f.render_widget(p, area);
            return;
        }

        let max_display = (area.height as usize).saturating_sub(2);
        let total = app.engine.global_playlist.len();
        let current = app.engine.global_index;
        let start = if total <= max_display {
            0
        } else {
            let half = max_display / 2;
            current
                .saturating_sub(half)
                .min(total.saturating_sub(max_display))
        };
        let end = (start + max_display).min(total);

        let mut lns: Vec<Line> = vec![];
        for i in start..end {
            let song = &app.engine.global_playlist[i];
            let prefix = if i == current { "> " } else { "  " };
            let is_playing = app
                .engine
                .current_song_cid
                .as_ref()
                .map_or(false, |cid| cid == &song.cid);

            let color = if is_playing {
                Color::Cyan
            } else if i == current {
                Color::White
            } else {
                Color::Gray
            };

            let text = format!(
                "{}{} - {}",
                prefix,
                song.name,
                song.artists.join(", ")
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
        return;
    }

    // 专辑模式：原有逻辑
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

/// 歌词视图：当前行青色加粗，滚动跟随
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
