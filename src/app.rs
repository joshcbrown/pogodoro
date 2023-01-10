use crate::args::{Commands, Start};
use crate::states::WorkingState;
use std::error;
use std::io::Stderr;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::terminal::Frame;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub trait AppState {
    fn render(&mut self, frame: &mut Frame<'_, CrosstermBackend<Stderr>>);
    fn update(&mut self);
}

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    state: Box<dyn AppState>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            state: Box::new(WorkingState::new(
                Duration::from_secs(5),
                Duration::from_secs(5),
                Duration::from_secs(5),
            )),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(ops: Option<Commands>) -> AppResult<Self> {
        match ops {
            Some(Commands::Start(Start {
                work_dur,
                short_break_dur,
                long_break_dur,
            })) => Ok(Self {
                running: true,
                state: Box::new(WorkingState::new(
                    Duration::from_secs(work_dur * 60),
                    Duration::from_secs(short_break_dur * 60),
                    Duration::from_secs(long_break_dur * 60),
                )),
            }),
            None => Ok(Self::default()),
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.state.update()
    }

    /// Renders the user interface widgets.
    pub fn render(&mut self, frame: &mut Frame<'_, CrosstermBackend<Stderr>>) {
        self.state.render(frame)
    }
}
