use crate::db;
use crate::tasks::Task;
use crossterm::event::{KeyCode, KeyEvent};
use notify_rust::Notification;
use std::{
    cmp::max,
    fmt,
    time::{Duration, Instant},
};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
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
        self.start_time = Instant::now() - self.elapsed;
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
                Self::ShortBreak => "Short break",
                Self::LongBreak => "Long break",
            }
        )
    }
}

#[derive(Debug)]
pub struct Pomodoro {
    pub id: Option<u32>,
    pub current: Timer,
    pub task: Task,
    pub state: PomodoroState,
    pub show_help: bool,
}

const POMO_HEIGHT: u16 = 5;
const POMO_WIDTH: u16 = 25;
const HELP_TEXT: &str = "[p] - toggle pause on current pomo
[n] - skip to next cycle in pomo
[q] - quit session and return to command line
[enter] - complete task and return to tasks page
[?] - toggle this help page";

impl Default for Pomodoro {
    fn default() -> Self {
        let task = Task::default();
        let mut first_timer = Timer::new(task.work_dur);
        first_timer.update();
        Self {
            id: None,
            current: first_timer,
            task: Task::default(),
            state: PomodoroState::Work,
            show_help: false,
        }
    }
}

impl Pomodoro {
    pub fn assign(self, task: Task) -> Self {
        let mut current = Timer::new(task.work_dur);
        current.update();
        Self {
            task,
            current,
            ..self
        }
    }

    pub async fn update(&mut self) {
        self.current.update();
        if self.current.is_finished() {
            self.change_timers().await
        }
    }

    async fn change_timers(&mut self) {
        (self.state, self.current) = match self.state {
            PomodoroState::Work => {
                self.task.pomos_finished += 1;
                if let Some(id) = self.task.id {
                    db::set_finished(id as i64, self.task.pomos_finished as i64)
                        .await
                        .unwrap();
                }
                if self.task.pomos_finished % 4 == 0 {
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

    pub fn style(&self) -> Style {
        match self.state {
            PomodoroState::Work => Style::default().fg(Color::Red),
            PomodoroState::ShortBreak => Style::default().fg(Color::Green),
            PomodoroState::LongBreak => Style::default().fg(Color::Blue),
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        if self.show_help {
            let help_chunk = centered_rect(50, 7, frame.size());
            let help_text = Paragraph::new(HELP_TEXT)
                .block(
                    Block::default()
                        .title("Help")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(help_text, help_chunk);
            return;
        }
        let (height, width) = if let Some(desc) = &self.task.desc {
            (
                POMO_HEIGHT + 1,
                max(
                    (desc.width() + "Working on: ".width() + 2) as u16,
                    POMO_WIDTH,
                ),
            )
        } else {
            (POMO_HEIGHT, POMO_WIDTH)
        };

        let pomo_chunk = centered_rect(width, height, frame.size());

        let pause_text = if self.current.paused {
            " — paused"
        } else {
            ""
        };

        frame.render_widget(
            Block::default()
                .title(format!("{}{}", self.state, pause_text))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(self.style()),
            pomo_chunk,
        );

        // split into info and gauge
        let pomo_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if self.task.desc.is_some() { 2 } else { 1 }),
                Constraint::Length(2),
            ])
            .margin(1)
            .split(pomo_chunk);

        let task_text = if let Some(desc) = &self.task.desc {
            format!("Working on: {}\n", desc)
        } else {
            "".into()
        };

        let pomo_text = format!("{}Finished: {}", task_text, self.task.pomos_finished);

        let pomo_par = Paragraph::new(pomo_text).alignment(Alignment::Left);

        frame.render_widget(pomo_par, pomo_chunks[0]);

        let gauge = Gauge::default()
            .block(Block::default().title(format!("Remaining: {}", self.current)))
            .gauge_style(self.style())
            .ratio(self.current.elapsed.as_secs_f64() / self.current.dur.as_secs_f64())
            .use_unicode(true);

        frame.render_widget(gauge, pomo_chunks[1]);
    }

    pub fn should_finish(&self, key: &KeyEvent) -> bool {
        key.code == KeyCode::Char('q') || key.code == KeyCode::Esc
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Option<u32> {
        match key.code {
            KeyCode::Char('p') => self.current.toggle_pause(),
            KeyCode::Char('n') => self.change_timers().await,
            KeyCode::Enter => return self.task.id,
            KeyCode::Char('?') => {
                if self.show_help || !(self.show_help || self.current.paused) {
                    self.current.toggle_pause()
                }
                self.show_help = !self.show_help
            }
            _ => {}
        }
        None
    }
}

pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let vert_buffer = r.height.saturating_sub(height) / 2;
    let hor_buffer = r.width.saturating_sub(width) / 2;

    let hor_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(vert_buffer),
            Constraint::Length(height),
            Constraint::Min(vert_buffer),
        ])
        .split(r)[1];

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(hor_buffer),
            Constraint::Length(width),
            Constraint::Min(hor_buffer),
        ])
        .split(hor_chunk)[1]
}
