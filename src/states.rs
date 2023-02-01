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

pub enum AppState {
    Tasks(TasksState),
    Working(Pomodoro),
    Finished,
}

const CFG_PATH_STR: &'static str = ".config/pogodoro/";
const DB_NAME: &'static str = "records.db";

impl AppState {
    pub fn cfg_path() -> PathBuf {
        let mut path = dir::home_dir().unwrap();
        path.push(CFG_PATH_STR);
        path
    }

    pub async fn setup_db() -> Result<(), sqlx::Error> {
        let mut path = Self::cfg_path();
        std::fs::create_dir_all(&path).unwrap();
        // redirect to db path
        path.push(DB_NAME);

        let mut connection = SqliteConnection::connect(path.to_str().unwrap()).await?;

        sqlx::query!(
            "
CREATE TABLE IF NOT EXISTS task (
    desc TEXT,
    task_dur INTEGER NOT NULL,
    short_break_dur INTEGER NOT NULL,
    long_break_dur INTEGER NOT NULL,
    completed INTEGER DEFAULT 0
);  
"
        )
        .execute(&mut connection).await?;
        // connection.execute(query).unwrap();
        Ok(())
    }

    pub fn new(ops: Option<Commands>) -> Self {
        match ops {
            Some(Commands::Start(Start {
                work_dur,
                short_break_dur,
                long_break_dur,
            })) => Self::Working(Pomodoro::default().assign(Task {
                desc: None,
                work_dur: Duration::from_secs(work_dur * 60),
                short_break_dur: Duration::from_secs(short_break_dur * 60),
                long_break_dur: Duration::from_secs(long_break_dur * 60),
            })),

            None => Self::Tasks(TasksState::default()),
        }
    }

    pub fn tick(&mut self) {
        if let Self::Working(pomo) = self {
            pomo.update()
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
            *self = Self::Finished
        }
        match self {
            Self::Tasks(tasks) => {
                if tasks.should_finish(&key) {
                    *self = Self::Finished;
                    return;
                }
                // check if user has chosen some task, move on to pomo if so
                if let Some(task) = tasks.handle_key_event(key) {
                    *self = Self::Working(Pomodoro::default().assign(task))
                }
            }
            Self::Working(pomo) => {
                if pomo.should_finish(&key) {
                    *self = Self::Finished;
                    return;
                }
                pomo.handle_key_event(key)
            }
            Self::Finished => {}
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
