use crate::app::{App, Panel};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::{editor, help, results, sidebar};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(10),   // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    render_title_bar(frame, app, chunks[0]);
    render_main_content(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    if app.show_cell_detail {
        results::render_cell_detail(frame, app);
    }

    if app.show_help {
        help::render(frame, app);
    }
}

fn render_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let db_name = std::path::Path::new(&app.db.path())
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| app.db.path());

    let title = Line::from(vec![
        Span::styled(" SQLClix ", Style::default().fg(Color::Cyan)),
        Span::styled("─ ", Style::default().fg(Color::DarkGray)),
        Span::styled(db_name, Style::default().fg(Color::White)),
        Span::styled(
            format!(
                "{:>width$}",
                "[?] Help ",
                width = area.width as usize
                    - 12
                    - app.db.path().len().min(area.width as usize - 20)
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(title).style(Style::default().bg(Color::Black)),
        area,
    );
}

fn render_main_content(frame: &mut Frame, app: &mut App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30), // Sidebar
            Constraint::Min(40),    // Editor + Results
        ])
        .split(area);

    // Sidebar
    let sidebar_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == Panel::Sidebar))
        .title(Span::styled(
            " Schema ",
            Style::default().fg(if app.focus == Panel::Sidebar {
                Color::Cyan
            } else {
                Color::White
            }),
        ));

    let sidebar_inner = sidebar_block.inner(main_chunks[0]);
    frame.render_widget(sidebar_block, main_chunks[0]);
    sidebar::render(frame, app, sidebar_inner);

    // Right side: Editor + Results
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Editor
            Constraint::Percentage(50), // Results
        ])
        .split(main_chunks[1]);

    // Editor
    let editor_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == Panel::Editor))
        .title(editor::render_tabs(app));

    let editor_inner = editor_block.inner(right_chunks[0]);
    frame.render_widget(editor_block, right_chunks[0]);
    editor::render(frame, app, editor_inner);

    // Results
    let result_title = match &app.result {
        Some(r) if r.error.is_some() => " Error ".to_string(),
        Some(r) => format!(
            " Results ({} rows) ─ Page {}/{} ",
            r.row_count,
            app.result_page + 1,
            app.result_page_count()
        ),
        None => " Results ".to_string(),
    };

    let results_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == Panel::Results))
        .title(Span::styled(
            result_title,
            Style::default().fg(if app.focus == Panel::Results {
                Color::Cyan
            } else {
                Color::White
            }),
        ));

    let results_inner = results_block.inner(right_chunks[1]);
    frame.render_widget(results_block, right_chunks[1]);
    results::render(frame, app, results_inner);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let hints = match app.focus {
        Panel::Sidebar => {
            "↑↓:Navigate  Enter:Select/Expand  s:SELECT  c:COUNT  d:Schema  r:Refresh"
        }
        Panel::Editor => "F5:Run  F2:New Tab  Ctrl+W:Close  Alt+←→:Switch Tab",
        Panel::Results => "↑↓←→:Navigate  Enter:View  PgUp/Dn:Scroll  Home/End:Jump",
    };

    let time_info = match &app.result {
        Some(r) => format!(" {:?} ", r.execution_time),
        None => String::new(),
    };

    let status = Line::from(vec![
        Span::styled(" ", Style::default().bg(Color::DarkGray)),
        Span::styled(hints, Style::default().fg(Color::White).bg(Color::DarkGray)),
        Span::styled(
            format!(
                "{:>width$}",
                time_info,
                width = area.width.saturating_sub(hints.len() as u16 + 2) as usize
            ),
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(status).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
