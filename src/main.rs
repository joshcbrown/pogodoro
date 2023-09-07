use clap::Parser;
use flexi_logger::{FileSpec, Logger};
use pogodoro::{
    args::Cli,
    db,
    event::{Event, EventHandler},
    states::{parse_args, AppResult},
    tui::Tui,
};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> AppResult<()> {
    // Read command line args
    let args = Cli::parse();
    // Create an application.
    let state = parse_args(args.command).await?;
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
        if state.should_finish() {
            break;
        }
        // Render the user interface.
        tui.draw(&mut state)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => state.tick().await?,
            Event::Key(key_event) => state = state.handle_key_event(key_event).await?,
            _ => {}
        };
    }
    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
