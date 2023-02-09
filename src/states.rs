use crate::{
    args::{Add, Command, Start, WorkOn},
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
    pub async fn parse_args(args: Option<Command>) -> Option<Self> {
        let state: Self = if let Some(command) = args {
            match command {
                Command::Start(Start {
                    work_mins,
                    short_break_mins,
                    long_break_mins,
                }) => Self::Working(Pomodoro::default().assign(Task {
                    work_secs: work_mins * 60,
                    short_break_secs: short_break_mins * 60,
                    long_break_secs: long_break_mins * 60,
                    ..Task::default()
                })),
                Command::List => {
                    db::print_tasks().await;
                    None?
                }
                Command::Add(Add {
                    desc,
                    work_mins,
                    short_break_mins,
                    long_break_mins,
                }) => {
                    db::write_task(
                        desc,
                        work_mins as i64,
                        short_break_mins as i64,
                        long_break_mins as i64,
                    )
                    .await
                    .unwrap();
                    None?
                }
                Command::WorkOn(WorkOn { id }) => {
                    Self::Working(Pomodoro::default().assign(db::read_task(id).await.unwrap()))
                }
            }
        } else {
            Self::Tasks(TasksState::new().await.unwrap())
        };
        Some(state)
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
