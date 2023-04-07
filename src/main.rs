mod config;
mod pipeline;
mod runner;

use anyhow::bail;
use clap::Parser;
use config::{Config, Mode, Run};
use runner::{Runner, RunnerOpts};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    // positional arguments
    #[arg(use_value_delimiter = true, value_delimiter = ',')]
    selectors: Vec<String>,

    // flag
    #[arg(
        short,
        long = "command",
        value_name = "CMD",
        help = "Append an additional command to run"
    )]
    commands: Vec<String>,

    // flag
    #[arg(
        short,
        long,
        help = "Specify the config file to use (default: run.toml in the current directory)",
        env = "RUN_CLI_FILE"
    )]
    file: Option<PathBuf>,

    // flag
    #[arg(
        short,
        long,
        value_enum,
        help = "Change the mode to use to run commands",
        env = "RUN_CLI_MODE"
    )]
    mode: Option<Mode>,

    // flag
    #[arg(
        long,
        help = "Ask OpenAPI for advices when your runs return an error",
        env = "RUN_CLI_OPENAI"
    )]
    openai: bool,

    // flag
    #[arg(
        short,
        long,
        value_enum,
        help = "The OpenAI API key to use",
        env = "RUN_CLI_OPENAI_API_KEY"
    )]
    openai_api_key: Option<String>,

    // --check command
    #[arg(long, help = "Start run in check mode")]
    check: bool,

    // --print-config command
    #[arg(long, help = "Print the resolved run config on stdout")]
    print_config: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse arguments
    let args = Args::try_parse_from(std::env::args_os())?;

    // Find and parse config. If no config file is specified, we try to find one in the current
    // directory, but it is allowed not to be here.
    let mut config = if let Some(file) = args.file {
        Config::load(file).await?
    } else if !args.commands.is_empty() {
        Config::load_allow_missing(std::env::current_dir().expect("infaillible")).await?
    } else {
        bail!("No config file specified and no additional commands to run");
    };

    // Override config with CLI flags
    if let Some(mode) = args.mode {
        config.mode = mode;
    }

    // Override config with additional runs
    for (i, cmd) in args.commands.into_iter().enumerate() {
        config.runs.insert(
            format!("cli-{}", i),
            Run {
                cmd: vec![cmd],       // TODO: not correct, properly parse cmd
                ..Default::default()  // no other fields can be set from CLI
            },
        );
    }

    if args.check {
        // noop
        Ok(())
    } else if args.print_config {
        println!("{}", toml::to_string_pretty(&config)?);
        Ok(())
    } else {
        let runner = Runner::new(
            config,
            RunnerOpts {
                openai: args.openai,
                openai_api_key: args.openai_api_key,
                required_tags: args.selectors, // TODO: not correct, properly parse selectors
            },
        );
        runner.run().await
    }
}
