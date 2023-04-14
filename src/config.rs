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
    #[arg(
        short,
        long = "env",
        help = "Append an environment variable to all commands. Can be called multiple times",
        value_name = "KEY=VALUE"
    )]
    #[serde(rename = "env")]
    #[merge(strategy = merge::vec::prepend)] // highest priority is at the end
    pub envs: Vec<String>,

    #[arg(
        short,
        long,
        value_enum,
        env = "RUN_CLI_MODE",
        help = "Change the mode used to run commands"
    )]
    #[serde(rename = "mode")]
    pub mode: Option<Mode>,

    #[command(flatten)]
    #[serde(rename = "openai")]
    pub openai: Openai,

    #[command(flatten)]
    #[serde(rename = "prefix")]
    pub prefix: Prefix,

    #[arg(
        short,
        long,
        env = "RUN_CLI_RAW",
        help = "Output only stdout and stderr. Disabling all processors (prefix, openai, etc)",
        // boolean options
        value_parser = clap::builder::BoolishValueParser::new(),
        hide_possible_values = true,
        value_name = "true|false"
    )]
    #[serde(rename = "raw")]
    pub raw: Option<Option<bool>>,

    #[arg(skip)]
    #[serde(rename = "run")]
    #[merge(strategy = merge::vec::append)]
    pub runs: Vec<Command>,

    #[arg(
        short,
        long,
        env = "RUN_CLI_TAGS",
        help = "Filter to only run the commands matching at least one of the given tags. Can be comma-separated or passed multiple times",
        value_name = "TAG[,TAG]...",
        use_value_delimiter = true
    )]
    #[merge(strategy = merge::vec::append)]
    pub tags: Vec<String>,

    #[command(flatten)]
    #[serde(rename = "tmux")]
    pub tmux: Tmux,

    #[arg(
        long,
        env = "RUN_CLI_WORKDIR",
        help = "Change the base working directory of all commands"
    )]
    #[serde(rename = "workdir")]
    pub workdir: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Command {
    #[serde(rename = "cmd")]
    pub command_cmd: Vec<String>,

    #[serde(rename = "env")]
    pub command_envs: Vec<String>,

    #[serde(rename = "name")]
    pub command_name: Option<String>,

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
        // boolean options
        value_parser = clap::builder::BoolishValueParser::new(),
        hide_possible_values = true,
        value_name = "true|false"
    )]
    #[serde(rename = "enabled")]
    pub openai_enabled: Option<Option<bool>>,

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
        // boolean options
        value_parser = clap::builder::BoolishValueParser::new(),
        hide_possible_values = true,
        value_name = "true|false"
    )]
    #[serde(rename = "enabled")]
    pub prefix_enabled: Option<Option<bool>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Parser, Merge)]
#[serde(deny_unknown_fields, default)]
pub struct Tmux {
    #[arg(
        long = "tmux-kill-duplicate-session",
        env = "RUN_CLI_TMUX_KILL_DUPLICATE_SESSION",
        help = "Kill the existing tmux session if it already exists",
        // boolean options
        value_parser = clap::builder::BoolishValueParser::new(),
        hide_possible_values = true,
        value_name = "true|false"
    )]
    #[serde(rename = "kill_duplicate_session")]
    pub tmux_kill_duplicate_session: Option<Option<bool>>,

    #[arg(
        long = "tmux-program",
        env = "RUN_CLI_TMUX_PROGRAM",
        help = "Specify which tmux binary to use"
    )]
    #[serde(rename = "program")]
    pub tmux_program: Option<String>,

    #[arg(
        long = "tmux-session-prefix",
        env = "RUN_CLI_TMUX_SESSION_PREFIX",
        help = "Specify the tmux session prefix to use"
    )]
    #[serde(rename = "session_prefix")]
    pub tmux_session_prefix: Option<String>,

    #[arg(
        long = "tmux-socket-path",
        env = "RUN_CLI_TMUX_SOCKET_PATH",
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

    pub fn merge(&mut self, other: Self) {
        Merge::merge(self, other);
    }
}

impl TryFrom<Config> for RunnerOptions {
    type Error = anyhow::Error;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let raw = resolve_bool(config.raw, false);

        let workdir = config
            .workdir
            .unwrap_or(std::env::current_dir().expect("infaillible"));
        if !workdir.is_absolute() {
            anyhow::bail!("workdir must be an absolute path");
        }

        if config.runs.is_empty() {
            anyhow::bail!("no commands found in the config file or CLI arguments");
        }
        let commands = config
            .runs
            .into_iter()
            .map(|run| {
                let program = match run.command_cmd.get(0) {
                    Some(p) => p.to_string(),
                    _ => anyhow::bail!("no program found"),
                };

                let args = match run.command_cmd.get(1..) {
                    Some(a) => a.to_vec(),
                    _ => anyhow::bail!("no args found"),
                };

                let description = run.command_description;

                let envs: Vec<_> = config
                    .envs
                    .iter()
                    .chain(run.command_envs.iter())
                    .map(|kv| match kv.split_once('=') {
                        Some((k, v)) => Ok((k.to_string(), v.to_string())),
                        _ => anyhow::bail!("invalid environment variable: {}", kv),
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;

                let name = run.command_name.unwrap_or(run.command_cmd[0].clone());

                let tags = run.command_tags;

                let workdir = run
                    .command_workdir
                    .map(|w| {
                        let mut abs = workdir.to_path_buf();
                        abs.push(w);
                        abs.canonicalize().expect("infaillible")
                    })
                    .unwrap_or(workdir.clone());

                Ok(RunnerCommand {
                    program,
                    args,
                    description,
                    envs,
                    name,
                    tags,
                    workdir,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let mode = match config.mode.unwrap_or(Mode::Sequential) {
            Mode::Sequential => RunnerMode::Sequential,
            Mode::Parallel => RunnerMode::Parallel,
            Mode::Tmux => RunnerMode::Tmux,
        };

        let openai = match (
            raw,
            resolve_bool(config.openai.openai_enabled, false),
            config.openai.openai_api_key,
        ) {
            (false, true, Some(api_key)) => RunnerOpenai::Enabled {
                api_key,
                api_base_url: config
                    .openai
                    .openai_api_base_url
                    .unwrap_or("https://api.openai.com".into()),
            },
            _ => RunnerOpenai::Disabled,
        };

        let prefix = match (raw, resolve_bool(config.prefix.prefix_enabled, true)) {
            (false, true) => RunnerPrefix::Enabled,
            _ => RunnerPrefix::Disabled,
        };

        let tags = config.tags;

        let tmux = RunnerTmux {
            kill_duplicate_session: resolve_bool(config.tmux.tmux_kill_duplicate_session, true),
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
            tags,
            tmux,
        })
    }
}

fn resolve_bool(opts: Option<Option<bool>>, default_value: bool) -> bool {
    match opts {
        Some(Some(b)) => b,
        Some(None) => true,
        None => default_value,
    }
}
