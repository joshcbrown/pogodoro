use crate::pomodoro::{Pomodoro, PomodoroState};
use std::error;
use std::time::Duration;
use tui::backend::Backend;
use tui::layout::Alignment;
use tui::terminal::Frame;
use tui::widgets::{Block, BorderType, Borders, Paragraph};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    pomo: Pomodoro,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            pomo: Pomodoro::new(
                Duration::from_secs(20 * 60),
                Duration::from_secs(5 * 60),
                Duration::from_secs(15 * 60),
            ),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(ops: Option<[u64; 3]>) -> Self {
        match ops {
            Some([work_dur, short_break_dur, long_break_dur]) => Self {
                running: true,
                pomo: Pomodoro::new(
                    Duration::from_secs(work_dur * 60),
                    Duration::from_secs(short_break_dur * 60),
                    Duration::from_secs(long_break_dur * 60),
                ),
            },
            None => Self::default(),
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        if let PomodoroState::LongBreak = self.pomo.state() {
            self.running = false
        }
        self.pomo.update()
    }

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        // This is where you add new widgets.
        // See the following resources:
        // - https://docs.rs/tui/latest/tui/widgets/index.html
        // - https://github.com/fdehau/tui-rs/tree/master/examples
        frame.render_widget(
            Paragraph::new(format!(
                "Timer: {}\nFinished: {}\n\n[q]uit",
                self.pomo.current,
                self.pomo.pomos_completed()
            ))
            .block(
                Block::default()
                    .title(format!("Pogodoro — {}", self.pomo.state()))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(self.pomo.style()),
            )
            .alignment(Alignment::Center),
            frame.size(),
        )
    }
}
