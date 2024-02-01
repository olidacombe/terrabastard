use clap::Args;

use super::Run;

pub mod iam;

#[derive(Clone, Debug, clap::Subcommand)]
pub enum Subcommand {
    Iam(iam::Command),
}

#[derive(Clone, Debug, Args)]
pub struct Command {
    #[command(subcommand)]
    command: Subcommand,
}

impl Run for Command {
    fn run(&self) {
        match &self.command {
            Subcommand::Iam(cmd) => cmd.run(),
        }
    }
}
