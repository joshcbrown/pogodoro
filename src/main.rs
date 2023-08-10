use clap::Parser;
use flexi_logger::{FileSpec, Logger};
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
    let state = AppState::parse_args(args.command).await?;
    if state.is_none() {
        return Ok(());
    }
    let mut state = state.unwrap();

    Logger::try_with_env()?
        .log_to_file(FileSpec::default())
        .print_message()
        .start()?;
    db::setup().await?;

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
        let result = match tui.events.next()? {
            Event::Tick => state.tick().await,
            Event::Key(key_event) => state.handle_key_event(key_event).await,
            _ => Ok(()),
        };
        if let Err(e) = result {
            tui.exit()?;
            println!("Application error: {e}");
            std::process::exit(1);
        }
    }
    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
