use crate::app::{App, Panel};
use crate::ui::results::{get_json_line_count, get_selected_json_path};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    // Handle cell detail / JSON viewer
    if app.show_cell_detail {
        handle_cell_detail_key(app, key);
        return;
    }

    // Global shortcuts (work in any panel)
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::F(1)) | (KeyModifiers::NONE, KeyCode::Char('?')) => {
            if app.focus != Panel::Editor || key.code == KeyCode::F(1) {
                app.show_help = !app.show_help;
                return;
            }
        }
        (KeyModifiers::NONE, KeyCode::Esc) => {
            if app.show_help {
                app.show_help = false;
                return;
            }
        }
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            app.running = false;
            return;
        }
        (KeyModifiers::NONE, KeyCode::Char('q')) => {
            if app.focus != Panel::Editor && !app.show_help {
                app.running = false;
                return;
            }
        }
        (KeyModifiers::NONE, KeyCode::F(5))
        | (KeyModifiers::CONTROL, KeyCode::Enter)
        | (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
            app.execute_query();
            return;
        }
        (KeyModifiers::NONE, KeyCode::F(2)) => {
            app.new_tab();
            app.focus = Panel::Editor;
            return;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
            app.close_tab();
            return;
        }
        (KeyModifiers::CONTROL, KeyCode::Tab)
        | (KeyModifiers::CONTROL, KeyCode::Right)
        | (KeyModifiers::ALT, KeyCode::Right) => {
            app.next_tab();
            return;
        }
        (KeyModifiers::CONTROL, KeyCode::Left) | (KeyModifiers::ALT, KeyCode::Left) => {
            app.prev_tab();
            return;
        }
        (KeyModifiers::NONE, KeyCode::Tab) => {
            if !app.show_help {
                app.focus = app.focus.next();
                return;
            }
        }
        (_, KeyCode::BackTab) => {
            // Shift+Tab or Ctrl+Shift+Tab
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.prev_tab();
            } else if !app.show_help {
                app.focus = app.focus.prev();
            }
            return;
        }
        (KeyModifiers::CONTROL, KeyCode::PageDown) => {
            app.next_result_page();
            return;
        }
        (KeyModifiers::CONTROL, KeyCode::PageUp) => {
            app.prev_result_page();
            return;
        }
        _ => {}
    }

    if app.show_help {
        return;
    }

    // Panel-specific shortcuts
    match app.focus {
        Panel::Sidebar => handle_sidebar_key(app, key),
        Panel::Editor => handle_editor_key(app, key),
        Panel::Results => handle_results_key(app, key),
    }
}

fn handle_sidebar_key(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::NONE, KeyCode::Char('k')) => {
            app.sidebar_up();
        }
        (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::NONE, KeyCode::Char('j')) => {
            app.sidebar_down();
        }
        (KeyModifiers::NONE, KeyCode::Enter) | (KeyModifiers::NONE, KeyCode::Right) => {
            let item = app.sidebar_items.get(app.sidebar_selected);
            if let Some(item) = item {
                if item.is_expanded || item.children.is_empty() {
                    app.generate_select_query();
                } else {
                    app.toggle_sidebar_expand();
                }
            }
        }
        (KeyModifiers::NONE, KeyCode::Left) => {
            if let Some(item) = app.sidebar_items.get(app.sidebar_selected)
                && item.is_expanded {
                    app.toggle_sidebar_expand();
                }
        }
        (KeyModifiers::NONE, KeyCode::Char(' ')) => {
            app.toggle_sidebar_expand();
        }
        (KeyModifiers::NONE, KeyCode::Char('s')) => {
            app.generate_select_query();
        }
        (KeyModifiers::NONE, KeyCode::Char('c')) => {
            app.generate_count_query();
        }
        (KeyModifiers::NONE, KeyCode::Char('d')) => {
            app.generate_schema_query();
        }
        (KeyModifiers::NONE, KeyCode::Char('r')) => {
            let _ = app.refresh_schema();
        }
        _ => {}
    }
}

fn handle_editor_key(app: &mut App, key: KeyEvent) {
    let tab = app.current_tab_mut();

    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Left) => tab.move_cursor_left(),
        (KeyModifiers::NONE, KeyCode::Right) => tab.move_cursor_right(),
        (KeyModifiers::NONE, KeyCode::Up) => tab.move_cursor_up(),
        (KeyModifiers::NONE, KeyCode::Down) => tab.move_cursor_down(),
        (KeyModifiers::NONE, KeyCode::Home) | (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
            tab.move_cursor_home()
        }
        (KeyModifiers::NONE, KeyCode::End) | (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
            tab.move_cursor_end()
        }
        (KeyModifiers::NONE, KeyCode::Backspace) => tab.backspace(),
        (KeyModifiers::NONE, KeyCode::Delete) => tab.delete(),
        (KeyModifiers::NONE, KeyCode::Enter) => tab.insert_char('\n'),
        (KeyModifiers::NONE, KeyCode::Char(c)) => tab.insert_char(c),
        (KeyModifiers::SHIFT, KeyCode::Char(c)) => tab.insert_char(c),
        (KeyModifiers::NONE, KeyCode::PageUp) => {
            for _ in 0..10 {
                tab.move_cursor_up();
            }
        }
        (KeyModifiers::NONE, KeyCode::PageDown) => {
            for _ in 0..10 {
                tab.move_cursor_down();
            }
        }
        _ => {}
    }
}

fn handle_results_key(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::NONE, KeyCode::Char('k')) => {
            app.result_move_up();
        }
        (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::NONE, KeyCode::Char('j')) => {
            app.result_move_down();
        }
        (KeyModifiers::NONE, KeyCode::Left) | (KeyModifiers::NONE, KeyCode::Char('h')) => {
            app.result_move_left();
        }
        (KeyModifiers::NONE, KeyCode::Right) | (KeyModifiers::NONE, KeyCode::Char('l')) => {
            app.result_move_right();
        }
        (KeyModifiers::NONE, KeyCode::Enter) => {
            app.toggle_cell_detail();
        }
        (KeyModifiers::NONE, KeyCode::PageUp) => {
            for _ in 0..10 {
                app.result_move_up();
            }
        }
        (KeyModifiers::NONE, KeyCode::PageDown) => {
            for _ in 0..10 {
                app.result_move_down();
            }
        }
        (KeyModifiers::NONE, KeyCode::Home) => {
            app.result_selected_row = 0;
            app.result_scroll = 0;
        }
        (KeyModifiers::NONE, KeyCode::End) => {
            app.result_move_to_end();
        }
        _ => {}
    }
}

fn handle_cell_detail_key(app: &mut App, key: KeyEvent) {
    let line_count = get_json_line_count(app);

    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Char('q')) => {
            app.show_cell_detail = false;
        }
        (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::NONE, KeyCode::Char('k')) => {
            app.json_move_up();
        }
        (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::NONE, KeyCode::Char('j')) => {
            app.json_move_down(line_count);
        }
        (KeyModifiers::NONE, KeyCode::Enter) | (KeyModifiers::NONE, KeyCode::Right) => {
            if let Some(path) = get_selected_json_path(app) {
                app.json_toggle_expand(&path);
            }
        }
        (KeyModifiers::NONE, KeyCode::Left) => {
            // Collapse current node
            if let Some(path) = get_selected_json_path(app)
                && app.json_expanded.contains(&path) {
                    app.json_expanded.remove(&path);
                }
        }
        (KeyModifiers::NONE, KeyCode::PageUp) => {
            for _ in 0..10 {
                app.json_move_up();
            }
        }
        (KeyModifiers::NONE, KeyCode::PageDown) => {
            for _ in 0..10 {
                app.json_move_down(line_count);
            }
        }
        (KeyModifiers::NONE, KeyCode::Home) => {
            app.json_selected = 0;
            app.json_scroll = 0;
        }
        (KeyModifiers::NONE, KeyCode::End) => {
            app.json_selected = line_count.saturating_sub(1);
        }
        _ => {}
    }
}
