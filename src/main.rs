mod config;
mod executor;
mod processors;
mod runner;

use clap::Parser;
use config::{Command, Config};
use merge::Merge;
use runner::{Runner, RunnerOptions};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[arg(
        short,
        long = "file",
        help = "Specify the config file to use (default is to load run.toml in the current directory, unless at least one COMMAND is passed)",
        value_name = "FILE"
    )]
    pub file: Option<PathBuf>,

    #[arg(
        short = 'c',
        long = "command",
        help = "Append a command to run. Can be called multiple times. Providing at least one command will prevent the default config file from being loaded",
        value_name = "COMMAND"
    )]
    pub commands: Vec<String>,

    #[command(flatten)]
    config: Config,

    #[arg(
        long = "check",
        help = "Start in check mode to validate the config and exit"
    )]
    pub command_check: bool,

    #[arg(long = "print-config", help = "Print the config on stdout and exit")]
    pub command_print_config: bool,

    #[arg(
        long = "print-runner-options",
        help = "Print the final runner options on stdout and exit"
    )]
    pub command_print_options: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(std::env::args_os())?;

    // The highest priority is the cli
    let mut config = cli.config;

    // Then comes the config file
    if let Some(file) = cli.file {
        config.merge(Config::load(file).await?);
    } else if cli.commands.is_empty() {
        config.merge(Config::load("run.toml").await?);
    }

    // And finally the defaults
    config.merge(Config::default());

    // Append all the cli commands to the config
    for command in cli.commands {
        config.runs.push(Command {
            command_cmd: shell_words::split(&command)?,
            ..Default::default()
        });
    }

    if cli.command_check {
        return Ok(());
    }

    if cli.command_print_config {
        serde_json::to_writer_pretty(std::io::stdout(), &config)?;
        return Ok(());
    }

    let options = RunnerOptions::try_from(config)?;

    if cli.command_print_options {
        serde_json::to_writer_pretty(std::io::stdout(), &options)?;
        return Ok(());
    }

    let runner = Runner::new(options);
    runner.run().await
}
