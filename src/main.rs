use clap::Parser;
use pogodoro::args::Cli;
use pogodoro::event::{Event, EventHandler};
use pogodoro::states::{AppResult, AppState};
use pogodoro::tui::Tui;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Read command line args
    let args = Cli::parse();
    // Create an application.
    let mut app = AppState::new(args.command);
    AppState::setup_db().await.unwrap();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    loop {
        if let AppState::Finished = app {
            break;
        }
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.

        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => app.handle_key_event(key_event),
            _ => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
