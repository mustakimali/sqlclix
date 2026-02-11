use crate::app::{App, Panel};
use crate::highlight::SqlHighlighter;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn render_tabs(app: &App) -> Line<'static> {
    let mut spans = Vec::new();
    spans.push(Span::raw(" "));

    for (i, tab) in app.tabs.iter().enumerate() {
        let is_active = i == app.active_tab;
        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        spans.push(Span::styled(format!(" {} ", tab.name), style));
        spans.push(Span::raw(" "));
    }

    spans.push(Span::styled("[+]", Style::default().fg(Color::DarkGray)));

    Line::from(spans)
}

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let highlighter = SqlHighlighter::new();
    let visible_lines = area.height as usize;
    let is_focused = app.focus == Panel::Editor;

    // Ensure cursor is visible
    {
        let tab = app.current_tab_mut();
        tab.ensure_cursor_visible(visible_lines.saturating_sub(1));
    }

    // Now borrow immutably for rendering
    let tab = app.current_tab();
    let mut lines: Vec<Line> = Vec::new();
    let line_num_width = tab.content.len().to_string().len().max(2);

    for (i, line_content) in tab.content.iter().enumerate() {
        if i < tab.scroll_offset {
            continue;
        }
        if lines.len() >= visible_lines {
            break;
        }

        let line_num = format!("{:>width$} ", i + 1, width = line_num_width);
        let mut spans = vec![Span::styled(line_num, Style::default().fg(Color::DarkGray))];

        // Apply syntax highlighting
        let highlighted = highlighter.highlight_line(line_content);

        // If this is the cursor line and editor is focused, we need to handle cursor
        if i == tab.cursor_line && is_focused {
            let cursor_col = tab.cursor_col.min(line_content.len());
            let mut col = 0;

            for span in highlighted.spans {
                let span_text = span.content.to_string();
                let span_len = span_text.len();

                if col + span_len <= cursor_col {
                    // Entire span is before cursor
                    spans.push(Span::styled(span_text, span.style));
                    col += span_len;
                } else if col > cursor_col {
                    // Entire span is after cursor (cursor already rendered)
                    spans.push(Span::styled(span_text, span.style));
                    col += span_len;
                } else {
                    // Cursor is within this span
                    let before_cursor = cursor_col - col;
                    let (before, rest) = span_text.split_at(before_cursor);

                    if !before.is_empty() {
                        spans.push(Span::styled(before.to_string(), span.style));
                    }

                    // Cursor character
                    if !rest.is_empty() {
                        let mut chars = rest.chars();
                        let cursor_char = chars.next().unwrap();
                        let after: String = chars.collect();

                        spans.push(Span::styled(
                            cursor_char.to_string(),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ));

                        if !after.is_empty() {
                            spans.push(Span::styled(after, span.style));
                        }
                    }

                    col += span_len;
                }
            }

            // If cursor is at end of line
            if cursor_col >= line_content.len() {
                spans.push(Span::styled(
                    " ",
                    Style::default().fg(Color::Black).bg(Color::White),
                ));
            }
        } else {
            spans.extend(
                highlighted
                    .spans
                    .into_iter()
                    .map(|s| Span::styled(s.content.to_string(), s.style)),
            );
        }

        lines.push(Line::from(spans));
    }

    // Fill remaining space with empty line numbers
    while lines.len() < visible_lines {
        let line_num = format!("{:>width$} ", "~", width = line_num_width);
        lines.push(Line::from(Span::styled(
            line_num,
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
