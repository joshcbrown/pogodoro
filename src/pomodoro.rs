use crate::tasks::Task;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use notify_rust::Notification;
use std::{
    cmp::max,
    fmt,
    time::{Duration, Instant},
};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct Timer {
    start_time: Instant,
    dur: Duration,
    elapsed: Duration,
    paused: bool,
}

impl Timer {
    fn new(dur: Duration) -> Self {
        Self {
            dur,
            start_time: Instant::now(),
            elapsed: Duration::from_secs(0),
            paused: false,
        }
    }

    fn update(&mut self) {
        // might check here later if timer is over
        if !self.paused {
            self.elapsed = self.start_time.elapsed();
        }
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.dur
    }

    pub fn toggle_pause(&mut self) {
        if !self.paused {
            self.paused = true;
            return;
        }
        self.paused = false;
        self.dur -= self.elapsed;
        self.start_time = Instant::now();
        self.update()
    }
}

impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_finished() {
            write!(f, "Finished!")
        } else {
            let to_go = (self.dur - self.elapsed + Duration::from_secs(1)).as_secs();
            let mins = to_go / 60;
            let hours = mins / 60;
            if hours > 0 {
                write!(f, "{}h{}m{}s", hours, mins - 60 * hours, to_go % 60)
            } else {
                write!(f, "{}m{}s", mins, to_go % 60)
            }
        }
    }
}

#[derive(Debug)]
pub enum PomodoroState {
    Work,
    ShortBreak,
    LongBreak,
}

impl PomodoroState {
    fn notify(&self) {
        let message = match self {
            Self::Work => "time to work!",
            Self::ShortBreak => "short break time! alright man",
            Self::LongBreak => "ALRIGHT! long break time man",
        };
        Notification::new()
            .summary("pogodoro")
            .body(message)
            .show()
            .unwrap();
    }
}

impl fmt::Display for PomodoroState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Work => "Work",
                Self::ShortBreak => "Short Break",
                Self::LongBreak => "Long Break",
            }
        )
    }
}

#[derive(Debug)]
pub struct Pomodoro {
    pub current: Timer,
    task: Task,
    state: PomodoroState,
    pomos_completed: u16,
}

const POMO_HEIGHT: u16 = 6;
const POMO_WIDTH: u16 = 25;

impl Default for Pomodoro {
    fn default() -> Self {
        let task = Task::default();
        let mut first_timer = Timer::new(task.work_dur);
        first_timer.update();
        Self {
            current: first_timer,
            task: Task::default(),
            state: PomodoroState::Work,
            pomos_completed: 0,
        }
    }
}

impl Pomodoro {
    pub fn assign(self, task: Task) -> Self {
        Self { task, ..self }
    }

    pub fn update(&mut self) {
        self.current.update();
        if self.current.is_finished() {
            (self.state, self.current) = match self.state {
                PomodoroState::Work => {
                    self.pomos_completed += 1;
                    if self.pomos_completed % 4 == 0 {
                        (
                            PomodoroState::LongBreak,
                            Timer::new(self.task.long_break_dur),
                        )
                    } else {
                        (
                            PomodoroState::ShortBreak,
                            Timer::new(self.task.short_break_dur),
                        )
                    }
                }
                PomodoroState::ShortBreak | PomodoroState::LongBreak => {
                    (PomodoroState::Work, Timer::new(self.task.work_dur))
                }
            };
            self.state.notify();
            self.current.update();
        }
    }

    pub fn style(&self) -> Style {
        match self.state {
            PomodoroState::Work => Style::default().fg(Color::Red),
            PomodoroState::ShortBreak => Style::default().fg(Color::Green),
            PomodoroState::LongBreak => Style::default().fg(Color::Blue),
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let frame_rect = frame.size();

        let (vert_height, hor_height) = if let Some(desc) = &self.task.desc {
            (
                POMO_HEIGHT + 1,
                max((desc.width() + "Remaining: ".width()) as u16, POMO_WIDTH),
            )
        } else {
            (POMO_HEIGHT, POMO_WIDTH)
        };

        let vert_buffer = frame_rect.height.saturating_sub(vert_height) / 2;
        let hor_buffer = frame_rect.width.saturating_sub(hor_height) / 2;

        let vert_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(vert_buffer),
                Constraint::Length(vert_height),
                Constraint::Min(vert_buffer),
            ])
            .split(frame.size());

        let hor_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(hor_buffer),
                Constraint::Length(hor_height),
                Constraint::Min(hor_buffer),
            ])
            .split(vert_chunks[1]);

        let pause_text = if self.current.paused {
            "un[p]ause"
        } else {
            "[p]ause"
        };

        let task_text = if let Some(desc) = &self.task.desc {
            format!("Working on: {}\n", desc)
        } else {
            "".into()
        };

        let pomo_text = format!(
            "{}Remaining: {}\nFinished: {}\n\n [q]uit {}",
            task_text, self.current, self.pomos_completed, pause_text
        );

        let pomo_widget = Paragraph::new(pomo_text)
            .block(
                Block::default()
                    .title(format!("Pogodoro — {}", self.state))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(self.style()),
            )
            .alignment(Alignment::Center);

        frame.render_widget(pomo_widget, hor_chunks[1]);
    }

    pub fn should_finish(&self, key: &KeyEvent) -> bool {
        key.code == KeyCode::Char('q') || key.code == KeyCode::Esc
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('p') => self.current.toggle_pause(),
            KeyCode::Enter => {
                // TODO: make this return to tasks page
            }
            _ => {}
        }
    }
}
