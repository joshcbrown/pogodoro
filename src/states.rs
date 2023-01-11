use crate::tasks::TasksState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::error;
use std::time::Duration;
use tui::backend::Backend;
use tui::Frame;

use crate::args::{Commands, Start};
use crate::pomodoro::Pomodoro;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum AppState {
    Tasks(TasksState),
    Working(Pomodoro),
    Finished,
}

impl AppState {
    pub fn new(ops: Option<Commands>) -> Self {
        match ops {
            Some(Commands::Start(Start {
                work_dur,
                short_break_dur,
                long_break_dur,
            })) => Self::Working(Pomodoro::new(
                Duration::from_secs(work_dur * 60),
                Duration::from_secs(short_break_dur * 60),
                Duration::from_secs(long_break_dur * 60),
            )),

            None => Self::Tasks(TasksState::new()),
        }
    }

    pub fn tick(&mut self) {
        match self {
            Self::Working(pomo) => pomo.update(),
            _ => {}
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
            *self = Self::Finished
        }
        if let Self::Tasks(tasks) = self {
            if tasks.should_finish(&key) {
                *self = Self::Finished
            }
        }
        match self {
            Self::Tasks(tasks) => tasks.handle_key_event(key),
            _ => {}
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        match self {
            Self::Working(pomo) => pomo.render(frame),
            Self::Tasks(tasks) => tasks.render(frame),
            AppState::Finished => {}
        }
    }
}
