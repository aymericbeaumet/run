use clap::ValueEnum;
use serde::Deserialize;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

pub type Runs = indexmap::map::IndexMap<String, Run>;
pub type Tags = indexmap::set::IndexSet<String>;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub mode: Mode,
    #[serde(rename = "run")]
    pub runs: Runs,
    #[serde(default)]
    pub tmux: Tmux,
    #[serde(default)]
    pub workdir: Workdir,
}

#[derive(Deserialize, Clone, ValueEnum, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Sequential,
    Parallel,
    Tmux,
}

impl Config {
    pub async fn load<P: AsRef<Path>>(config_cli_path: Option<P>) -> anyhow::Result<Config> {
        let mut config_path = std::env::current_dir()?;
        if let Some(config_cli_path) = config_cli_path {
            config_path.push(config_cli_path);
        }
        if std::fs::metadata(&config_path)?.is_dir() {
            config_path.push("workbench.toml");
        }
        let config_path = config_path.canonicalize()?;

        let config_str = tokio::fs::read_to_string(&config_path).await?;
        let mut config: Config = toml::from_str(&config_str)?;

        // if workdir is not set, we set it to the directory of the config file
        if config.workdir.is_none() {
            config.workdir.set(
                config_path
                    .parent()
                    .expect("infaillible with an existing file")
                    .to_path_buf(),
            );
        }

        Ok(config)
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct Run {
    pub cmd: Vec<String>,
    #[serde(default)]
    pub tags: Tags,
}

#[derive(Deserialize)]
pub struct Tmux {
    pub kill_duplicate_session: bool,
    pub program: String,
    pub session_prefix: String,
    pub socket_path: String,
}

impl Default for Tmux {
    fn default() -> Self {
        Tmux {
            kill_duplicate_session: true,
            program: "tmux".to_string(),
            session_prefix: "workbench-".to_string(),
            socket_path: "/tmp/tmux.workbench.sock".to_string(),
        }
    }
}

#[derive(Deserialize, Default)]
pub struct Workdir(Option<PathBuf>);

impl Workdir {
    pub fn set(&mut self, workdir: PathBuf) {
        self.0.replace(workdir);
    }

    fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl AsRef<Path> for Workdir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref().expect("implementation error: always set")
    }
}

impl Deref for Workdir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("implementation error: always set")
    }
}
