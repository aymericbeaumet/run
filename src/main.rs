mod config;
mod pipeline;
mod runner;

use clap::Parser;
use config::Config;
use merge::Merge;
use runner::{Runner, RunnerOptions};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[arg(
        short,
        long = "file",
        help = "Specify the config file(s) to use (default: run.toml in the current directory)"
    )]
    pub files: Vec<PathBuf>,

    #[arg(
        help = "Only run the commands matching the given selectors",
        use_value_delimiter = true,
        value_delimiter = ','
    )] // positional arguments
    selectors: Vec<String>,

    #[command(flatten)]
    config: Config,

    #[arg(
        long = "check",
        help = "Start in check mode to validate the config and exit"
    )]
    pub command_check: bool,

    #[arg(
        long = "print-merged-config",
        help = "Print the merged config on stdout and exit"
    )]
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

    // Lowest priority is the default config
    let mut config = Config::default();

    // Then the config files
    for file in &cli.files {
        let (config_file, mut loaded_config) = Config::load(file).await?;

        // TODO: handle relative workdir
        loaded_config.workdir = Some(
            loaded_config
                .workdir
                .unwrap_or_else(|| config_file.parent().unwrap().to_path_buf()),
        );

        config.merge(loaded_config);
    }

    // And finally the cli/env
    config.merge(cli.config);

    if cli.command_check {
        return Ok(());
    }

    if cli.command_print_config {
        serde_json::to_writer_pretty(std::io::stdout(), &config)?;
        return Ok(());
    }

    let options = RunnerOptions::try_from(&config)?;

    if cli.command_print_options {
        serde_json::to_writer_pretty(std::io::stdout(), &options)?;
        return Ok(());
    }

    let runner = Runner::new(options);
    runner.run().await
}
