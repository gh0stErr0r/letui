use std::{env, io, path::PathBuf, time::Duration};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod file;
mod lsp;
mod ui;

include!(concat!(env!("OUT_DIR"), "/abbreviations.rs"));

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args_os().nth(1).map(PathBuf::from);
    let mut app = app::App::new(path)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;
        if app.should_quit {
            break;
        }
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
