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

    pub fn run_sequential(&self) -> anyhow::Result<()> {
        for run in self.config.runs() {
            let (program, args) = run.cmd()?;
            let mut child = Command::new(program)
                .args(args)
                .current_dir(&self.cwd)
                .spawn()?;
            child.wait()?;
        }

        Ok(())
    }

    pub fn run_parallel(&self) -> anyhow::Result<()> {
        let mut children = vec![];

        for run in self.config.runs() {
            let (program, args) = run.cmd()?;
            let child = Command::new(program)
                .args(args)
                .current_dir(&self.cwd)
                .spawn()?;
            children.push(child);
        }

        for mut child in children {
            child.wait()?;
        }

        Ok(())
    }
}
