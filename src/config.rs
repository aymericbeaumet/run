use crate::runner::{RunnerCommand, RunnerMode, RunnerOpenai, RunnerOptions, RunnerTmux, RunnerPrefixer};
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
    pub commands: Option<Vec<String>>,

    #[arg(skip)]
    #[serde(rename = "run")]
    #[merge(strategy = merge::vec::append)]
    pub runs: Vec<Command>,

    #[arg(short, long, value_enum, help = "Change the mode used to run commands")]
    #[serde(rename = "mode")]
    pub mode: Option<Mode>,

    #[command(flatten)]
    #[serde(rename = "openai")]
    pub openai: Openai,

    #[command(flatten)]
    #[serde(rename = "prefixer")]
    pub prefixer: Prefixer,

    #[command(flatten)]
    #[serde(rename = "tmux")]
    pub tmux: Tmux,

    #[arg(long, help = "Change the base working directory")]
    #[serde(rename = "workdir")]
    pub workdir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Parser, Clone, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Command {
    #[serde(rename = "cmd")]
    pub cmd: Vec<String>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "workdir")]
    pub workdir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Parser, Clone, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Openai {
    #[arg(
        long = "openai-enabled",
        env = "RUN_CLI_OPENAI_ENABLED",
        help = "Call the OpenAI API with stderr to try and give you advices"
    )]
    #[serde(rename = "enabled")]
    pub openai_enabled: Option<bool>,

    #[arg(
        long = "openai-api-base-url",
        env = "RUN_CLI_OPENAI_API_BASE_URL",
        help = "The OpenAI API base url to use"
    )]
    #[serde(rename = "api_base_url")]
    pub openai_api_base_url: Option<String>,

    #[arg(
        long = "openai-api-key",
        env = "RUN_CLI_OPENAI_API_KEY",
        help = "The OpenAI API key to use"
    )]
    #[serde(rename = "api_key")]
    pub openai_api_key: Option<String>,
}

impl Default for Openai {
    fn default() -> Self {
        Self {
           openai_enabled: Some(false),
           openai_api_base_url: Some(String::from("https://api.openai.com")),
           openai_api_key: None,
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
#[serde(deny_unknown_fields)]
pub struct Prefixer {
    #[arg(
        long = "prefixer-enabled",
        env = "RUN_CLI_PREFIXER_ENABLED",
        help = "Add a prefix to the command output"
    )]
    #[serde(rename = "enabled")]
    pub prefixer_enabled: Option<bool>,
}

impl Default for Prefixer {
    fn default() -> Self {
        Self {
            prefixer_enabled: Some(true),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Parser, Clone, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Tmux {
    #[arg(
        long = "tmux-kill-duplicate-session",
        env = "RUN_CLI_TMUX_KILL_DUPLICATE_SESSION",
        help = "Kill the existing tmux session if it already exists"
    )]
    #[serde(rename = "kill_duplicate_session")]
    pub tmux_kill_duplicate_session: Option<bool>,

    #[arg(
        long = "tmux-program",
        env = "RUN_CLI_TMUX_PROGRAM",
        help = "Specify which tmux binary to use"
    )]
    #[serde(rename = "program")]
    pub tmux_program: Option<String>,

    #[arg(
        long = "tmux-session-prefix",
        env = "TMUX_SESSION_PREFIX",
        help = "Specify the tmux session prefix to use"
    )]
    #[serde(rename = "session_prefix")]
    pub tmux_session_prefix: Option<String>,

    #[arg(
        long = "tmux-socket-path",
        env = "TMUX_SOCKET_PATH",
        help = "Specify the tmux socket path to use"
    )]
    #[serde(rename = "socket_path")]
    pub tmux_socket_path: Option<String>,
}

impl Default for Tmux {
    fn default() -> Self {
        Self {
           tmux_kill_duplicate_session: Some(true),
           tmux_program: Some("tmux".to_string()),
           tmux_session_prefix: Some("run-".to_string()),
           tmux_socket_path: Some("/tmp/tmux.run.sock".to_string()),
        }
    }
}

impl Config {
    pub async fn load<P: AsRef<Path>>(relpath: P) -> anyhow::Result<(PathBuf, Config)> {
        let mut config_path = std::env::current_dir()?;
        config_path.push(relpath);
        if std::fs::metadata(&config_path)?.is_dir() {
            config_path.push("run.toml");
        }
        let config_path = config_path.canonicalize()?;

        let config_str = tokio::fs::read_to_string(&config_path).await?;
        let config = toml::from_str(&config_str)?;

        Ok((config_path, config))
    }
}

impl TryFrom<Config> for RunnerOptions {
    type Error = anyhow::Error;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let commands = config
            .runs
            .into_iter()
            .map(|run| RunnerCommand {
                cmd: run.cmd.clone(),
                description: run.description.clone(),
                workdir: run.workdir.unwrap_or(config.workdir.clone().unwrap()),
            })
            .collect();

        let mode = match config.mode.as_ref().unwrap() {
            Mode::Sequential => RunnerMode::Sequential,
            Mode::Parallel => RunnerMode::Parallel,
            Mode::Tmux => RunnerMode::Tmux,
        };

        let openai = match (
            config.openai.openai_enabled.unwrap(),
            config.openai.openai_api_key.as_ref(),
        ) {
            (true, Some(api_key)) => RunnerOpenai::Enabled {
                api_key: api_key.clone(),
                api_base_url: config.openai.openai_api_base_url.clone().unwrap(),
            },
            _ => RunnerOpenai::Disabled,
        };

        let prefixer = if config.prefixer.prefixer_enabled.unwrap() {
            RunnerPrefixer::Enabled
        } else {
            RunnerPrefixer::Disabled
        };

        let tmux = RunnerTmux {
            kill_duplicate_session: config.tmux.tmux_kill_duplicate_session.unwrap(),
            program: config.tmux.tmux_program.clone().unwrap(),
            session_prefix: config.tmux.tmux_session_prefix.clone().unwrap(),
            socket_path: config.tmux.tmux_socket_path.unwrap(),
        };

        Ok(Self {
            commands,
            mode,
            openai,
            prefixer,
            tmux,
        })
    }
}
