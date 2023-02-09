use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A poggers-as-hell terminal UI pomodoro timer"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Adds task to DB
    Add(Add),
    /// Lists incomplete tasks
    List,
    /// Start a pomodoro session working on task with given ID
    WorkOn(WorkOn),
    /// Starts a (non-default) pomo session
    Start(Start),
}

#[derive(Args)]
pub struct Start {
    /// Duration of each working session in minutes
    pub work_mins: u64,
    /// Duration of each short break in minutes
    pub short_break_mins: u64,
    /// Duration of each long break in minutes
    pub long_break_mins: u64,
}

#[derive(Args)]
pub struct WorkOn {
    /// IDs can be listed with `pogodoro list`
    pub id: i64,
}

#[derive(Args)]
pub struct Task {
    /// id of task to begin (list IDs with pogodoro list)
    id: i64,
}

#[derive(Args)]
pub struct Add {
    pub desc: String,
    /// Duration of each working session in minutes
    pub work_mins: u64,
    /// Duration of each short break in minutes
    pub short_break_mins: u64,
    /// Duration of each long break in minutes
    pub long_break_mins: u64,
}
