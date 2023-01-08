use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long)]
    pub name: Option<String>,
    #[arg(short, long)]
    pub nime: Option<String>,
}
