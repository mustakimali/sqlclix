# SQLClix

A SQL database browser with TUI interface supporting SQLite and PostgreSQL.

<img width="3840" height="2160" alt="Query with multiple tabs" src="https://github.com/user-attachments/assets/fe672c0a-1d28-473e-9d56-defa402180e6" />

<img width="3840" height="2160" alt="JSON view" src="https://github.com/user-attachments/assets/b9fc49b8-a331-4d85-9081-d340b596735d" />

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

## Session State

SQLClix saves your tabs and active tab between sessions. State is stored in a SQLite database at:

- **Linux:** `~/.cache/sqlclix/state.db`
- **macOS:** `~/Library/Caches/sqlclix/state.db`

Database paths are stored as SHA-256 hashes, so connection strings containing credentials are never saved in plaintext.

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
