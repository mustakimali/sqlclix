use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::cell::RefCell;
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

// Trait for database operations
trait DatabaseConnection {
    fn load_schema(&self) -> Result<Schema>;
    fn execute_query(&self, sql: &str) -> QueryResult;
    fn get_display_name(&self) -> String;
}

// SQLite implementation
pub struct SqliteDatabase {
    conn: Connection,
    path: String,
}

impl SqliteDatabase {
    fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
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

impl DatabaseConnection for SqliteDatabase {
    fn load_schema(&self) -> Result<Schema> {
        let tables = self.load_tables("table")?;
        let views = self.load_tables("view")?;
        let indexes = self.load_indexes()?;

        Ok(Schema {
            tables,
            views,
            indexes,
        })
    }

    fn execute_query(&self, sql: &str) -> QueryResult {
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

    fn get_display_name(&self) -> String {
        self.path.clone()
    }
}

// PostgreSQL implementation
pub struct PostgresDatabase {
    client: RefCell<postgres::Client>,
    connection_string: String,
}

impl PostgresDatabase {
    fn open(connection_string: &str) -> Result<Self> {
        let client = postgres::Client::connect(connection_string, postgres::NoTls)
            .with_context(|| format!("Failed to connect to PostgreSQL: {}", connection_string))?;

        Ok(Self {
            client: RefCell::new(client),
            connection_string: connection_string.to_string(),
        })
    }

    fn load_tables(&self, is_view: bool) -> Result<Vec<TableInfo>> {
        let table_type = if is_view { "VIEW" } else { "BASE TABLE" };
        let query = format!(
            "SELECT table_name FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_type = '{}' \
             ORDER BY table_name",
            table_type
        );

        let rows = self.client.borrow_mut().query(&query, &[])?;
        let mut tables = Vec::new();

        for row in rows {
            let name: String = row.get(0);
            let columns = self.load_columns(&name)?;
            tables.push(TableInfo { name, columns });
        }

        Ok(tables)
    }

    fn load_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let query = "SELECT column_name, data_type, is_nullable \
                     FROM information_schema.columns \
                     WHERE table_schema = 'public' AND table_name = $1 \
                     ORDER BY ordinal_position";

        let rows = self.client.borrow_mut().query(query, &[&table_name])?;

        // Load primary key information
        let pk_query = "SELECT a.attname \
                        FROM pg_index i \
                        JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey) \
                        WHERE i.indrelid = $1::regclass AND i.indisprimary";

        let pk_rows = self
            .client
            .borrow_mut()
            .query(pk_query, &[&table_name])
            .unwrap_or_default();
        let primary_keys: std::collections::HashSet<String> = pk_rows
            .iter()
            .filter_map(|row| row.get::<_, Option<String>>(0))
            .collect();

        let columns = rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let data_type: String = row.get(1);
                let is_nullable_str: String = row.get(2);
                let is_nullable = is_nullable_str == "YES";
                let is_primary_key = primary_keys.contains(&name);

                ColumnInfo {
                    name,
                    data_type,
                    is_nullable,
                    is_primary_key,
                }
            })
            .collect();

        Ok(columns)
    }

    fn load_indexes(&self) -> Result<Vec<IndexInfo>> {
        let query = "SELECT indexname, tablename, indexdef \
                     FROM pg_indexes \
                     WHERE schemaname = 'public' \
                     ORDER BY indexname";

        let rows = self.client.borrow_mut().query(query, &[])?;
        let indexes = rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let table_name: String = row.get(1);
                let indexdef: String = row.get(2);
                let is_unique = indexdef.to_uppercase().contains("UNIQUE");

                IndexInfo {
                    name,
                    table_name,
                    is_unique,
                }
            })
            .collect();

        Ok(indexes)
    }

    fn execute_query_inner(&self, sql: &str) -> Result<QueryResult> {
        let rows = self.client.borrow_mut().query(sql, &[])?;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time: Duration::ZERO,
                error: None,
            });
        }

        let columns: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect();

        let mut result_rows = Vec::new();
        for row in &rows {
            let mut row_data = Vec::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value = Self::get_column_value(&row, i, column.type_());
                row_data.push(value);
            }
            result_rows.push(row_data);
        }

        let row_count = result_rows.len();

        Ok(QueryResult {
            columns,
            rows: result_rows,
            row_count,
            execution_time: Duration::ZERO,
            error: None,
        })
    }

    fn get_column_value(
        row: &postgres::Row,
        idx: usize,
        col_type: &postgres::types::Type,
    ) -> String {
        use postgres::types::Type;

        // Try to get the value based on the column type
        match *col_type {
            Type::BOOL => row
                .get::<_, Option<bool>>(idx)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
            Type::INT2 => row
                .get::<_, Option<i16>>(idx)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
            Type::INT4 => row
                .get::<_, Option<i32>>(idx)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
            Type::INT8 => row
                .get::<_, Option<i64>>(idx)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
            Type::FLOAT4 => row
                .get::<_, Option<f32>>(idx)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
            Type::FLOAT8 => row
                .get::<_, Option<f64>>(idx)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
            Type::VARCHAR | Type::TEXT | Type::BPCHAR | Type::NAME => row
                .get::<_, Option<String>>(idx)
                .unwrap_or_else(|| "NULL".to_string()),
            _ => {
                // For other types, try to get as string, fallback to NULL
                row.get::<_, Option<String>>(idx)
                    .unwrap_or_else(|| "NULL".to_string())
            }
        }
    }
}

impl DatabaseConnection for PostgresDatabase {
    fn load_schema(&self) -> Result<Schema> {
        let tables = self.load_tables(false)?;
        let views = self.load_tables(true)?;
        let indexes = self.load_indexes()?;

        Ok(Schema {
            tables,
            views,
            indexes,
        })
    }

    fn execute_query(&self, sql: &str) -> QueryResult {
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

    fn get_display_name(&self) -> String {
        self.connection_string.clone()
    }
}

// Public enum wrapper
pub enum Database {
    Sqlite(SqliteDatabase),
    Postgres(PostgresDatabase),
}

impl Database {
    pub fn open(connection_string: &str) -> Result<Self> {
        // Auto-detect database type
        if connection_string.starts_with("postgres://")
            || connection_string.starts_with("postgresql://")
            || connection_string.contains("host=")
        {
            Ok(Database::Postgres(PostgresDatabase::open(
                connection_string,
            )?))
        } else {
            Ok(Database::Sqlite(SqliteDatabase::open(connection_string)?))
        }
    }

    pub fn load_schema(&self) -> Result<Schema> {
        match self {
            Database::Sqlite(db) => db.load_schema(),
            Database::Postgres(db) => db.load_schema(),
        }
    }

    pub fn execute_query(&self, sql: &str) -> QueryResult {
        match self {
            Database::Sqlite(db) => db.execute_query(sql),
            Database::Postgres(db) => db.execute_query(sql),
        }
    }

    pub fn get_display_name(&self) -> String {
        match self {
            Database::Sqlite(db) => db.get_display_name(),
            Database::Postgres(db) => db.get_display_name(),
        }
    }

    pub fn path(&self) -> String {
        self.get_display_name()
    }
}
