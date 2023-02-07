use crate::tasks::Task;
use sqlx::Connection;
use sqlx::{query, SqliteConnection};
use std::path::PathBuf;
use std::time::Duration;

const CFG_PATH_STR: &str = ".config/pogodoro/";
const DB_NAME: &str = "records.db";

pub async fn get_conn() -> SqliteConnection {
    SqliteConnection::connect(crate::db::path().to_str().unwrap())
        .await
        .expect("couldn't connect to database")
}

pub async fn read_tasks() -> Result<Vec<Task>, sqlx::Error> {
    let mut conn = get_conn().await;
    let vec = query!("SELECT rowid, * FROM tasks WHERE completed = 0")
        .map(|task| Task {
            desc: task.desc,
            id: Some(task.rowid as u32),
            work_dur: Duration::from_secs(task.task_dur as u64),
            short_break_dur: Duration::from_secs(task.short_break_dur as u64),
            long_break_dur: Duration::from_secs(task.long_break_dur as u64),
            pomos_finished: task.pomos_finished as u32,
            completed: if task.completed == 1 { true } else { false },
        })
        .fetch_all(&mut conn)
        .await?;
    Ok(vec)
}

pub async fn write_and_return_task(
    desc: String,
    work_dur: i64,
    short_break_dur: i64,
    long_break_dur: i64,
) -> Result<Task, sqlx::Error> {
    let mut conn = get_conn().await;
    // put task in DB
    query!(
        "INSERT INTO tasks VALUES (?, ?, ?, ?, 0, 0)",
        desc,
        work_dur,
        short_break_dur,
        long_break_dur,
    )
    .execute(&mut conn)
    .await?;
    // extract newly created task from db
    query!("SELECT rowid, * FROM tasks ORDER BY rowid DESC")
        .map(|task| Task {
            desc: task.desc,
            id: Some(task.rowid as u32),
            work_dur: Duration::from_secs(task.task_dur as u64),
            short_break_dur: Duration::from_secs(task.short_break_dur as u64),
            long_break_dur: Duration::from_secs(task.long_break_dur as u64),
            pomos_finished: task.pomos_finished as u32,
            completed: if task.completed == 1 { true } else { false },
        })
        .fetch_one(&mut conn)
        .await
}

pub async fn set_finished(id: i64, finished: i64) -> Result<(), sqlx::Error> {
    let mut conn = get_conn().await;
    query!(
        "UPDATE tasks SET pomos_finished = ? WHERE rowid = ?",
        finished,
        id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn set_done(id: i64) {
    let mut conn = get_conn().await;
    query!("UPDATE tasks SET completed = 1 WHERE rowid = ?", id)
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
