use crate::states::App;
use crate::tasks::Task;
use sqlx::Connection;
use sqlx::{query, SqliteConnection};
use std::path::PathBuf;
use std::time::Duration;

const CFG_PATH_STR: &str = ".config/pogodoro/";
const DB_NAME: &str = "records.db";

pub async fn read_tasks() -> Result<Vec<Task>, sqlx::Error> {
    let mut conn = SqliteConnection::connect(App::db_path().to_str().unwrap())
        .await
        .unwrap();
    let vec = query!("SELECT * FROM tasks WHERE completed = 0")
        .map(|task| Task {
            desc: task.desc,
            work_dur: Duration::from_secs(task.task_dur as u64),
            short_break_dur: Duration::from_secs(task.short_break_dur as u64),
            long_break_dur: Duration::from_secs(task.long_break_dur as u64),
        })
        .fetch_all(&mut conn)
        .await?;
    Ok(vec)
}

pub fn cfg_path() -> PathBuf {
    let mut path = dir::home_dir().unwrap();
    path.push(CFG_PATH_STR);
    path
}

pub fn db_path() -> PathBuf {
    let mut path = cfg_path();
    path.push(DB_NAME);
    path
}

pub async fn setup_db() -> Result<(), sqlx::Error> {
    let path = db_path();
    let mut conn = SqliteConnection::connect(path.to_str().unwrap()).await?;
    sqlx::migrate!().run(&mut conn).await?;
    Ok(())
}
