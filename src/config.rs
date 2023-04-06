use clap::ValueEnum;
use serde::Deserialize;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

pub type Runs = indexmap::map::IndexMap<String, Run>;
pub type Tags = indexmap::set::IndexSet<String>;

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Deserialize, Clone, ValueEnum, Default)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Sequential,
    Parallel,
    Tmux,
}

impl Config {
    pub async fn load<P: AsRef<Path>>(relpath: Option<P>) -> anyhow::Result<Config> {
        let mut config_path = std::env::current_dir()?;
        if let Some(relpath) = relpath {
            config_path.push(relpath);
        }
        if std::fs::metadata(&config_path)?.is_dir() {
            config_path.push("workbench.toml");
        }
        let config_path = config_path.canonicalize()?;

        let config_str = tokio::fs::read_to_string(&config_path).await?;
        let mut config: Config = toml::from_str(&config_str)?;

        // if workdir is not set, we set it to the directory of the config file
        let config_dir = config_path
            .parent()
            .expect("infaillible with an existing file")
            .to_path_buf();
        if config.workdir.is_none() {
            config.workdir.set(config_dir);
        } else {
            let mut abs = config_dir;
            abs.push(&config.workdir);
            config.workdir.set(abs);
        }

        Ok(config)
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub cmd: Vec<String>,
    pub description: Option<String>,
    pub workdir: Option<PathBuf>,
    #[serde(default)]
    pub tags: Tags,
}

impl Run {
    pub fn title<S: AsRef<str>>(&self, id: S) -> String {
        self.description
            .as_ref()
            .map(|desc| format!("{}: {}", id.as_ref(), desc))
            .unwrap_or(id.as_ref().to_string())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Workdir(Option<PathBuf>);

impl Workdir {
    pub fn set(&mut self, workdir: PathBuf) {
        assert!(
            workdir.is_absolute(),
            "implementation error: must always be absolute"
        );
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
