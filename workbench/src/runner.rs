use crate::config::Config;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

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
        thread::scope(|s| {
            for run in self.config.runs() {
                let (program, args) = run.cmd().unwrap();
                s.spawn(|| {
                    Command::new(program)
                        .args(args)
                        .current_dir(&self.cwd)
                        .spawn()
                        .unwrap()
                        .wait()
                        .unwrap();
                });
            }
        });

        Ok(())
    }
}
