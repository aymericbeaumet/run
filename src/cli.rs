use crate::config::Config;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(bin_name = "run")]
#[command(display_name = "Run")]
#[command(version)]
#[command(about = "
Run is a task runner.

You can pass commands directly for simple tasks:
    $ run 'echo hello' 'ls /tmp'

Or you can use config files for more complex setups:
    $ run -f dev.toml

For more information: https://run-cli.org")]
pub struct Cli {
    #[arg(
        short,
        long = "file",
        help = "Specify the config file to load (default is to load run.toml in the current directory, unless at least one COMMAND is passed)",
        value_name = "FILE"
    )]
    pub file: Option<PathBuf>,

    #[arg(
        help = "Append a command to run. Can be called multiple times. Providing at least one command will prevent the default config file from being loaded",
        value_name = "COMMAND"
    )]
    pub commands: Vec<String>,

    #[command(flatten)]
    pub config: Config,

    #[arg(
        long = "check",
        help = "Start in check mode to validate the config and exit"
    )]
    pub command_check: bool,

    #[arg(
        long = "print-options",
        help = "Print the resolved options on stdout and exit"
    )]
    pub command_print_options: bool,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
