use crate::db::{Database, QueryResult, Schema};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Sidebar,
    Editor,
    Results,
}

impl Panel {
    pub fn next(self) -> Self {
        match self {
            Panel::Sidebar => Panel::Editor,
            Panel::Editor => Panel::Results,
            Panel::Results => Panel::Sidebar,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Panel::Sidebar => Panel::Results,
            Panel::Editor => Panel::Sidebar,
            Panel::Results => Panel::Editor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarSection {
    Tables,
    Views,
    Indexes,
}

#[derive(Debug, Clone)]
pub struct SidebarItem {
    pub name: String,
    pub section: SidebarSection,
    pub is_expanded: bool,
    pub children: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EditorTab {
    pub name: String,
    pub content: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
}

impl EditorTab {
    pub fn new(name: String) -> Self {
        Self {
            name,
            content: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
        }
    }

    pub fn get_text(&self) -> String {
        self.content.join("\n")
    }

    pub fn set_text(&mut self, text: &str) {
        self.content = text.lines().map(String::from).collect();
        if self.content.is_empty() {
            self.content.push(String::new());
        }
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    pub fn insert_char(&mut self, ch: char) {
        if ch == '\n' {
            let current_line = &self.content[self.cursor_line];
            let (before, after) = current_line.split_at(self.cursor_col.min(current_line.len()));
            let before = before.to_string();
            let after = after.to_string();
            self.content[self.cursor_line] = before;
            self.content.insert(self.cursor_line + 1, after);
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            let line = &mut self.content[self.cursor_line];
            let pos = self.cursor_col.min(line.len());
            line.insert(pos, ch);
            self.cursor_col = pos + 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.content[self.cursor_line];
            let pos = self.cursor_col.min(line.len());
            if pos > 0 {
                line.remove(pos - 1);
                self.cursor_col = pos - 1;
            }
        } else if self.cursor_line > 0 {
            let current_line = self.content.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.content[self.cursor_line].len();
            self.content[self.cursor_line].push_str(&current_line);
        }
    }

    pub fn delete(&mut self) {
        let line = &mut self.content[self.cursor_line];
        if self.cursor_col < line.len() {
            line.remove(self.cursor_col);
        } else if self.cursor_line < self.content.len() - 1 {
            let next_line = self.content.remove(self.cursor_line + 1);
            self.content[self.cursor_line].push_str(&next_line);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.content[self.cursor_line].len();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.content[self.cursor_line].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line < self.content.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let line_len = self.content[self.cursor_line].len();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line < self.content.len() - 1 {
            self.cursor_line += 1;
            let line_len = self.content[self.cursor_line].len();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_col = self.content[self.cursor_line].len();
    }

    pub fn ensure_cursor_visible(&mut self, visible_lines: usize) {
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.cursor_line - visible_lines + 1;
        }
    }
}

pub struct App {
    pub db: Database,
    pub schema: Schema,
    pub running: bool,
    pub focus: Panel,
    pub show_help: bool,

    // Sidebar state
    pub sidebar_items: Vec<SidebarItem>,
    pub sidebar_selected: usize,
    pub sidebar_scroll: usize,

    // Editor state
    pub tabs: Vec<EditorTab>,
    pub active_tab: usize,

    // Results state
    pub result: Option<QueryResult>,
    pub result_page: usize,
    pub result_scroll: usize,
    pub result_selected_row: usize,
    pub result_selected_col: usize,
    pub page_size: usize,
    pub show_cell_detail: bool,

    // JSON viewer state
    pub json_expanded: HashSet<String>,
    pub json_selected: usize,
    pub json_scroll: usize,
}

impl App {
    pub fn new(db: Database) -> anyhow::Result<Self> {
        let schema = db.load_schema()?;
        let sidebar_items = Self::build_sidebar_items(&schema);

        let mut app = Self {
            db,
            schema,
            running: true,
            focus: Panel::Sidebar,
            show_help: false,
            sidebar_items,
            sidebar_selected: 0,
            sidebar_scroll: 0,
            tabs: vec![EditorTab::new("Query 1".to_string())],
            active_tab: 0,
            result: None,
            result_page: 0,
            result_scroll: 0,
            result_selected_row: 0,
            result_selected_col: 0,
            page_size: 100,
            show_cell_detail: false,
            json_expanded: HashSet::new(),
            json_selected: 0,
            json_scroll: 0,
        };

        if !app.sidebar_items.is_empty() {
            app.sidebar_items[0].is_expanded = true;
        }

        Ok(app)
    }

    fn build_sidebar_items(schema: &Schema) -> Vec<SidebarItem> {
        let mut items = Vec::new();

        for table in &schema.tables {
            items.push(SidebarItem {
                name: table.name.clone(),
                section: SidebarSection::Tables,
                is_expanded: false,
                children: table
                    .columns
                    .iter()
                    .map(|c| {
                        let pk = if c.is_primary_key { " PK" } else { "" };
                        let null = if c.is_nullable { "?" } else { "" };
                        format!("{}: {}{}{}", c.name, c.data_type, null, pk)
                    })
                    .collect(),
            });
        }

        for view in &schema.views {
            items.push(SidebarItem {
                name: view.name.clone(),
                section: SidebarSection::Views,
                is_expanded: false,
                children: view
                    .columns
                    .iter()
                    .map(|c| format!("{}: {}", c.name, c.data_type))
                    .collect(),
            });
        }

        for index in &schema.indexes {
            let unique = if index.is_unique { " UNIQUE" } else { "" };
            items.push(SidebarItem {
                name: format!("{}{} ({})", index.name, unique, index.table_name),
                section: SidebarSection::Indexes,
                is_expanded: false,
                children: vec![],
            });
        }

        items
    }

    pub fn current_tab(&self) -> &EditorTab {
        &self.tabs[self.active_tab]
    }

    pub fn current_tab_mut(&mut self) -> &mut EditorTab {
        &mut self.tabs[self.active_tab]
    }

    pub fn new_tab(&mut self) {
        let num = self.tabs.len() + 1;
        self.tabs.push(EditorTab::new(format!("Query {}", num)));
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = if self.active_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab - 1
        };
    }

    pub fn execute_query(&mut self) {
        let sql = self.current_tab().get_text();
        let result = self.db.execute_query(&sql);
        self.result = Some(result);
        self.result_page = 0;
        self.result_scroll = 0;
        self.result_selected_row = 0;
        self.result_selected_col = 0;
        self.focus = Panel::Results;
    }

    pub fn toggle_sidebar_expand(&mut self) {
        if self.sidebar_selected < self.sidebar_items.len() {
            let item = &mut self.sidebar_items[self.sidebar_selected];
            if !item.children.is_empty() {
                item.is_expanded = !item.is_expanded;
            }
        }
    }

    pub fn generate_select_query(&mut self) {
        if let Some(item) = self.sidebar_items.get(self.sidebar_selected) {
            if item.section != SidebarSection::Indexes {
                let query = format!("SELECT * FROM \"{}\" LIMIT 100;", item.name);
                self.new_tab_with_query(&item.name.clone(), &query);
            }
        }
    }

    pub fn generate_count_query(&mut self) {
        if let Some(item) = self.sidebar_items.get(self.sidebar_selected) {
            if item.section != SidebarSection::Indexes {
                let query = format!("SELECT COUNT(*) FROM \"{}\";", item.name);
                self.new_tab_with_query(&item.name.clone(), &query);
            }
        }
    }

    pub fn generate_schema_query(&mut self) {
        if let Some(item) = self.sidebar_items.get(self.sidebar_selected) {
            if item.section != SidebarSection::Indexes {
                let query = format!("PRAGMA table_info(\"{}\");", item.name);
                self.new_tab_with_query(&item.name.clone(), &query);
            }
        }
    }

    fn new_tab_with_query(&mut self, name: &str, query: &str) {
        let mut tab = EditorTab::new(name.to_string());
        tab.set_text(query);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.focus = Panel::Editor;
    }

    pub fn sidebar_up(&mut self) {
        if self.sidebar_selected > 0 {
            self.sidebar_selected -= 1;
        }
    }

    pub fn sidebar_down(&mut self) {
        if self.sidebar_selected < self.sidebar_items.len().saturating_sub(1) {
            self.sidebar_selected += 1;
        }
    }

    pub fn result_page_count(&self) -> usize {
        match &self.result {
            Some(r) if !r.rows.is_empty() => (r.rows.len() + self.page_size - 1) / self.page_size,
            _ => 1,
        }
    }

    pub fn next_result_page(&mut self) {
        let max_page = self.result_page_count().saturating_sub(1);
        if self.result_page < max_page {
            self.result_page += 1;
            self.result_scroll = 0;
        }
    }

    pub fn prev_result_page(&mut self) {
        if self.result_page > 0 {
            self.result_page -= 1;
            self.result_scroll = 0;
        }
    }

    pub fn get_current_page_rows<'a>(&self, result: &'a QueryResult) -> &'a [Vec<String>] {
        let start = self.result_page * self.page_size;
        let end = (start + self.page_size).min(result.rows.len());
        if start < result.rows.len() {
            &result.rows[start..end]
        } else {
            &[]
        }
    }

    pub fn refresh_schema(&mut self) -> anyhow::Result<()> {
        self.schema = self.db.load_schema()?;
        self.sidebar_items = Self::build_sidebar_items(&self.schema);
        self.sidebar_selected = 0;
        Ok(())
    }

    pub fn result_move_up(&mut self) {
        if self.result_selected_row > 0 {
            self.result_selected_row -= 1;
        }
    }

    pub fn result_move_down(&mut self) {
        if let Some(result) = &self.result {
            let page_rows = self.get_current_page_rows(result);
            if self.result_selected_row < page_rows.len().saturating_sub(1) {
                self.result_selected_row += 1;
            }
        }
    }

    pub fn result_move_left(&mut self) {
        if self.result_selected_col > 0 {
            self.result_selected_col -= 1;
        }
    }

    pub fn result_move_right(&mut self) {
        if let Some(result) = &self.result {
            if self.result_selected_col < result.columns.len().saturating_sub(1) {
                self.result_selected_col += 1;
            }
        }
    }

    pub fn get_selected_cell(&self) -> Option<(&str, &str)> {
        let result = self.result.as_ref()?;
        if result.columns.is_empty() || result.rows.is_empty() {
            return None;
        }
        let page_rows = self.get_current_page_rows(result);
        let row = page_rows.get(self.result_selected_row)?;
        let col_name = result.columns.get(self.result_selected_col)?;
        let cell_value = row.get(self.result_selected_col)?;
        Some((col_name.as_str(), cell_value.as_str()))
    }

    pub fn toggle_cell_detail(&mut self) {
        if self.get_selected_cell().is_some() {
            if !self.show_cell_detail {
                // Reset JSON viewer state when opening
                self.json_expanded.clear();
                self.json_selected = 0;
                self.json_scroll = 0;
                // Expand root by default
                self.json_expanded.insert("$".to_string());
            }
            self.show_cell_detail = !self.show_cell_detail;
        }
    }

    pub fn result_move_to_end(&mut self) {
        if let Some(result) = &self.result {
            let page_rows = self.get_current_page_rows(result);
            self.result_selected_row = page_rows.len().saturating_sub(1);
        }
    }

    pub fn json_toggle_expand(&mut self, path: &str) {
        if self.json_expanded.contains(path) {
            self.json_expanded.remove(path);
        } else {
            self.json_expanded.insert(path.to_string());
        }
    }

    pub fn json_move_up(&mut self) {
        if self.json_selected > 0 {
            self.json_selected -= 1;
        }
    }

    pub fn json_move_down(&mut self, max_lines: usize) {
        if self.json_selected < max_lines.saturating_sub(1) {
            self.json_selected += 1;
        }
    }
}
