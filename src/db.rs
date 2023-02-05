use crate::states::App;
use crate::tasks::Task;
use sqlx::Connection;
use sqlx::{query, SqliteConnection};
use std::path::PathBuf;
use std::time::Duration;

const CFG_PATH_STR: &str = ".config/pogodoro/";
const DB_NAME: &str = "records.db";

pub async fn read_tasks() -> Result<Vec<Task>, sqlx::Error> {
    let mut conn = SqliteConnection::connect(crate::db::path().to_str().unwrap())
        .await
        .unwrap();
    let vec = query!("SELECT * FROM tasks WHERE completed = 0")
        .map(|task| Task {
            desc: task.desc,
            work_dur: Duration::from_secs(task.task_dur as u64),
            short_break_dur: Duration::from_secs(task.short_break_dur as u64),
            long_break_dur: Duration::from_secs(task.long_break_dur as u64),
            num_completed: task.num_completed as u32,
            completed: if task.completed == 1 { true } else { false },
        })
        .fetch_all(&mut conn)
        .await?;
    Ok(vec)
}

pub async fn set_done(desc: String) {
    let mut conn = SqliteConnection::connect(crate::db::path().to_str().unwrap())
        .await
        .unwrap();
    query!("UPDATE tasks SET completed = 1 WHERE desc = ?", desc)
        .execute(&mut conn)
        .await
        .unwrap();
}

pub fn cfg_path() -> PathBuf {
    let mut path = dir::home_dir().unwrap();
    path.push(CFG_PATH_STR);
    path
}

pub fn path() -> PathBuf {
    let mut path = cfg_path();
    path.push(DB_NAME);
    path
}

pub async fn setup() -> Result<(), sqlx::Error> {
    let path = path();
    let mut conn = SqliteConnection::connect(path.to_str().unwrap()).await?;
    sqlx::migrate!().run(&mut conn).await?;
    Ok(())
}
