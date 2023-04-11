use clap::Parser;
use clap::ValueEnum;
use merge::Merge;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/*
 * Shared configuration for the command line interface and the TOML configuration file.
 *
 * Attributes will NOT BE deserialized from the CLI arguments by default. Unless
 * #[arg(...)] is passed.
 *
 * Attributes will BE deserialized from the TOML configuration by default. Unless
 * #[serde(skip_deserializing)] is passed.
 */

#[derive(Debug, Serialize, Deserialize, Default, Parser, Clone, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    #[arg(long = "check", help = "Start in check mode to validate the config")]
    #[serde(skip_deserializing, skip_serializing)]
    #[merge(skip)]
    pub command_check: bool,

    #[arg(long = "print-config", help = "Print the resolved config on stdout")]
    #[serde(skip_deserializing, skip_serializing)]
    #[merge(skip)]
    pub command_print_config: bool,

    #[arg(long = "print-optins", help = "Print the runners options on stdout")]
    #[serde(skip_deserializing, skip_serializing)]
    #[merge(skip)]
    pub command_print_options: bool,

    #[arg(
        short,
        long = "file",
        help = "Specify the config file(s) to use (default: run.toml in the current directory)"
    )]
    #[serde(skip_deserializing, skip_serializing)]
    #[merge(strategy = merge::vec::overwrite_empty)]
    pub files: Vec<PathBuf>,

    #[arg(use_value_delimiter = true, value_delimiter = ',')] // positional arguments
    #[serde(skip_deserializing)]
    #[merge(strategy = merge::vec::overwrite_empty)]
    selectors: Vec<String>,

    #[arg(
        short = 'A',
        long = "after",
        help = "Add a command to run after the selected commands"
    )]
    #[serde(skip_deserializing)]
    #[merge(strategy = merge::vec::append)]
    pub afters: Vec<String>,

    #[arg(
        short = 'B',
        long = "before",
        help = "Add a command to run before the selected commands"
    )]
    #[serde(skip_deserializing)]
    #[merge(strategy = merge::vec::append)]
    pub befores: Vec<String>,

    #[arg(skip)]
    #[serde(rename = "run")]
    #[merge(strategy = merge::vec::append)]
    pub commands: Vec<Command>,

    #[arg(
        short,
        long,
        value_enum,
        help = "Change the mode to use to run commands"
    )]
    pub mode: Option<Mode>,

    #[command(flatten)]
    pub openai: Openai,

    #[command(flatten)]
    pub tmux: Tmux,

    #[arg(long)]
    pub workdir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Default, Parser, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct Command {
    pub id: Option<String>,
    pub cmd: Vec<String>,
    pub description: Option<String>,
    pub workdir: Option<PathBuf>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Sequential,
    Parallel,
    Tmux,
}

#[derive(Debug, Serialize, Deserialize, Default, Parser, Clone, Merge)]
#[serde(deny_unknown_fields)]
pub struct Openai {
    #[arg(long, help = "Enable OpenAPI advices")]
    pub enabled: Option<bool>,

    #[arg(long = "openai-api-endpoint", help = "The OpenAI API endpoint to use")]
    pub api_endpoint: Option<String>,

    #[arg(long = "openai-api-key", help = "The OpenAI API key to use")]
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Parser, Clone, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Tmux {
    #[arg(long = "tmux-kill-duplicate-session")]
    pub kill_duplicate_session: Option<bool>,

    #[arg(long = "tmux-program")]
    pub program: Option<String>,

    #[arg(long = "tmux-session-prefix")]
    pub session_prefix: Option<String>,

    #[arg(long = "tmux-socket-path")]
    pub socket_path: Option<String>,
}

impl Config {
    pub async fn load<P: AsRef<Path>>(relpath: P) -> anyhow::Result<Config> {
        let mut config_path = std::env::current_dir()?;
        config_path.push(relpath);
        if std::fs::metadata(&config_path)?.is_dir() {
            config_path.push("run.toml");
        }
        let config_path = config_path.canonicalize()?;

        let config_str = tokio::fs::read_to_string(&config_path).await?;
        let config = toml::from_str(&config_str)?;

        Ok(config)
    }
}
