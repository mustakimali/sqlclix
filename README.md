# SQLClix

A SQL database browser with TUI interface supporting SQLite and PostgreSQL.

## Install

```bash
make publish  # builds and copies to ~/bin/
```

## Usage

```bash
# SQLite
sqlclix database.db

# PostgreSQL
sqlclix "postgres://user:password@localhost:5432/dbname"
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Switch panels |
| `F5` / `Ctrl+R` | Execute query |
| `F2` | New tab |
| `Ctrl+W` | Close tab |
| `Enter` | Select/expand table, view cell detail |
| `s/c/d` | Generate SELECT/COUNT/DESCRIBE query |
| `?` | Help |
| `q` | Quit |
