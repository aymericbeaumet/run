mod config;
mod runner;

use clap::Parser;
use config::{Config, Mode, Run};
use runner::{Runner, RunnerOpts};
use std::{ffi::OsString, path::PathBuf};

#[derive(Parser)]
struct Args {
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
        help = "The config file to use (default: workbench.toml in the working directory)"
    )]
    file: Option<PathBuf>,

    #[arg(short, long, value_enum)]
    mode: Option<Mode>,

    #[arg(
        long = "workdir",
        help = "Change the working directory (default: the parent directory of FILE)"
    )]
    workdir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    workbench(std::env::args_os()).await
}

pub async fn workbench<I, T>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    // Parse arguments
    let args = Args::try_parse_from(args)?;

    // Find and parse config
    let mut config = Config::load(args.file).await?;

    // Override config with CLI flags
    if let Some(mode) = args.mode {
        config.mode = mode;
    }
    if let Some(workdir) = args.workdir {
        config.workdir.set(workdir);
    }

    // Override config with additional commands
    for (i, cmd) in args.commands.into_iter().enumerate() {
        config.runs.insert(
            format!("cli-{}", i),
            Run {
                cmd: vec![cmd], // TODO: not correct, properly parse cmd
                ..Default::default()
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
