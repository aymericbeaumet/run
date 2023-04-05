mod config;
mod runner;

use clap::Parser;
use config::{Cmd, Config, Mode, Run};
use runner::Runner;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
struct Args {
    config_file: Option<PathBuf>,

    #[arg(short, long = "command", value_name = "CMD")]
    commands: Vec<String>,

    #[arg(short, long = "mode", value_enum)]
    mode: Option<Mode>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse arguments
    let args = Args::try_parse()?;

    // Find and parse config
    let (mut config, config_file) = Config::load(args.config_file).await?;

    // Apply CLI flags
    config.runs.extend(args.commands.into_iter().map(|cmd| Run {
        cmd: Cmd::CmdString(cmd),
    }));
    config.mode = args.mode.unwrap_or(config.mode);

    // Infer CWD
    let cwd = Path::parent(&config_file).expect("the config file must be in a directory");

    // Run commands
    let mut runner = Runner::new(config, cwd);
    runner.run().await
}
