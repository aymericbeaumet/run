mod config;
mod runner;

use clap::Parser;
use config::{Config, Mode, Run};
use runner::{Runner, RunnerOpts};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    // positional arguments
    #[arg(use_value_delimiter = true, value_delimiter = ',')]
    selectors: Vec<String>,

    // flag
    #[arg(
        short,
        long = "command",
        value_name = "CMD",
        help = "Append an additional command to run"
    )]
    commands: Vec<String>,

    // flag
    #[arg(
        short,
        long,
        help = "The config file to use (default: workbench.toml in the current directory)"
    )]
    file: Option<PathBuf>,

    // flag
    #[arg(short, long, value_enum, help = "The mode to use to run commands")]
    mode: Option<Mode>,

    // flag
    #[arg(
        long = "workdir",
        help = "The working directory to work in (default: FILE's parent directory)"
    )]
    workdir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse arguments
    let args = Args::try_parse_from(std::env::args_os())?;

    // Find and parse config
    let mut config = Config::load(args.file).await?;

    // Override config with CLI flags
    if let Some(mode) = args.mode {
        config.mode = mode;
    }
    if let Some(workdir) = args.workdir {
        let mut abs = std::env::current_dir()?; // TODO: clean this up
        abs.push(workdir);
        config.workdir.set(abs);
    }

    // Override config with additional commands
    for (i, cmd) in args.commands.into_iter().enumerate() {
        config.runs.insert(
            format!("cli-{}", i),
            Run {
                cmd: vec![cmd],       // TODO: not correct, properly parse cmd
                ..Default::default()  // no other fields can be set from CLI
            },
        );
    }

    // Create and start runner
    let runner = Runner::new(
        config,
        RunnerOpts {
            required_tags: args.selectors, // TODO: not correct, properly parse selectors
        },
    );
    runner.run().await
}
