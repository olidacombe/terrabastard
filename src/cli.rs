use clap::{Parser, ValueEnum};
use eyre::Result;
use std::{env, path::PathBuf};

fn get_default_path() -> PathBuf {
    env::current_dir().unwrap()
}

pub fn parse() -> Result<Cli> {
    Ok(Cli::parse())
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Command {
    Parse,
    Plague,
    Roots,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(value_enum)]
    pub command: Command,
    #[arg(default_value=get_default_path().into_os_string())]
    pub path: PathBuf,
}
