use crate::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn render(frame: &mut Frame, _app: &App) {
    let area = centered_rect(70, 80, frame.area());

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Keyboard Shortcuts ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let shortcuts: Vec<(&str, &str, bool)> = vec![
        ("", "Global", true),
        ("F5 / Ctrl+R / Ctrl+Enter", "Execute query", false),
        ("F2", "New query tab", false),
        ("Ctrl+W", "Close current tab", false),
        ("Alt+← / Alt+→", "Switch tabs", false),
        ("Tab / Shift+Tab", "Switch panels", false),
        ("Ctrl+PgUp / Ctrl+PgDn", "Previous/Next result page", false),
        ("? / F1", "Toggle help", false),
        ("Esc", "Close help / Cancel", false),
        ("q / Ctrl+C", "Quit", false),
        ("", "", false),
        ("", "Schema Browser (Left Panel)", true),
        ("↑ / ↓ / j / k", "Navigate items", false),
        ("Enter / →", "Expand or generate SELECT query", false),
        ("← / Space", "Collapse item", false),
        ("s", "Generate SELECT * query", false),
        ("c", "Generate COUNT(*) query", false),
        ("d", "Generate schema query (PRAGMA)", false),
        ("r", "Refresh schema", false),
        ("", "", false),
        ("", "SQL Editor", true),
        ("Arrow keys", "Move cursor", false),
        ("Home / End", "Go to line start/end", false),
        ("Backspace / Delete", "Delete character", false),
        ("Enter", "New line", false),
        ("PgUp / PgDn", "Scroll editor", false),
        ("", "", false),
        ("", "Results Panel", true),
        ("↑ / ↓ / j / k", "Scroll rows", false),
        ("← / → / h / l", "Navigate columns", false),
        ("Enter", "View cell detail", false),
        ("v", "View entire row as JSON", false),
        ("PgUp / PgDn", "Scroll by 10 rows", false),
        ("Home / End", "Go to first/last row", false),
    ];

    let lines: Vec<Line> = shortcuts
        .iter()
        .map(|(key, desc, is_header)| {
            if *is_header {
                Line::from(Span::styled(
                    format!("\n  {}", desc),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if key.is_empty() && desc.is_empty() {
                Line::from("")
            } else {
                Line::from(vec![
                    Span::styled(format!("  {:24}", key), Style::default().fg(Color::Cyan)),
                    Span::styled(desc.to_string(), Style::default().fg(Color::White)),
                ])
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
