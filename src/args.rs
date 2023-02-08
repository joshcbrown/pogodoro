use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A poggers-as-hell terminal UI pomodoro timer"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
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
