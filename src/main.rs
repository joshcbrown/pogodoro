use clap::Parser;
use pogodoro::app::{App, AppResult};
use pogodoro::event::{Event, EventHandler};
use pogodoro::handler::handle_key_events;
use pogodoro::tui::Tui;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A poggers-as-hell terminal UI pomodoro timer"
)]
struct Args {
    /// Duration of each working session in minutes
    #[arg(short, long)]
    work_dur: Option<u64>,
    /// Duration of each short break in minutes
    #[arg(short, long)]
    short_break_dur: Option<u64>,
    /// Duration of each long break in minutes
    #[arg(short, long)]
    long_break_dur: Option<u64>,
}

fn main() -> AppResult<()> {
    // Read command line args
    let args = Args::parse();
    // Create an application.
    let vec: Option<Vec<u64>> = [args.work_dur, args.short_break_dur, args.long_break_dur]
        .into_iter()
        .collect();
    let mut app = App::new(vec).unwrap();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
