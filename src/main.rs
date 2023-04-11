mod config;
//mod pipeline;
mod runner;
//mod selector;

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

    #[arg(use_value_delimiter = true, value_delimiter = ',')] // positional arguments
    selectors: Vec<String>,

    #[command(flatten)]
    config: Config,

    #[arg(long = "check", help = "Start in check mode to validate the config")]
    pub command_check: bool,

    #[arg(long = "print-config", help = "Print the resolved config on stdout")]
    pub command_print_config: bool,

    #[arg(long = "print-optins", help = "Print the runners options on stdout")]
    pub command_print_options: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(std::env::args_os())?;

    // Lowest priority is the default
    let mut config = Config::default();

    // Then the config files
    for file in &cli.files {
        let config_file = Config::load(file).await?;
        config.merge(config_file);
    }

    // And finally the cli/env
    config.merge(cli.config);

    if cli.command_check {
        return Ok(());
    }

    if cli.command_print_config {
        println!("{}", toml::to_string(&config)?);
        return Ok(());
    }

    let options = RunnerOptions::try_from(&config)?;

    if cli.command_print_options {
        println!("{}", toml::to_string(&options)?);
        return Ok(());
    }

    let runner = Runner::new(options);
    //Ok(runner.run().await?)
    Ok(())
}
