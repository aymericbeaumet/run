use crate::runner::{
    RunnerCommand, RunnerMode, RunnerOpenai, RunnerOptions, RunnerPrefix, RunnerTmux,
};
use clap::Parser;
use clap::ValueEnum;
use merge::Merge;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/*
 * Shared configuration for the command line interface and the TOML configuration file.
 */

#[derive(Debug, Clone, Default, Serialize, Deserialize, Parser, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
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
    #[serde(rename = "prefix")]
    pub prefix: Prefix,

    #[command(flatten)]
    #[serde(rename = "tmux")]
    pub tmux: Tmux,

    #[arg(long, help = "Change the base working directory")]
    #[serde(rename = "workdir")]
    pub workdir: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Command {
    #[serde(rename = "cmd")]
    pub command_cmd: Vec<String>,

    #[serde(rename = "description")]
    pub command_description: Option<String>,

    #[serde(rename = "tags")]
    pub command_tags: Vec<String>,

    #[serde(rename = "workdir")]
    pub command_workdir: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Parser, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Openai {
    #[arg(
        long = "openai-enabled",
        env = "RUN_CLI_OPENAI_ENABLED",
        help = "Call the OpenAI API with stderr to try and give you advices",
        value_parser = clap::builder::BoolishValueParser::new(),
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Sequential,
    Parallel,
    Tmux,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Parser, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Prefix {
    #[arg(
        long = "prefix-enabled",
        env = "RUN_CLI_PREFIX_ENABLED",
        help = "Prefix each line from stdout and stderr with the command id",
        value_parser = clap::builder::BoolishValueParser::new(),
    )]
    #[serde(rename = "enabled")]
    pub prefix_enabled: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Parser, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Tmux {
    #[arg(
        long = "tmux-kill-duplicate-session",
        env = "RUN_CLI_TMUX_KILL_DUPLICATE_SESSION",
        help = "Kill the existing tmux session if it already exists",
        value_parser = clap::builder::BoolishValueParser::new(),
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

impl Config {
    pub async fn load<P: AsRef<Path>>(relpath: P) -> anyhow::Result<Config> {
        // Find the absolute path to the config file
        let mut config_path = std::env::current_dir()?;
        config_path.push(relpath);
        if std::fs::metadata(&config_path)?.is_dir() {
            config_path.push("run.toml");
        }
        let config_path = config_path.canonicalize()?;

        // Load configuration
        let config_str = tokio::fs::read_to_string(&config_path).await?;
        let mut config: Config = toml::from_str(&config_str)?;

        // Make sure the workdir is resolved relatively to the config file
        let mut workdir = config_path.parent().expect("infaillible").to_path_buf();
        if let Some(w) = config.workdir.as_ref() {
            workdir.push(w); // use provided workdir if found
        }
        let workdir = workdir.canonicalize()?;
        config.workdir = Some(workdir);

        Ok(config)
    }
}

impl TryFrom<Config> for RunnerOptions {
    type Error = anyhow::Error;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let workdir = config
            .workdir
            .unwrap_or(std::env::current_dir().expect("infaillible"));

        let commands = config
            .runs
            .into_iter()
            .map(|run| RunnerCommand {
                cmd: run.command_cmd,
                description: run.command_description,
                workdir: run
                    .command_workdir
                    .map(|w| {
                        let mut abs = workdir.to_path_buf();
                        abs.push(w);
                        abs.canonicalize().expect("infaillible")
                    })
                    .unwrap_or(workdir.clone()),
            })
            .collect();

        let mode = match config.mode.unwrap_or(Mode::Sequential) {
            Mode::Sequential => RunnerMode::Sequential,
            Mode::Parallel => RunnerMode::Parallel,
            Mode::Tmux => RunnerMode::Tmux,
        };

        let openai = match (
            config.openai.openai_enabled.unwrap_or(false),
            config.openai.openai_api_key,
        ) {
            (true, Some(api_key)) => RunnerOpenai::Enabled {
                api_key,
                api_base_url: config
                    .openai
                    .openai_api_base_url
                    .unwrap_or("https://api.openai.com".into()),
            },
            _ => RunnerOpenai::Disabled,
        };

        let prefix = if config.prefix.prefix_enabled.unwrap_or(true) {
            RunnerPrefix::Enabled
        } else {
            RunnerPrefix::Disabled
        };

        let tmux = RunnerTmux {
            kill_duplicate_session: config.tmux.tmux_kill_duplicate_session.unwrap_or(true),
            program: config.tmux.tmux_program.unwrap_or("tmux".into()),
            session_prefix: config.tmux.tmux_session_prefix.unwrap_or("run-cli-".into()),
            socket_path: config
                .tmux
                .tmux_socket_path
                .unwrap_or("/tmp/tmux.run_cli.sock".into()),
        };

        Ok(Self {
            commands,
            mode,
            openai,
            prefix,
            tmux,
        })
    }
}
