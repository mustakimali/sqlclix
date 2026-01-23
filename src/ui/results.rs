use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

const MAX_CELL_WIDTH: usize = 40;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let result = match &app.result {
        Some(r) => r,
        None => {
            let hint = Paragraph::new(Line::from(Span::styled(
                "Press F5 to execute query",
                Style::default().fg(Color::DarkGray),
            )));
            frame.render_widget(hint, area);
            return;
        }
    };

    // Handle error
    if let Some(error) = &result.error {
        let error_lines: Vec<Line> = error
            .lines()
            .map(|line| {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Red),
                ))
            })
            .collect();
        let paragraph = Paragraph::new(error_lines);
        frame.render_widget(paragraph, area);
        return;
    }

    // No results
    if result.columns.is_empty() {
        let msg = if result.row_count == 0 {
            "Query executed successfully (no results)"
        } else {
            "Query executed successfully"
        };
        let hint = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::Green),
        )));
        frame.render_widget(hint, area);
        return;
    }

    let page_rows = app.get_current_page_rows(result);
    if page_rows.is_empty() && result.rows.is_empty() {
        let hint = Paragraph::new(Line::from(Span::styled(
            "No rows returned",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(hint, area);
        return;
    }

    // Calculate column widths
    let available_width = area.width as usize;
    let col_count = result.columns.len();
    let mut col_widths: Vec<usize> = result
        .columns
        .iter()
        .map(|c| c.width().min(MAX_CELL_WIDTH))
        .collect();

    // Consider data widths from current page
    for row in page_rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.width().min(MAX_CELL_WIDTH));
            }
        }
    }

    // Add padding and separators
    let total_width: usize = col_widths.iter().sum::<usize>() + (col_count * 3) + 1;

    // Scale down if needed
    if total_width > available_width && col_count > 0 {
        let scale = available_width as f64 / total_width as f64;
        for w in &mut col_widths {
            *w = ((*w as f64 * scale) as usize).max(3);
        }
    }

    let mut lines: Vec<Line> = Vec::new();
    let visible_height = area.height as usize;

    // Header
    let header_spans = build_row_spans(&result.columns, &col_widths, true);
    lines.push(Line::from(header_spans));

    // Separator
    let sep = build_separator(&col_widths);
    lines.push(Line::from(Span::styled(sep, Style::default().fg(Color::DarkGray))));

    // Data rows
    for (i, row) in page_rows.iter().enumerate() {
        if i < app.result_scroll {
            continue;
        }
        if lines.len() >= visible_height {
            break;
        }

        let row_spans = build_row_spans(row, &col_widths, false);
        lines.push(Line::from(row_spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn build_row_spans(cells: &[String], widths: &[usize], is_header: bool) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));

    for (i, cell) in cells.iter().enumerate() {
        let width = widths.get(i).copied().unwrap_or(10);
        let truncated = truncate_cell(cell, width);
        let padded = format!(" {:<width$} ", truncated, width = width);

        let style = if is_header {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if cell == "NULL" {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        spans.push(Span::styled(padded, style));
        spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
    }

    spans
}

fn build_separator(widths: &[usize]) -> String {
    let mut sep = String::from("├");
    for (i, &w) in widths.iter().enumerate() {
        sep.push_str(&"─".repeat(w + 2));
        if i < widths.len() - 1 {
            sep.push('┼');
        }
    }
    sep.push('┤');
    sep
}

fn truncate_cell(s: &str, max_width: usize) -> String {
    let width = s.width();
    if width <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        "…".to_string()
    } else {
        let mut result = String::new();
        let mut current_width = 0;
        for c in s.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
            if current_width + char_width + 1 > max_width {
                break;
            }
            result.push(c);
            current_width += char_width;
        }
        result.push('…');
        result
    }
}
