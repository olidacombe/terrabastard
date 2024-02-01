use eyre::Result;
use std::{collections::HashSet, path::PathBuf};
use terrabastard::{
    cli::{self, Command, PathArg, Run},
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
        Command::Aws(cmd) => cmd.run(),
        Command::Roots(PathArg { path }) => {
            let roots: HashSet<PathBuf> = walk::find_roots(&path).collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&roots).unwrap_or("[]".to_string())
            );
        }
        Command::Parse(PathArg { path }) => {
            let files: HashSet<PathBuf> = walk::find_files(&path).collect();
            for file in files {
                debug!("Parsing {:?}", &file);
                let body: Result<hcl::Body> = terraform::parse(&file);
                if let Err(e) = body {
                    error!("Bad terraform {:?}: {}", &file, e)
                }
            }
        }
        Command::Plague(PathArg { path }) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&string_repetitions(path, 2))
                    .unwrap_or("{}".to_string())
            );
        }
    }

    Ok(())
}
