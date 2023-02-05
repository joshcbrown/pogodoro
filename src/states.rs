use crate::db;
use crate::tasks::Task;
use crate::tasks::TasksState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sqlx::{query, Connection, SqliteConnection};
use std::error;
use std::path::PathBuf;
use std::time::Duration;
use tui::backend::Backend;
use tui::Frame;

use crate::args::{Commands, Start};
use crate::pomodoro::Pomodoro;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App {
    new_tasks: Vec<Task>,
    pub state: AppState,
}

impl App {
    pub async fn new(ops: Option<Commands>) -> Self {
        Self {
            new_tasks: Vec::new(),
            state: AppState::new(ops).await,
        }
    }

    pub async fn handle_key_event(&mut self, event: KeyEvent) {
        if event.code == KeyCode::Char('c') && event.modifiers == KeyModifiers::CONTROL {
            self.state = AppState::Finished
        }
        match &mut self.state {
            AppState::Tasks(tasks) => {
                if tasks.should_finish(&event) {
                    self.new_tasks.append(&mut tasks.new_tasks());
                    self.state = AppState::Finished;
                    return;
                }
                // check if user has chosen some task, move on to pomo if so
                if let Some(task) = tasks.handle_key_event(event).await {
                    self.new_tasks.append(&mut tasks.new_tasks());
                    self.state = AppState::Working(Pomodoro::default().assign(task))
                }
            }
            AppState::Working(pomo) => {
                if pomo.should_finish(&event) {
                    self.state = AppState::Finished;
                    return;
                }
                if let Some(desc) = pomo.handle_key_event(event) {
                    self.write_db().await.unwrap();
                    // TODO:
                    db::set_done(desc).await;
                    self.state = AppState::Tasks(TasksState::new().await.unwrap())
                }
            }
            AppState::Finished => {}
        }
    }

    pub async fn write_db(&mut self) -> Result<(), sqlx::Error> {
        // TODO: move this to db.rs
        let mut conn = SqliteConnection::connect(db::path().to_str().unwrap())
            .await
            .unwrap();
        for task in self.new_tasks.drain(..) {
            let (work_dur, sb_dur, lb_dur) = (
                task.work_dur.as_secs() as u32,
                task.short_break_dur.as_secs() as u32,
                task.long_break_dur.as_secs() as u32,
            );
            query!(
                "INSERT INTO tasks VALUES (?, ?, ?, ?, ?, ?)",
                task.desc,
                work_dur,
                sb_dur,
                lb_dur,
                task.num_completed,
                task.completed
            )
            .execute(&mut conn)
            .await?;
        }
        Ok(())
    }
}

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

    pub fn tick(&mut self) {
        if let Self::Working(pomo) = self {
            pomo.update()
        }
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {}

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        match self {
            Self::Working(pomo) => pomo.render(frame),
            Self::Tasks(tasks) => tasks.render(frame),
            AppState::Finished => {}
        }
    }
}
