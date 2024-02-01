use clap::{Args, Parser, Subcommand};
use eyre::Result;
use std::{env, path::PathBuf};

pub mod aws;

fn get_default_path() -> PathBuf {
    env::current_dir().unwrap()
}

pub fn parse() -> Result<Cli> {
    Ok(Cli::parse())
}

pub trait Run {
    fn run(&self);
}

#[derive(Args, Clone, Debug)]
pub struct PathArg {
    #[arg(default_value=get_default_path().into_os_string())]
    pub path: PathBuf,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Aws(aws::Command),
    Parse(PathArg),
    Plague(PathArg),
    Roots(PathArg),
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
