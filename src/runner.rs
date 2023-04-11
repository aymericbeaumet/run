//use crate::pipeline::Pipeline;
use anyhow::Context;
use serde::Serialize;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};

pub struct Runner {
    options: RunnerOptions,
}

impl Runner {
    pub fn new(options: RunnerOptions) -> Self {
        Self { options }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        match self.options.mode {
            RunnerMode::Sequential => self.run_sequential().await,
            RunnerMode::Parallel => self.run_parallel().await,
            RunnerMode::Tmux => self.run_tmux().await,
        }
    }

    async fn run_sequential(&self) -> anyhow::Result<()> {
        for run in &self.options.commands {
            // TODO: clean + refactor
            let mut child = self.exec(&run.cmd, &run.workdir).await?;

            //Pipeline::new(
            //format!("[{}]", &run.cmd[0]),
            //self.options.openai.api_key.clone(),
            //)
            //.process(child.stdout.take().unwrap(), child.stderr.take().unwrap())
            //.await?;

            child.wait().await?;
        }

        Ok(())
    }

    async fn run_parallel(&self) -> anyhow::Result<()> {
        let mut children = vec![];

        for run in &self.options.commands {
            let child = self.exec(&run.cmd, &run.workdir).await?;

            //let _pipeline = Pipeline::new(format!("[{}]", id), self.openapi_api_key.clone());
            // TODO: process with pipeline in a non-blocking manner

            children.push(child);
        }

        for mut child in children {
            child.wait().await?;
        }

        Ok(())
    }

    async fn run_tmux(&self) -> anyhow::Result<()> {
        let session = format!(
            "{}{}",
            self.options.tmux.session_prefix, "01" /* uuid::Uuid::new_v4() */
        );

        if self.options.tmux.kill_duplicate_session {
            if let Err(err) = self.tmux(["kill-session", "-t", &session]).await {
                println!("[debug] failed to kill duplicate session: {}", err); // TODO: use log library
            }
        }

        for (i, run) in self.options.commands.iter().enumerate() {
            let program = &run.cmd[0];
            let args = &run.cmd[1..];

            let workdir = &run.workdir.to_string_lossy();
            let cmd_str = &format!("{} {}; read", program, args.join(" ")); // TODO: make it more robust

            // create the pane
            if i == 0 {
                self.tmux(["new-session", "-s", &session, "-d", "-c", workdir, cmd_str])
                    .await?;
            } else {
                self.tmux(["split-window", "-t", &session, "-v", "-c", workdir, cmd_str])
                    .await?;
            }

            // set pane title
            self.tmux(["select-pane", "-t", &session, "-T", &run.cmd[0]])
                .await?;

            // select layout after spawning each command to avoid: https://stackoverflow.com/a/68362774/1071486
            self.tmux(["select-layout", "-t", &session, "even-vertical"])
                .await?;
        }

        // TODO: unbind-key -a
        // TODO: bind Ctrl-C globally to kill session

        for options in [
            ["mouse", "on"],
            // status
            ["status", "on"],
            ["status-position", "top"],
            ["status-justify", "absolute-centre"],
            ["status-left", ""],
            ["status-left-length", "0"],
            ["status-right", ""],
            ["status-right-length", "0"],
            ["window-status-current-format", "~ RUN ~"],
            // pane
            ["pane-border-format", "╣ #{pane_title} ╠"],
            ["pane-border-indicators", "off"],
            ["pane-border-lines", "double"],
            ["pane-border-status", "top"],
            // theme
            ["status-style", "fg=white bg=orange"],
            ["pane-border-style", "fg=white bg=orange"],
            ["pane-active-border-style", "fg=white bg=orange"],
        ] {
            self.tmux([["set-option", "-t", &session, "-s"].as_ref(), &options].concat())
                .await?;
        }

        self.tmux(["attach-session", "-t", &session]).await
    }

    async fn tmux<I, S>(&self, args: I) -> anyhow::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new(&self.options.tmux.program);
        cmd.args(["-S", &self.options.tmux.socket_path]);
        cmd.args(args);

        let mut child = cmd
            .spawn()
            .with_context(|| format!("could not spawn {:?}", &self.options.tmux.program))?;

        let status = child.wait().await?;
        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "tmux command failed",
            ))?;
        }

        // TODO: report status code in the pane title
        // TODO: report time to finish in the pane title

        Ok(())
    }

    async fn exec<P: AsRef<Path> + std::fmt::Debug>(
        &self,
        cmd: &[String],
        workdir: P,
    ) -> anyhow::Result<Child> {
        let mut child = Command::new(&cmd[0]);

        let child = child
            .env_clear()
            .args(&cmd[1..])
            .current_dir(workdir.as_ref())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("could not spawn {:?} in {:?}", &cmd, &workdir))?;

        Ok(child)
    }
}

#[derive(Debug, Serialize)]
pub struct RunnerOptions {
    pub commands: Vec<RunnerCommand>,
    pub mode: RunnerMode,
    pub openai: RunnerOpenai,
    pub tmux: RunnerTmux,
}

#[derive(Debug, Serialize)]
pub struct RunnerCommand {
    pub cmd: Vec<String>,
    pub workdir: PathBuf,
}

#[derive(Debug, Serialize)]
pub enum RunnerMode {
    Sequential,
    Parallel,
    Tmux,
}

#[derive(Debug, Serialize)]
pub enum RunnerOpenai {
    Disabled,
    Enabled {
        api_key: String,
        api_base_url: String,
    },
}

#[derive(Debug, Serialize)]
pub struct RunnerTmux {
    pub kill_duplicate_session: bool,
    pub program: String,
    pub session_prefix: String,
    pub socket_path: String,
}
