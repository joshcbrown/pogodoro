use crate::{
    db,
    states::{AppResult, State},
    tasks::{Task, TasksState},
};
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use notify_rust::Notification;
use std::{
    cmp::max,
    fmt, io,
    time::{Duration, Instant},
};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
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
    fn notify(&self) -> AppResult<()> {
        let message = match self {
            Self::Work => "Time to work!",
            Self::ShortBreak => "Short break time!",
            Self::LongBreak => "Long break time!",
        };
        Notification::new()
            .summary("pogodoro")
            .body(message)
            .show()?;
        Ok(())
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
    pub should_finish: bool,
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
        let mut first_timer = Timer::new(Duration::from_secs(task.work_secs));
        first_timer.update();
        Self {
            id: None,
            current: first_timer,
            task: Task::default(),
            state: PomodoroState::Work,
            show_help: false,
            should_finish: false,
        }
    }
}

#[async_trait]
impl State for Pomodoro {
    async fn tick(&mut self) -> AppResult<()> {
        self.current.update();
        if self.current.is_finished() {
            self.change_timers().await?
        }
        Ok(())
    }

    fn should_finish(&self) -> bool {
        self.should_finish
    }

    fn render(&mut self, frame: &mut Frame<'_, CrosstermBackend<io::Stderr>>) {
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

    async fn handle_key_event(mut self: Box<Self>, event: KeyEvent) -> AppResult<Box<dyn State>> {
        match event.code {
            KeyCode::Char('p') => self.current.toggle_pause(),
            KeyCode::Char('n') => self.change_timers().await?,
            KeyCode::Char('q') => self.should_finish = true,
            KeyCode::Enter => {
                if let Some(id) = self.task.id {
                    db::complete(id as i64).await?;
                }
                return Ok(Box::new(TasksState::new().await?));
            }
            KeyCode::Esc => return Ok(Box::new(TasksState::new().await?)),
            KeyCode::Char('?') => {
                if self.show_help || !self.current.paused {
                    self.current.toggle_pause()
                }
                self.show_help = !self.show_help
            }
            _ => {}
        }
        Ok(self)
    }
}

impl Pomodoro {
    pub fn assign(self, task: Task) -> Self {
        let mut current = Timer::new(Duration::from_secs(task.work_secs));
        current.update();
        Self {
            task,
            current,
            ..self
        }
    }

    async fn change_timers(&mut self) -> AppResult<()> {
        (self.state, self.current) = match self.state {
            PomodoroState::Work => {
                self.task.pomos_finished += 1;
                db::complete_cycle(self.task.id.map(|i| i as i64)).await?;
                if let Some(id) = self.task.id {
                    db::set_finished(id as i64, self.task.pomos_finished as i64).await?;
                }
                if self.task.pomos_finished % 4 == 0 {
                    (
                        PomodoroState::LongBreak,
                        Timer::new(Duration::from_secs(self.task.long_break_secs)),
                    )
                } else {
                    (
                        PomodoroState::ShortBreak,
                        Timer::new(Duration::from_secs(self.task.short_break_secs)),
                    )
                }
            }
            PomodoroState::ShortBreak | PomodoroState::LongBreak => (
                PomodoroState::Work,
                Timer::new(Duration::from_secs(self.task.work_secs)),
            ),
        };
        self.state.notify()?;
        self.current.update();
        Ok(())
    }

    pub fn style(&self) -> Style {
        match self.state {
            PomodoroState::Work => Style::default().fg(Color::Red),
            PomodoroState::ShortBreak => Style::default().fg(Color::Green),
            PomodoroState::LongBreak => Style::default().fg(Color::Blue),
        }
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
