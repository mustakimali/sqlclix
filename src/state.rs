use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};

pub struct StateStore {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct SavedTab {
    pub name: String,
    pub content: String,
    #[expect(dead_code)]
    pub position: i32,
}

impl StateStore {
    pub fn open() -> Result<Self> {
        let state_path = Self::state_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&state_path)
            .with_context(|| format!("Failed to open state database: {:?}", state_path))?;

        // Create tables if they don't exist
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                db_path TEXT UNIQUE NOT NULL,
                active_tab INTEGER DEFAULT 0,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS tabs (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                position INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_tabs_session ON tabs(session_id);
            ",
        )?;

        Ok(Self { conn })
    }

    fn state_db_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .or_else(dirs::data_local_dir)
            .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
            .context("Could not determine cache directory")?;

        Ok(cache_dir.join("sqlclix").join("state.db"))
    }

    pub fn load_session(&self, db_path: &str) -> Result<Option<(Vec<SavedTab>, usize)>> {
        let canonical_path = Self::canonicalize_path(db_path);

        let mut stmt = self
            .conn
            .prepare("SELECT id, active_tab FROM sessions WHERE db_path = ?")?;

        let session: Option<(i64, i32)> = stmt
            .query_row([&canonical_path], |row| Ok((row.get(0)?, row.get(1)?)))
            .ok();

        let (session_id, active_tab) = match session {
            Some(s) => s,
            None => return Ok(None),
        };

        let mut stmt = self.conn.prepare(
            "SELECT name, content, position FROM tabs WHERE session_id = ? ORDER BY position",
        )?;

        let tabs: Vec<SavedTab> = stmt
            .query_map([session_id], |row| {
                Ok(SavedTab {
                    name: row.get(0)?,
                    content: row.get(1)?,
                    position: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        if tabs.is_empty() {
            Ok(None)
        } else {
            Ok(Some((tabs, active_tab as usize)))
        }
    }

    pub fn save_session(&self, db_path: &str, tabs: &[SavedTab], active_tab: usize) -> Result<()> {
        let canonical_path = Self::canonicalize_path(db_path);

        // Upsert session
        self.conn.execute(
            "INSERT INTO sessions (db_path, active_tab, updated_at)
             VALUES (?, ?, CURRENT_TIMESTAMP)
             ON CONFLICT(db_path) DO UPDATE SET
                active_tab = excluded.active_tab,
                updated_at = CURRENT_TIMESTAMP",
            params![&canonical_path, active_tab as i32],
        )?;

        // Get session id
        let session_id: i64 = self.conn.query_row(
            "SELECT id FROM sessions WHERE db_path = ?",
            [&canonical_path],
            |row| row.get(0),
        )?;

        // Delete old tabs
        self.conn
            .execute("DELETE FROM tabs WHERE session_id = ?", [session_id])?;

        // Insert new tabs
        let mut stmt = self.conn.prepare(
            "INSERT INTO tabs (session_id, name, content, position) VALUES (?, ?, ?, ?)",
        )?;

        for (i, tab) in tabs.iter().enumerate() {
            stmt.execute(params![session_id, &tab.name, &tab.content, i as i32])?;
        }

        Ok(())
    }

    fn canonicalize_path(path: &str) -> String {
        // Don't canonicalize PostgreSQL connection strings
        if path.starts_with("postgres://")
            || path.starts_with("postgresql://")
            || path.contains("host=")
        {
            return path.to_string();
        }

        // For file paths, try to canonicalize
        Path::new(path)
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string())
    }
}
