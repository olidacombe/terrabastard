use clap::Args;

use crate::{cli::Run, policy::json_iam_policy_to_data_resource};

#[derive(Clone, Debug, clap::Subcommand)]
pub enum Subcommand {
    ConvertJsonPolicy,
}

#[derive(Clone, Debug, Args)]
pub struct Command {
    #[command(subcommand)]
    command: Subcommand,
}

impl Run for Command {
    fn run(&self) {
        match self.command {
            Subcommand::ConvertJsonPolicy => {
                let policy_resource =
                    json_iam_policy_to_data_resource(std::io::stdin(), "CHANGEME")
                        // TODO
                        .unwrap();
                let policy_resource = hcl::to_string(&policy_resource)
                    // TODO
                    .unwrap();
                println!("{policy_resource}");
            }
        }
    }
}
