use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, Mode};

const LINE_NUMBER_WIDTH: usize = 2;
const TEXT_GAP: usize = 2;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let visible_height = chunks[0].height as usize;
    if app.cursor_row < app.scroll {
        app.scroll = app.cursor_row;
    } else if app.cursor_row >= app.scroll + visible_height {
        app.scroll = app
            .cursor_row
            .saturating_sub(visible_height.saturating_sub(1));
    }

    let text: Vec<Line> = app.lines[app.scroll..]
        .iter()
        .enumerate()
        .take(visible_height)
        .map(|(index, line)| {
            Line::from(vec![
                Span::styled(
                    format!(
                        "{:>width$}{}",
                        app.scroll + index + 1,
                        " ".repeat(TEXT_GAP),
                        width = LINE_NUMBER_WIDTH
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(line.clone()),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(text), chunks[0]);
    app.lsp.draw(frame, chunks[1]);

    let mode = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::Command => "COMMAND",
    };
    let footer = if app.mode == Mode::Command {
        format!(":{}", app.command)
    } else {
        format!(" {mode} ")
    };
    frame.render_widget(
        Paragraph::new(footer).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        chunks[2],
    );

    if app.mode != Mode::Command {
        frame.set_cursor_position((
            (LINE_NUMBER_WIDTH + TEXT_GAP) as u16 + app.cursor_col as u16,
            (app.cursor_row - app.scroll) as u16,
        ));
    }
}
