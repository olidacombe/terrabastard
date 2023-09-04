use eyre::Result;
use std::{collections::HashSet, path::PathBuf};
use terrabastard::{
    cli::{self, Command, PlagueArgs},
    terraform::{self},
    walk::{self, string_repetitions},
};
use tracing::{debug, error};

fn init_tracing() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();
}

fn main() -> Result<()> {
    init_tracing();

    let args = cli::parse()?;

    match args.command {
        Command::Roots => {
            let roots: HashSet<PathBuf> = walk::find_roots(&args.path).collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&roots).unwrap_or("[]".to_string())
            );
        }
        Command::Parse => {
            let files: HashSet<PathBuf> = walk::find_files(&args.path).collect();
            for file in files {
                debug!("Parsing {:?}", &file);
                let body: Result<hcl::Body> = terraform::parse(&file);
                if let Err(e) = body {
                    error!("Bad terraform {:?}: {}", &file, e)
                }
            }
        }
        Command::Plague(PlagueArgs { min_repetitions }) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&string_repetitions(&args.path, min_repetitions))
                    .unwrap_or("{}".to_string())
            );
        }
    }

    Ok(())
}
