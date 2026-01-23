mod app;
mod db;
mod events;
mod highlight;
mod ui;

use anyhow::{Context, Result};
use app::App;
use clap::Parser;
use crossterm::{
    event::Event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use db::Database;
use ratatui::prelude::*;
use std::io::stdout;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "sqlitex")]
#[command(author, version, about = "A SQLite database browser with TUI interface")]
struct Cli {
    /// Path to the SQLite database file
    #[arg(value_name = "DATABASE")]
    database: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Open database
    let db = Database::open(&cli.database)
        .with_context(|| format!("Failed to open database: {}", cli.database))?;

    // Create app
    let mut app = App::new(db)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Run app
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    while app.running {
        // Draw
        terminal.draw(|frame| ui::render(frame, app))?;

        // Handle events
        if let Some(event) = events::poll_event(Duration::from_millis(100))? {
            match event {
                Event::Key(key) => events::handle_key_event(app, key),
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}
