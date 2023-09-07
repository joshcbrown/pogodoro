use crate::{
    args::{Command, Complete, Start, WorkOn},
    db,
    pomodoro::Pomodoro,
    tasks::{Task, TasksState},
};
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use std::{error, io};
use tui::{prelude::CrosstermBackend, Frame};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub type TaskId = u32;

#[async_trait]
pub trait State {
    async fn tick(&mut self) -> AppResult<()>;
    fn should_finish(&self) -> bool;
    fn render(&mut self, frame: &mut Frame<'_, CrosstermBackend<io::Stderr>>);
    async fn handle_key_event(mut self: Box<Self>, event: KeyEvent) -> AppResult<Box<dyn State>>;
}

pub async fn parse_args(args: Option<Command>) -> AppResult<Option<Box<dyn State>>> {
    let state: Box<dyn State> = if let Some(command) = args {
        match command {
            Command::Start(Start {
                work_mins,
                short_break_mins,
                long_break_mins,
            }) => Box::new(Pomodoro::default().assign(Task {
                work_secs: work_mins * 60,
                short_break_secs: short_break_mins * 60,
                long_break_secs: long_break_mins * 60,
                ..Task::default()
            })),
            Command::List => {
                db::print_tasks().await?;
                return Ok(None);
            }
            Command::Add(task) => {
                db::write_from_add(task).await?;
                return Ok(None);
            }
            Command::WorkOn(WorkOn { id }) => {
                Box::new(Pomodoro::default().assign(db::read_task(id).await?))
            }
            Command::Complete(Complete { id }) => {
                db::complete(id).await?;
                return Ok(None);
            }
        }
    } else {
        Box::new(TasksState::new().await?)
    };
    Ok(Some(state))
}
