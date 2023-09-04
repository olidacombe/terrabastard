use clap::{Args, Parser, Subcommand};
use eyre::Result;
use std::{env, path::PathBuf};

fn get_default_path() -> PathBuf {
    env::current_dir().unwrap()
}

pub fn parse() -> Result<Cli> {
    Ok(Cli::parse())
}

#[derive(Args)]
pub struct PlagueArgs {
    #[arg(short, long, default_value_t = 2)]
    /// Minimum repetitions for reporting
    pub min_repetitions: usize,
}

#[derive(Subcommand)]
pub enum Command {
    Parse,
    Plague(PlagueArgs),
    Roots,
}

#[derive(Parser)]
pub struct Cli {
    /// Top-level directory to process
    #[arg(short, long, default_value=get_default_path().into_os_string())]
    pub path: PathBuf,
    #[command(subcommand)]
    pub command: Command,
}
