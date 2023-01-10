use crate::pomodoro::{Pomodoro, PomodoroState};
use std::error;
use std::time::Duration;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
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
                Duration::from_secs(5),
                Duration::from_secs(5),
                Duration::from_secs(5),
            ),
        }
    }
}

const POMO_HEIGHT: u16 = 6;
const POMO_WIDTH: u16 = 25;

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(ops: Option<Vec<u64>>) -> AppResult<Self> {
        match ops {
            Some(vec) => {
                if let [work_dur, short_break_dur, long_break_dur] = &vec[..] {
                    Ok(Self {
                        running: true,
                        pomo: Pomodoro::new(
                            Duration::from_secs(work_dur * 60),
                            Duration::from_secs(short_break_dur * 60),
                            Duration::from_secs(long_break_dur * 60),
                        ),
                    })
                } else {
                    Err("expected vector of length 3".into())
                }
            }
            None => Ok(Self::default()),
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.pomo.update()
    }

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        // This is where you add new widgets.
        // See the following resources:
        // - https://docs.rs/tui/latest/tui/widgets/index.html
        // - https://github.com/fdehau/tui-rs/tree/master/examples

        let frame_rect = frame.size();
        let vert_buffer = frame_rect.height.checked_sub(POMO_HEIGHT).unwrap_or(0) / 2;
        let hor_buffer = frame_rect.width.checked_sub(POMO_WIDTH).unwrap_or(0) / 2;

        let vert_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(vert_buffer),
                Constraint::Length(POMO_HEIGHT),
                Constraint::Min(vert_buffer),
            ])
            .split(frame.size());

        let hor_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(hor_buffer),
                Constraint::Length(POMO_WIDTH),
                Constraint::Min(hor_buffer),
            ])
            .split(vert_chunks[1]);

        let pomo_widget = Paragraph::new(format!(
            "Remaining: {}\nFinished: {}\n\n[q]uit",
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
        .alignment(Alignment::Center);

        frame.render_widget(pomo_widget, hor_chunks[1]);
    }
}
