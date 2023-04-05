mod config;
mod runner;

use clap::Parser;
use config::{Cmd, Config, Mode, Run};
use runner::Runner;
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

    #[arg(short, long = "mode", value_enum)]
    mode: Option<Mode>,

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

    // Extend config with CLI args
    config.runs.extend(args.commands.into_iter().map(|cmd| Run {
        cmd: Cmd::CmdString(cmd),
    }));
    config.mode = args.mode.unwrap_or(config.mode);
    config.workdir = args.workdir.unwrap_or(config.workdir);

    // Run commands
    let mut runner = Runner::new(config);
    runner.run().await
}
