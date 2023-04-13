mod cli;
mod config;
mod executor;
mod processors;
mod runner;

use config::{Command, Config};
use runner::{Runner, RunnerOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    // The highest priority is the cli/env config
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

    let options = RunnerOptions::try_from(config)?;

    if cli.command_check {
        return Ok(());
    }

    if cli.command_print_options {
        serde_json::to_writer_pretty(std::io::stdout(), &options)?;
        return Ok(());
    }

    let runner = Runner::new(options);
    runner.run().await
}
