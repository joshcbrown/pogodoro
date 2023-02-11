use crate::tasks::Task;
use sqlx::{query, query_as, Connection, SqliteConnection};
use std::path::PathBuf;

const CFG_PATH_STR: &str = ".config/pogodoro/";
const DB_NAME: &str = "records.db";

pub async fn get_conn() -> sqlx::Result<SqliteConnection> {
    SqliteConnection::connect(crate::db::path().to_str().unwrap()).await
}

pub async fn read_tasks() -> sqlx::Result<Vec<Task>> {
    let mut conn = get_conn().await?;
    let vec = query_as("SELECT * FROM tasks WHERE completed = 0")
        .fetch_all(&mut conn)
        .await?;
    Ok(vec)
}

pub async fn read_task(id: i64) -> sqlx::Result<Task> {
    let mut conn = get_conn().await?;
    let vec = query_as(&format!("SELECT * FROM tasks WHERE id = {}", id))
        .fetch_one(&mut conn)
        .await?;
    Ok(vec)
}

pub async fn print_tasks() -> sqlx::Result<()> {
    let vec = read_tasks().await?;
    vec.iter().for_each(|task| println!("{}", task.to_string()));
    Ok(())
}

pub async fn write_task(
    desc: String,
    work_secs: i64,
    short_break_secs: i64,
    long_break_secs: i64,
) -> sqlx::Result<()> {
    let mut conn = get_conn().await?;
    // put task in DB
    query!(
        "
INSERT INTO tasks 
    (desc, work_secs, short_break_secs, long_break_secs, pomos_finished, completed) 
VALUES (?, ?, ?, ?, 0, 0)
        ",
        desc,
        work_secs,
        short_break_secs,
        long_break_secs,
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn write_and_return_task(
    desc: String,
    work_secs: i64,
    short_break_secs: i64,
    long_break_secs: i64,
) -> Result<Task, sqlx::Error> {
    write_task(desc, work_secs, short_break_secs, long_break_secs).await?;
    let mut conn = get_conn().await?;
    // extract newly created task from db
    query_as("SELECT * FROM tasks ORDER BY rowid DESC")
        .fetch_one(&mut conn)
        .await
}

pub async fn set_finished(id: i64, finished: i64) -> Result<(), sqlx::Error> {
    let mut conn = get_conn().await?;
    query!(
        "UPDATE tasks SET pomos_finished = ? WHERE rowid = ?",
        finished,
        id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn complete(id: i64) -> sqlx::Result<()> {
    let mut conn = get_conn().await?;
    query!("UPDATE tasks SET completed = 1 WHERE rowid = ?", id)
        .execute(&mut conn)
        .await?;
    Ok(())
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
