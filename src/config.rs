use crate::runner::{RunnerMode, RunnerOpenai, RunnerOptions, RunnerTmux};
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

#[derive(Debug, Serialize, Deserialize, Parser, Clone, Merge, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    #[arg(
        short = 'c',
        long = "command",
        help = "Register a command to run. Can be called multiple times",
        value_name = "COMMAND"
    )]
    #[serde(skip_deserializing)]
    #[merge(strategy = merge::vec::append)]
    pub commands: Vec<String>,

    #[arg(skip)]
    #[serde(rename = "run")]
    #[merge(strategy = merge::vec::append)]
    pub runs: Vec<CommandOrString>,

    #[arg(short, long, value_enum, help = "Change the mode used to run commands")]
    pub mode: Option<Mode>,

    #[command(flatten)]
    pub openai: Openai,

    #[command(flatten)]
    pub tmux: Tmux,

    #[arg(long, help = "Change the working directory")]
    pub workdir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CommandOrString {
    String(String),
    Command(Command),
}

#[derive(Debug, Serialize, Deserialize, Parser, Clone, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Command {
    pub id: Option<String>,
    pub cmd: Vec<String>,
    pub description: Option<String>,
    pub workdir: Option<PathBuf>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Parser, Clone, Merge)]
#[serde(deny_unknown_fields)]
pub struct Openai {
    #[arg(
        long = "openai-enabled",
        env = "RUN_CLI_OPENAI_ENABLED",
        help = "Call the OpenAI API with the standard error output to try and give you advices"
    )]
    pub enabled: Option<bool>,

    #[arg(
        long = "openai-api-base-url",
        env = "RUN_CLI_OPENAI_API_BASE_URL",
        help = "The OpenAI API base url to use"
    )]
    pub api_base_url: Option<String>,

    #[arg(
        long = "openai-api-key",
        env = "RUN_CLI_OPENAI_API_KEY",
        help = "The OpenAI API key to use"
    )]
    pub api_key: Option<String>,
}

impl Default for Openai {
    fn default() -> Self {
        Self {
            enabled: Some(false),
            api_base_url: Some(String::from("https://api.openai.com")),
            api_key: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Sequential,
    Parallel,
    Tmux,
}

#[derive(Debug, Serialize, Deserialize, Parser, Clone, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Tmux {
    #[arg(
        long = "tmux-kill-duplicate-session",
        env = "RUN_CLI_TMUX_KILL_DUPLICATE_SESSION",
        help = "Kill the existing tmux session if it already exists"
    )]
    pub kill_duplicate_session: Option<bool>,

    #[arg(
        long = "tmux-program",
        env = "RUN_CLI_TMUX_PROGRAM",
        help = "Specify which tmux binary to use"
    )]
    pub program: Option<String>,

    #[arg(
        long = "tmux-session-prefix",
        env = "TMUX_SESSION_PREFIX",
        help = "Specify the tmux session prefix to use"
    )]
    pub session_prefix: Option<String>,

    #[arg(
        long = "tmux-socket-path",
        env = "TMUX_SOCKET_PATH",
        help = "Specify the tmux socket path to use"
    )]
    pub socket_path: Option<String>,
}

impl Default for Tmux {
    fn default() -> Self {
        Self {
            kill_duplicate_session: Some(true),
            program: Some("tmux".to_string()),
            session_prefix: Some("run-".to_string()),
            socket_path: Some("/tmp/tmux.run.sock".to_string()),
        }
    }
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

impl TryFrom<&Config> for RunnerOptions {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let commands = vec![];

        let mode = match config.mode.as_ref().unwrap() {
            Mode::Sequential => RunnerMode::Sequential,
            Mode::Parallel => RunnerMode::Parallel,
            Mode::Tmux => RunnerMode::Tmux,
        };

        let openai = match (config.openai.enabled, config.openai.api_key.as_ref()) {
            (Some(enabled), Some(api_key)) if enabled => RunnerOpenai::Enabled {
                api_key: api_key.clone(),
                api_base_url: config.openai.api_base_url.clone().unwrap(),
            },
            _ => RunnerOpenai::Disabled,
        };

        let tmux = RunnerTmux {
            kill_duplicate_session: config.tmux.kill_duplicate_session.unwrap_or_default(),
            program: config.tmux.program.clone().unwrap_or_default(),
            session_prefix: config.tmux.session_prefix.clone().unwrap_or_default(),
            socket_path: config.tmux.socket_path.clone().unwrap_or_default(),
        };

        Ok(Self {
            commands,
            mode,
            openai,
            tmux,
        })
    }
}
