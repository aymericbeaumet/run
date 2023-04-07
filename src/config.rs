use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

pub type Runs = indexmap::map::IndexMap<String, Run>;
pub type Tags = indexmap::set::IndexSet<String>;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // misc
    #[serde(default)]
    pub workdir: Workdir,

    // mode
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub sequential: Sequential,
    #[serde(default)]
    pub parallel: Parallel,
    #[serde(default)]
    pub tmux: Tmux,

    // pipeline
    #[serde(default)]
    pub prefixer: Prefixer, // should this be in the pipeline package?
    #[serde(default)]
    pub openai: Openai, // should this be in the pipeline package?

    // runs
    #[serde(rename = "run")]
    pub runs: Runs,
}

#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum, Default)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Sequential,
    Parallel,
    Tmux,
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

    pub async fn load_allow_missing<P: AsRef<Path>>(relpath: P) -> anyhow::Result<Config> {
        match Config::load(relpath).await {
            Ok(config) => Ok(config),
            Err(err) => {
                if err
                    .downcast_ref::<std::io::Error>()
                    .map_or(false, |e| e.kind() == std::io::ErrorKind::NotFound)
                {
                    Ok(Config::default())
                } else {
                    Err(err)
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub cmd: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tmux {
    #[serde(default)]
    pub kill_duplicate_session: bool,
    #[serde(default)]
    pub program: String,
    #[serde(default)]
    pub session_prefix: String,
    #[serde(default)]
    pub socket_path: String,
}

impl Default for Tmux {
    fn default() -> Self {
        Tmux {
            kill_duplicate_session: true,
            program: "tmux".to_string(),
            session_prefix: "run-".to_string(),
            socket_path: "/tmp/tmux.run.sock".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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
