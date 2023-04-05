mod config;
mod runner;

use clap::Parser;
use config::{Cmd, Config, Mode, Run};
use runner::{Runner, RunnerOpts};
use std::path::PathBuf;

#[derive(Parser, Debug)]
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

    #[arg(short, long, help = "Only run the tasks matching all the given tags")]
    tags: Vec<String>,

    #[arg(
        long = "workdir",
        help = "Change the working directory (default to the workbench file directory)"
    )]
    workdir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse arguments
    let args = Args::try_parse()?;

    // Find and parse config
    let mut config = Config::load(args.config_file).await?;

    // Override config with CLI args
    config.runs.extend(args.commands.into_iter().map(|cmd| Run {
        cmd: Cmd::CmdString(cmd),
        tags: vec![],
    }));
    config.mode = args.mode.unwrap_or(config.mode);
    config.workdir = args.workdir.unwrap_or(config.workdir);

    let opts = RunnerOpts { tags: args.tags };

    // Run commands
    let mut runner = Runner::new(config, opts);
    runner.run().await
}
