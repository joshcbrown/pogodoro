use crate::{
    args::{Commands, Start},
    db,
    pomodoro::Pomodoro,
    tasks::{Task, TasksState},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::error;
use tui::{backend::Backend, Frame};

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
                work_mins,
                short_break_mins,
                long_break_mins,
            })) => Self::Working(Pomodoro::default().assign(Task {
                work_secs: work_mins * 60,
                short_break_secs: short_break_mins * 60,
                long_break_secs: long_break_mins * 60,
                ..Task::default()
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
