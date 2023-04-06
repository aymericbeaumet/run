mod config;
mod runner;

use clap::Parser;
use config::{Config, Mode, Run};
use runner::{Runner, RunnerOpts};
use std::path::PathBuf;

#[derive(Parser)]
struct CLI {
    // positional arguments
    #[arg(use_value_delimiter = true, value_delimiter = ',')]
    selectors: Vec<String>,

    #[arg(
        short,
        long = "command",
        value_name = "CMD",
        help = "Append an additional command to run"
    )]
    commands: Vec<String>,

    #[arg(
        short,
        long,
        help = "The config file to use (default: workbench.toml in the current directory)"
    )]
    file: Option<PathBuf>,

    #[arg(short, long, value_enum, help = "The mode to use to run commands")]
    mode: Option<Mode>,

    #[arg(
        long = "workdir",
        help = "The working directory to work in (default: FILE's parent directory)"
    )]
    workdir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse arguments
    let cli = CLI::try_parse_from(std::env::args_os())?;

    // Find and parse config
    let mut config = Config::load(cli.file).await?;

    // Override config with CLI flags
    if let Some(mode) = cli.mode {
        config.mode = mode;
    }
    if let Some(workdir) = cli.workdir {
        let mut abs = std::env::current_dir()?;
        abs.push(workdir);
        config.workdir.set(abs);
    }

    // Override config with additional commands
    for (i, cmd) in cli.commands.into_iter().enumerate() {
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
            required_tags: cli.selectors, // TODO: not correct, properly parse selectors
        },
    );
    runner.run().await
}
