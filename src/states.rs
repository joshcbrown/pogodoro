use crate::db;
use crate::tasks::Task;
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
    pub async fn new(ops: Option<Commands>) -> Self {
        match ops {
            Some(Commands::Start(Start {
                work_dur,
                short_break_dur,
                long_break_dur,
            })) => Self::Working(Pomodoro::default().assign(Task {
                work_dur: Duration::from_secs(work_dur * 60),
                short_break_dur: Duration::from_secs(short_break_dur * 60),
                long_break_dur: Duration::from_secs(long_break_dur * 60),
                ..Default::default()
            })),

            None => Self::Tasks(TasksState::new().await.unwrap()),
        }
    }

    pub async fn tick(&mut self) {
        if let Self::Working(pomo) = self {
            pomo.update().await
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        match self {
            Self::Working(pomo) => pomo.render(frame),
            Self::Tasks(tasks) => tasks.render(frame),
            AppState::Finished => {}
        }
    }

    pub async fn handle_key_event(&mut self, event: KeyEvent) {
        if event.code == KeyCode::Char('c') && event.modifiers == KeyModifiers::CONTROL {
            *self = AppState::Finished
        }
        match self {
            AppState::Tasks(tasks) => {
                if tasks.should_finish(&event) {
                    *self = AppState::Finished;
                    return;
                }
                // check if user has chosen some task, move on to pomo if so
                if let Some(task) = tasks.handle_key_event(event).await {
                    *self = AppState::Working(Pomodoro::default().assign(task))
                }
            }
            AppState::Working(pomo) => {
                if pomo.should_finish(&event) {
                    *self = AppState::Finished;
                    return;
                }
                // check if user has completed the pomo, return to tasks if so
                if let Some(id) = pomo.handle_key_event(event).await {
                    db::set_done(id as i64).await;
                    *self = AppState::Tasks(TasksState::new().await.unwrap())
                }
            }
            AppState::Finished => {}
        }
    }
}
