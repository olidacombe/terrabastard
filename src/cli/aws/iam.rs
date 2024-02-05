use clap::Args;

use crate::{cli::Run, policy::json_iam_policy_to_data_resource};

#[derive(Args, Clone, Debug)]
pub struct ConvertJsonPolicyArgs {
    #[arg(short, long, default_value = "\"this\"")]
    name: String,
}

#[derive(Clone, Debug, clap::Subcommand)]
pub enum Subcommand {
    ConvertJsonPolicy(ConvertJsonPolicyArgs),
}

#[derive(Clone, Debug, Args)]
pub struct Command {
    #[command(subcommand)]
    command: Subcommand,
}

impl Run for Command {
    fn run(&self) {
        match self.command {
            Subcommand::ConvertJsonPolicy(ref args) => {
                let policy_resource =
                    json_iam_policy_to_data_resource(std::io::stdin(), args.name.as_str())
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
