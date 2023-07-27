use crate::tasks::Task;
use chrono::{Duration, Local, NaiveDateTime};
use sqlx::{query, query_as, Connection, Encode, FromRow, SqliteConnection};
use std::path::PathBuf;

const CFG_PATH_STR: &str = ".config/pogodoro/";
const DB_NAME: &str = "records.db";

#[derive(Debug, FromRow, Encode)]
pub struct Cycle {
    pub id: i64,
    pub task_id: i64,
    pub created_at: NaiveDateTime,
}

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

pub async fn write_from_add(task: crate::args::Add) -> sqlx::Result<()> {
    Ok(write_task(
        task.desc,
        task.work_mins as i64 * 60,
        task.short_break_mins as i64 * 60,
        task.long_break_mins as i64 * 60,
    )
    .await?)
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

pub async fn complete_cycle(task_id: Option<i64>) -> sqlx::Result<()> {
    let mut conn = get_conn().await?;
    query!("INSERT INTO cycles (task_id) VALUES (?)", task_id)
        .execute(&mut conn)
        .await?;
    Ok(())
}

async fn num_in_day(day: NaiveDateTime) -> sqlx::Result<usize> {
    let mut conn = get_conn().await?;
    let date_str = day_to_db_str(day);
    let result = query!(
        r#"SELECT COUNT(*) as count
           FROM cycles
           WHERE DATE(created_at) = ?"#,
        date_str
    )
    .fetch_one(&mut conn)
    .await?;

    Ok(result.count as usize)
}

fn day_to_db_str(day: NaiveDateTime) -> String {
    day.format("%Y-%m-%d").to_string()
}

async fn get_counts_for_dates(
    dates: Vec<NaiveDateTime>,
) -> sqlx::Result<Vec<(NaiveDateTime, usize)>> {
    // can't figure out how to do this with a map due to async weirdness with closures
    let mut counts = Vec::with_capacity(dates.len());

    for date in dates {
        counts.push((date, num_in_day(date).await?));
    }

    Ok(counts)
}

pub async fn last_7_day_cycles() -> sqlx::Result<Vec<(NaiveDateTime, usize)>> {
    let now = Local::now().naive_local();
    let mut dates = Vec::with_capacity(7);

    for days_back in (0..7).rev() {
        let date = now - Duration::days(days_back as i64);
        dates.push(date);
    }

    get_counts_for_dates(dates).await
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
