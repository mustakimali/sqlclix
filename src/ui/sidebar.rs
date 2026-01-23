use crate::app::{App, SidebarSection};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let mut current_section: Option<SidebarSection> = None;
    let mut item_index = 0;
    let visible_height = area.height as usize;

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if app.sidebar_selected >= app.sidebar_scroll + visible_height {
        app.sidebar_selected.saturating_sub(visible_height - 1)
    } else if app.sidebar_selected < app.sidebar_scroll {
        app.sidebar_selected
    } else {
        app.sidebar_scroll
    };

    for (idx, item) in app.sidebar_items.iter().enumerate() {
        // Section header
        if current_section != Some(item.section) {
            current_section = Some(item.section);
            let section_name = match item.section {
                SidebarSection::Tables => "TABLES",
                SidebarSection::Views => "VIEWS",
                SidebarSection::Indexes => "INDEXES",
            };

            if item_index > 0 {
                lines.push(Line::from(""));
                item_index += 1;
            }

            lines.push(Line::from(Span::styled(
                section_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            item_index += 1;
        }

        // Item
        let is_selected = idx == app.sidebar_selected;
        let prefix = if item.children.is_empty() {
            "  "
        } else if item.is_expanded {
            "▾ "
        } else {
            "▸ "
        };

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let name = if item.name.len() > area.width as usize - 4 {
            format!("{}…", &item.name[..area.width as usize - 5])
        } else {
            item.name.clone()
        };

        lines.push(Line::from(Span::styled(format!("{}{}", prefix, name), style)));
        item_index += 1;

        // Expanded children (columns)
        if item.is_expanded {
            for child in &item.children {
                let child_style = Style::default().fg(Color::DarkGray);
                let child_text = if child.len() > area.width as usize - 6 {
                    format!("    {}…", &child[..area.width as usize - 7])
                } else {
                    format!("    {}", child)
                };
                lines.push(Line::from(Span::styled(child_text, child_style)));
                item_index += 1;
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No tables found",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines).scroll((scroll_offset as u16, 0));
    frame.render_widget(paragraph, area);
}
