use clap::Parser;
use pogodoro::args::Cli;
use pogodoro::db;
use pogodoro::event::{Event, EventHandler};
use pogodoro::states::{App, AppResult, AppState};
use pogodoro::tui::Tui;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Read command line args
    let args = Cli::parse();
    // Create an application.
    let mut app = App::new(args.command).await;
    // TODO: investigate automatic setup of db
    db::setup().await.unwrap();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    loop {
        if let AppState::Finished = app.state {
            break;
        }
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.

        match tui.events.next()? {
            Event::Tick => app.state.tick(),
            Event::Key(key_event) => app.handle_key_event(key_event).await,
            _ => {}
        }
    }
    app.write_db().await?;
    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
