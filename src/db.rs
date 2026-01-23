use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub table_name: String,
    pub is_unique: bool,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub tables: Vec<TableInfo>,
    pub views: Vec<TableInfo>,
    pub indexes: Vec<IndexInfo>,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub execution_time: Duration,
    pub error: Option<String>,
}

impl QueryResult {
    pub fn error(msg: String, time: Duration) -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            row_count: 0,
            execution_time: time,
            error: Some(msg),
        }
    }
}

pub struct Database {
    conn: Connection,
    pub path: String,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .with_context(|| format!("Failed to open database: {}", path_str))?;

        Ok(Self {
            conn,
            path: path_str,
        })
    }

    pub fn load_schema(&self) -> Result<Schema> {
        let tables = self.load_tables("table")?;
        let views = self.load_tables("view")?;
        let indexes = self.load_indexes()?;

        Ok(Schema {
            tables,
            views,
            indexes,
        })
    }

    fn load_tables(&self, object_type: &str) -> Result<Vec<TableInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = ? AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;

        let names: Vec<String> = stmt
            .query_map([object_type], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut tables = Vec::new();
        for name in names {
            let columns = self.load_columns(&name)?;
            tables.push(TableInfo { name, columns });
        }

        Ok(tables)
    }

    fn load_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let sql = format!("PRAGMA table_info(\"{}\")", table_name.replace('"', "\"\""));
        let mut stmt = self.conn.prepare(&sql)?;

        let columns = stmt
            .query_map([], |row| {
                Ok(ColumnInfo {
                    name: row.get(1)?,
                    data_type: row.get::<_, String>(2).unwrap_or_default(),
                    is_nullable: row.get::<_, i32>(3).unwrap_or(1) == 0,
                    is_primary_key: row.get::<_, i32>(5).unwrap_or(0) != 0,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(columns)
    }

    fn load_indexes(&self) -> Result<Vec<IndexInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, tbl_name, sql FROM sqlite_master WHERE type = 'index' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;

        let indexes = stmt
            .query_map([], |row| {
                let sql: Option<String> = row.get(2).ok();
                let is_unique = sql
                    .map(|s| s.to_uppercase().contains("UNIQUE"))
                    .unwrap_or(false);
                Ok(IndexInfo {
                    name: row.get(0)?,
                    table_name: row.get(1)?,
                    is_unique,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(indexes)
    }

    pub fn execute_query(&self, sql: &str) -> QueryResult {
        let start = Instant::now();

        let trimmed = sql.trim();
        if trimmed.is_empty() {
            return QueryResult::error("Empty query".to_string(), start.elapsed());
        }

        let result = self.execute_query_inner(trimmed);
        let elapsed = start.elapsed();

        match result {
            Ok(mut qr) => {
                qr.execution_time = elapsed;
                qr
            }
            Err(e) => QueryResult::error(e.to_string(), elapsed),
        }
    }

    fn execute_query_inner(&self, sql: &str) -> Result<QueryResult> {
        let mut stmt = self.conn.prepare(sql)?;
        let column_count = stmt.column_count();
        let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let mut rows = Vec::new();
        let mut query_rows = stmt.query([])?;

        while let Some(row) = query_rows.next()? {
            let mut row_data = Vec::with_capacity(column_count);
            for i in 0..column_count {
                let value = Self::get_column_value(row, i);
                row_data.push(value);
            }
            rows.push(row_data);
        }

        let row_count = rows.len();

        Ok(QueryResult {
            columns,
            rows,
            row_count,
            execution_time: Duration::ZERO,
            error: None,
        })
    }

    fn get_column_value(row: &rusqlite::Row, idx: usize) -> String {
        use rusqlite::types::ValueRef;

        match row.get_ref(idx) {
            Ok(ValueRef::Null) => "NULL".to_string(),
            Ok(ValueRef::Integer(i)) => i.to_string(),
            Ok(ValueRef::Real(f)) => f.to_string(),
            Ok(ValueRef::Text(t)) => String::from_utf8_lossy(t).to_string(),
            Ok(ValueRef::Blob(b)) => format!("[BLOB {} bytes]", b.len()),
            Err(_) => "ERROR".to_string(),
        }
    }
}
