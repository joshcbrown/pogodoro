use std::error;
use std::time::{Duration, Instant};
use tui::backend::Backend;
use tui::layout::Alignment;
use tui::style::{Color, Style};
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
                Duration::from_secs(2),
                Duration::from_secs(5),
                Duration::from_secs(10),
            ),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
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
        frame.render_widget(
            Paragraph::new(
                format!(
                "This is a tui-rs template.\nPress `Esc`, `Ctrl-C` or `q` to stop running.\nTimer: {}\nState: {:?}\nFinished: {}",
                self.pomo.current.to_string(),
                self.pomo.state,
                self.pomo.pomos_completed
                )
            )
            .block(
                Block::default()
                    .title("Template")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .alignment(Alignment::Center),
            frame.size(),
        )
    }
}

#[derive(Debug)]
struct Timer {
    start_time: Instant,
    dur: Duration,
    elapsed: Duration,
}

impl Timer {
    fn new(dur: Duration) -> Self {
        Self {
            dur,
            start_time: Instant::now(),
            elapsed: Duration::from_secs(0),
        }
    }

    fn update(&mut self) {
        // might check here later if timer is over
        self.elapsed = self.start_time.elapsed();
    }

    fn is_finished(&self) -> bool {
        // println!("{:#?} {:#?}", self.total_time, self.elapsed);
        self.elapsed >= self.dur
    }
    fn to_string(&self) -> String {
        if self.is_finished() {
            "finished!".into()
        } else {
            let to_go = (self.dur - self.elapsed + Duration::from_secs(1)).as_secs();
            format!("{}m{}s", to_go / 60, to_go % 60)
        }
    }
}

#[derive(Debug)]
enum PomodoroState {
    Work,
    Break,
}

#[derive(Debug)]
struct Pomodoro {
    current: Timer,
    work_dur: Duration,
    short_break_dur: Duration,
    long_break_dur: Duration,
    state: PomodoroState,
    pomos_completed: u16,
}

impl Pomodoro {
    fn new(work_dur: Duration, short_break_dur: Duration, long_break_dur: Duration) -> Self {
        Self {
            current: Timer::new(work_dur),
            state: PomodoroState::Work,
            pomos_completed: 0,
            work_dur,
            short_break_dur,
            long_break_dur,
        }
    }

    fn update(&mut self) {
        self.current.update();
        if self.current.is_finished() {
            (self.state, self.current) = match self.state {
                PomodoroState::Work => {
                    self.pomos_completed += 1;
                    if self.pomos_completed % 4 == 0 {
                        (PomodoroState::Break, Timer::new(self.long_break_dur))
                    } else {
                        (PomodoroState::Break, Timer::new(self.short_break_dur))
                    }
                }
                PomodoroState::Break => (PomodoroState::Work, Timer::new(self.work_dur)),
            }
        }
    }
}
