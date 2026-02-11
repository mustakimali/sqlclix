use crate::app::{App, Panel};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

const MAX_CELL_WIDTH: usize = 40;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
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

    let is_focused = app.focus == Panel::Results;

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

    // Calculate horizontal scroll to keep selected column visible
    if app.result_col_scroll > app.result_selected_col {
        app.result_col_scroll = app.result_selected_col;
    }
    // Scroll right if selected column is beyond visible area
    loop {
        let mut used = 1usize; // leading border
        let mut last_visible_col = app.result_col_scroll;
        for (i, _) in col_widths
            .iter()
            .enumerate()
            .take(col_count)
            .skip(app.result_col_scroll)
        {
            let col_total = col_widths[i] + 3; // padding + separator
            if used + col_total > available_width && i > app.result_col_scroll {
                break;
            }
            used += col_total;
            last_visible_col = i;
        }
        if app.result_selected_col <= last_visible_col {
            break;
        }
        app.result_col_scroll += 1;
        if app.result_col_scroll >= col_count {
            break;
        }
    }

    // Determine visible column range
    let col_start = app.result_col_scroll;
    let mut col_end = col_start;
    {
        let mut used = 1usize;
        for (i, _) in col_widths
            .iter()
            .enumerate()
            .take(col_count)
            .skip(col_start)
        {
            let col_total = col_widths[i] + 3;
            if used + col_total > available_width && i > col_start {
                break;
            }
            used += col_total;
            col_end = i + 1;
        }
    }

    let visible_col_widths = &col_widths[col_start..col_end];

    let mut lines: Vec<Line> = Vec::new();
    let visible_height = area.height as usize;
    let visible_data_rows = visible_height.saturating_sub(2); // minus header and separator

    // Calculate scroll offset to keep selected row visible
    if app.result_selected_row < app.result_scroll {
        app.result_scroll = app.result_selected_row;
    } else if app.result_selected_row >= app.result_scroll + visible_data_rows {
        app.result_scroll = app
            .result_selected_row
            .saturating_sub(visible_data_rows - 1);
    }

    // Header
    let visible_columns: Vec<String> = result.columns[col_start..col_end].to_vec();
    let selected_col_in_view = if is_focused
        && app.result_selected_col >= col_start
        && app.result_selected_col < col_end
    {
        Some(app.result_selected_col - col_start)
    } else {
        None
    };
    let header_spans = build_row_spans(
        &visible_columns,
        visible_col_widths,
        true,
        None,
        selected_col_in_view,
    );
    lines.push(Line::from(header_spans));

    // Separator
    let sep = build_separator(visible_col_widths);
    lines.push(Line::from(Span::styled(
        sep,
        Style::default().fg(Color::DarkGray),
    )));

    // Data rows (with scroll offset)
    for (i, row) in page_rows.iter().enumerate().skip(app.result_scroll) {
        if lines.len() >= visible_height {
            break;
        }

        let is_selected_row = is_focused && i == app.result_selected_row;
        let selected_col = if is_selected_row {
            if app.result_selected_col >= col_start && app.result_selected_col < col_end {
                Some(app.result_selected_col - col_start)
            } else {
                None
            }
        } else {
            None
        };

        let visible_cells: Vec<String> = row[col_start..col_end.min(row.len())].to_vec();
        let row_spans = build_row_spans(
            &visible_cells,
            visible_col_widths,
            false,
            Some(is_selected_row),
            selected_col,
        );
        lines.push(Line::from(row_spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

pub fn render_cell_detail(frame: &mut Frame, app: &mut App) {
    let (title, cell_value_owned);

    if app.show_row_detail {
        cell_value_owned = match &app.row_detail_json {
            Some(json) => json.clone(),
            None => return,
        };
        title = " Row Detail ".to_string();
    } else {
        let (col_name, cell_value) = match app.get_selected_cell() {
            Some(v) => v,
            None => return,
        };
        cell_value_owned = cell_value.to_string();
        let json_result: Result<serde_json::Value, _> = serde_json::from_str(&cell_value_owned);
        title = if json_result.is_ok() {
            format!(" {} (JSON) ", col_name)
        } else {
            format!(" {} ", col_name)
        };
    }

    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&cell_value_owned);
    if let Ok(json) = json_result {
        render_json_tree(frame, app, inner, &json);
    } else {
        let content = Paragraph::new(cell_value_owned)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        frame.render_widget(content, inner);
    }
}

struct JsonLine {
    path: String,
    indent: usize,
    content: String,
    is_expandable: bool,
    is_expanded: bool,
    value_type: JsonValueType,
}

#[derive(Clone, Copy)]
enum JsonValueType {
    Object,
    Array,
    String,
    Number,
    Bool,
    Null,
}

fn render_json_tree(frame: &mut Frame, app: &mut App, area: Rect, json: &serde_json::Value) {
    let mut lines = Vec::new();
    build_json_lines(&mut lines, json, "$", 0, &app.json_expanded);

    let visible_height = area.height as usize;

    // Ensure selected line is visible
    if app.json_selected >= lines.len() {
        app.json_selected = lines.len().saturating_sub(1);
    }
    if app.json_selected < app.json_scroll {
        app.json_scroll = app.json_selected;
    } else if app.json_selected >= app.json_scroll + visible_height {
        app.json_scroll = app.json_selected.saturating_sub(visible_height - 1);
    }

    let display_lines: Vec<Line> = lines
        .iter()
        .enumerate()
        .skip(app.json_scroll)
        .take(visible_height)
        .map(|(i, json_line)| {
            let is_selected = i == app.json_selected;
            let indent_str = "  ".repeat(json_line.indent);

            let prefix = if json_line.is_expandable {
                if json_line.is_expanded {
                    "▾ "
                } else {
                    "▸ "
                }
            } else {
                "  "
            };

            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
            };

            let value_style = if is_selected {
                style
            } else {
                match json_line.value_type {
                    JsonValueType::String => Style::default().fg(Color::Green),
                    JsonValueType::Number => Style::default().fg(Color::Yellow),
                    JsonValueType::Bool => Style::default().fg(Color::Magenta),
                    JsonValueType::Null => Style::default().fg(Color::DarkGray),
                    JsonValueType::Object | JsonValueType::Array => {
                        Style::default().fg(Color::White)
                    }
                }
            };

            Line::from(vec![
                Span::styled(indent_str, style),
                Span::styled(
                    prefix.to_string(),
                    if is_selected {
                        style
                    } else {
                        Style::default().fg(Color::Cyan)
                    },
                ),
                Span::styled(json_line.content.clone(), value_style),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(display_lines);
    frame.render_widget(paragraph, area);
}

fn build_json_lines(
    lines: &mut Vec<JsonLine>,
    value: &serde_json::Value,
    path: &str,
    indent: usize,
    expanded: &std::collections::HashSet<String>,
) {
    match value {
        serde_json::Value::Object(map) => {
            let is_expanded = expanded.contains(path);
            let count = map.len();
            if indent == 0 {
                lines.push(JsonLine {
                    path: path.to_string(),
                    indent,
                    content: format!("{{}} ({} keys)", count),
                    is_expandable: !map.is_empty(),
                    is_expanded,
                    value_type: JsonValueType::Object,
                });
            }
            if is_expanded || indent == 0 {
                for (key, val) in map {
                    let child_path = format!("{}.{}", path, key);
                    let is_child_expandable = matches!(
                        val,
                        serde_json::Value::Object(_) | serde_json::Value::Array(_)
                    );
                    let is_child_expanded = expanded.contains(&child_path);

                    let content = match val {
                        serde_json::Value::Object(m) => {
                            format!("\"{}\": {{}} ({} keys)", key, m.len())
                        }
                        serde_json::Value::Array(a) => {
                            format!("\"{}\": [] ({} items)", key, a.len())
                        }
                        serde_json::Value::String(s) => {
                            format!("\"{}\": \"{}\"", key, truncate_json_string(s, 50))
                        }
                        serde_json::Value::Number(n) => format!("\"{}\": {}", key, n),
                        serde_json::Value::Bool(b) => format!("\"{}\": {}", key, b),
                        serde_json::Value::Null => format!("\"{}\": null", key),
                    };

                    let value_type = match val {
                        serde_json::Value::Object(_) => JsonValueType::Object,
                        serde_json::Value::Array(_) => JsonValueType::Array,
                        serde_json::Value::String(_) => JsonValueType::String,
                        serde_json::Value::Number(_) => JsonValueType::Number,
                        serde_json::Value::Bool(_) => JsonValueType::Bool,
                        serde_json::Value::Null => JsonValueType::Null,
                    };

                    lines.push(JsonLine {
                        path: child_path.clone(),
                        indent: indent + 1,
                        content,
                        is_expandable: is_child_expandable,
                        is_expanded: is_child_expanded,
                        value_type,
                    });

                    if is_child_expanded && is_child_expandable {
                        build_json_lines(lines, val, &child_path, indent + 1, expanded);
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            let is_expanded = expanded.contains(path);
            let count = arr.len();
            if indent == 0 {
                lines.push(JsonLine {
                    path: path.to_string(),
                    indent,
                    content: format!("[] ({} items)", count),
                    is_expandable: !arr.is_empty(),
                    is_expanded,
                    value_type: JsonValueType::Array,
                });
            }
            if is_expanded || indent == 0 {
                for (i, val) in arr.iter().enumerate() {
                    let child_path = format!("{}[{}]", path, i);
                    let is_child_expandable = matches!(
                        val,
                        serde_json::Value::Object(_) | serde_json::Value::Array(_)
                    );
                    let is_child_expanded = expanded.contains(&child_path);

                    let content = match val {
                        serde_json::Value::Object(m) => format!("[{}]: {{}} ({} keys)", i, m.len()),
                        serde_json::Value::Array(a) => format!("[{}]: [] ({} items)", i, a.len()),
                        serde_json::Value::String(s) => {
                            format!("[{}]: \"{}\"", i, truncate_json_string(s, 50))
                        }
                        serde_json::Value::Number(n) => format!("[{}]: {}", i, n),
                        serde_json::Value::Bool(b) => format!("[{}]: {}", i, b),
                        serde_json::Value::Null => format!("[{}]: null", i),
                    };

                    let value_type = match val {
                        serde_json::Value::Object(_) => JsonValueType::Object,
                        serde_json::Value::Array(_) => JsonValueType::Array,
                        serde_json::Value::String(_) => JsonValueType::String,
                        serde_json::Value::Number(_) => JsonValueType::Number,
                        serde_json::Value::Bool(_) => JsonValueType::Bool,
                        serde_json::Value::Null => JsonValueType::Null,
                    };

                    lines.push(JsonLine {
                        path: child_path.clone(),
                        indent: indent + 1,
                        content,
                        is_expandable: is_child_expandable,
                        is_expanded: is_child_expanded,
                        value_type,
                    });

                    if is_child_expanded && is_child_expandable {
                        build_json_lines(lines, val, &child_path, indent + 1, expanded);
                    }
                }
            }
        }
        _ => {
            // Primitive value at root
            let content = match value {
                serde_json::Value::String(s) => format!("\"{}\"", s),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "null".to_string(),
                _ => value.to_string(),
            };
            let value_type = match value {
                serde_json::Value::String(_) => JsonValueType::String,
                serde_json::Value::Number(_) => JsonValueType::Number,
                serde_json::Value::Bool(_) => JsonValueType::Bool,
                serde_json::Value::Null => JsonValueType::Null,
                _ => JsonValueType::Null,
            };
            lines.push(JsonLine {
                path: path.to_string(),
                indent,
                content,
                is_expandable: false,
                is_expanded: false,
                value_type,
            });
        }
    }
}

fn truncate_json_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    } else {
        format!(
            "{}...",
            &s[..max_len]
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t")
        )
    }
}

fn get_detail_json_str(app: &App) -> Option<String> {
    if app.show_row_detail {
        return app.row_detail_json.clone();
    }
    app.get_selected_cell().map(|(_, v)| v.to_string())
}

pub fn get_json_line_count(app: &App) -> usize {
    let cell_value = match get_detail_json_str(app) {
        Some(v) => v,
        None => return 0,
    };

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&cell_value) {
        let mut lines = Vec::new();
        build_json_lines(&mut lines, &json, "$", 0, &app.json_expanded);
        lines.len()
    } else {
        0
    }
}

pub fn get_selected_json_path(app: &App) -> Option<String> {
    let cell_value = get_detail_json_str(app)?;

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&cell_value) {
        let mut lines = Vec::new();
        build_json_lines(&mut lines, &json, "$", 0, &app.json_expanded);
        lines.get(app.json_selected).map(|l| l.path.clone())
    } else {
        None
    }
}

fn build_row_spans(
    cells: &[String],
    widths: &[usize],
    is_header: bool,
    is_selected_row: Option<bool>,
    selected_col: Option<usize>,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));

    for (i, cell) in cells.iter().enumerate() {
        let width = widths.get(i).copied().unwrap_or(10);
        let truncated = truncate_cell(cell, width);
        let padded = format!(" {:<width$} ", truncated, width = width);

        let is_selected_cell = selected_col == Some(i);

        let style = if is_selected_cell && !is_header {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_header {
            let mut s = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD);
            if is_selected_cell {
                s = s.add_modifier(Modifier::UNDERLINED);
            }
            s
        } else if is_selected_row == Some(true) {
            Style::default().fg(Color::White).bg(Color::DarkGray)
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
