mod config;
mod runner;

use clap::Parser;
use config::{Config, Mode, Run};
use runner::{Runner, RunnerOpts};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    config_file: Option<PathBuf>,

    #[arg(
        short,
        long = "command",
        value_name = "CMD",
        help = "Append an additional command to run"
    )]
    commands: Vec<String>,

    #[arg(short, long, value_enum)]
    mode: Option<Mode>,

    #[arg(
        short = 't',
        long = "tags",
        help = "Only run the tasks matching all the given tags"
    )]
    required_tags: Vec<String>,

    #[arg(
        long = "workdir",
        help = "Change the working directory (default to the workbench.toml directory)"
    )]
    workdir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse arguments
    let args = Args::try_parse()?;

    // Find and parse config
    let mut config = Config::load(args.config_file).await?;

    // Override config with CLI flags
    config.mode = args.mode.unwrap_or(config.mode);
    config.workdir = args.workdir.unwrap_or(config.workdir);

    // Override config with additional commands
    for (i, cmd) in args.commands.into_iter().enumerate() {
        config.runs.insert(
            format!("cli-{}", i),
            Run {
                cmd: vec![cmd], // TODO: parse cmd
                ..Default::default()
            },
        );
    }

    // Create and start runner
    let runner = Runner::new(
        config,
        RunnerOpts {
            required_tags: args.required_tags,
        },
    );
    runner.run().await
}
