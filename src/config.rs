use clap::ValueEnum;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub type Tags = indexmap::set::IndexSet<String>;
pub type Runs = indexmap::map::IndexMap<String, Run>;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: Mode,
    pub tmux: Tmux,
    pub workdir: PathBuf,
    #[serde(rename = "run")]
    pub runs: Runs,
}

#[derive(Deserialize, Clone, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Sequential,
    Parallel,
    Tmux,
}

#[derive(Deserialize)]
pub struct Tmux {
    pub kill_duplicate_session: bool,
    pub program: String,
    pub session_prefix: String,
    pub socket_path: String,
}

#[derive(Deserialize, Clone, Default)]
pub struct Run {
    pub cmd: Vec<String>,
    #[serde(default)]
    pub tags: Tags,
}

impl Config {
    pub async fn load<P: AsRef<Path>>(rel_path: Option<P>) -> anyhow::Result<Config> {
        let mut abs_path = std::env::current_dir()?;
        if let Some(rel_path) = rel_path {
            abs_path.push(rel_path);
        }
        if std::fs::metadata(&abs_path)?.is_dir() {
            abs_path.push("workbench.toml");
        }
        let abs_path = abs_path.canonicalize()?;

        let content = tokio::fs::read_to_string(&abs_path).await?;
        let mut config: Config = toml::from_str(&content)?;

        config.workdir = abs_path
            .parent()
            .expect("infaillible with an existing file")
            .to_path_buf();

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mode: Mode::Sequential,
            tmux: Tmux {
                kill_duplicate_session: true,
                program: "tmux".to_string(),
                session_prefix: "workbench-".to_string(),
                socket_path: "/tmp/tmux.workbench.sock".to_string(),
            },
            workdir: std::env::current_dir().expect("cannot guess current dir"),
            runs: Runs::default(),
        }
    }
}
