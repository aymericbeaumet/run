use crate::config::Config;
use std::path::PathBuf;
use std::process::Command;

pub struct Runner {
    config: Config,
    cwd: PathBuf,
}

impl Runner {
    pub fn new<P: Into<PathBuf>>(config: Config, cwd: P) -> Self {
        Runner {
            config,
            cwd: cwd.into(),
        }
    }

    pub fn execute(&self) -> anyhow::Result<()> {
        let children = self.config.runs().map(|run| {
            let (program, args) = run.cmd().unwrap();
            Command::new(program)
                .args(args)
                .current_dir(&self.cwd)
                .spawn()
                .unwrap()
        });

        for mut child in children {
            child.wait().unwrap();
        }

        Ok(())
    }
}
