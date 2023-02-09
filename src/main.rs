use clap::Parser;
use pogodoro::{
    args::Cli,
    db,
    event::{Event, EventHandler},
    states::{AppResult, AppState},
    tui::Tui,
};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> AppResult<()> {
    // Read command line args
    let args = Cli::parse();
    // Create an application.
    let state = AppState::parse_args(args.command).await;
    if state.is_none() {
        return Ok(());
    }
    let mut state = state.unwrap();

    db::setup().await.unwrap();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    loop {
        if let AppState::Finished = state {
            break;
        }
        // Render the user interface.
        tui.draw(&mut state)?;
        // Handle events.

        match tui.events.next()? {
            Event::Tick => state.tick().await,
            Event::Key(key_event) => state.handle_key_event(key_event).await,
            _ => {}
        }
    }
    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
