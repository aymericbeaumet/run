use crate::executor::Executor;
use crate::processors;
use anyhow::Context;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use serde::Serialize;
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::process::Command;

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
        for cmd in &self.options.commands {
            self.exec(cmd).await?;
        }

        Ok(())
    }

    async fn run_parallel(&self) -> anyhow::Result<()> {
        let mut waits = FuturesUnordered::new();
        for cmd in &self.options.commands {
            waits.push(self.exec(cmd));
        }

        while let Some(res) = waits.next().await {
            res?;
        }

        Ok(())
    }

    async fn run_tmux(&self) -> anyhow::Result<()> {
        let session = format!("{}{}", self.options.tmux.session_prefix, "01");

        if self.options.tmux.kill_duplicate_session {
            if let Err(err) = self.tmux(["kill-session", "-t", &session]).await {
                println!("[debug] failed to kill duplicate session: {}", err); // TODO: use log library
            }
        }

        for (i, run) in self.options.commands.iter().enumerate() {
            let workdir = &run.workdir.to_string_lossy();
            let cmd_str = &format!("{}; read", shell_words::join(&run.cmd));

            // create the pane
            if i == 0 {
                self.tmux(["new-session", "-s", &session, "-d", "-c", workdir, cmd_str])
                    .await?;
            } else {
                self.tmux(["split-window", "-t", &session, "-v", "-c", workdir, cmd_str])
                    .await?;
            }

            // set pane title
            self.tmux(["select-pane", "-t", &session, "-T", &run.name])
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

        Ok(())
    }

    async fn exec(&self, cmd: &RunnerCommand) -> anyhow::Result<()> {
        let mut executor = Executor::default();

        if let RunnerOpenai::Enabled {
            api_base_url,
            api_key,
        } = &self.options.openai
        {
            executor.push_err(processors::Openai::new(
                api_base_url.clone(),
                api_key.clone(),
            ));
        }

        if let RunnerPrefix::Enabled = self.options.prefix {
            executor.push_out(processors::Prefix::new(format!("[{}]", &cmd.name)));
            executor.push_err(processors::Prefix::new(format!("[{}]", &cmd.name)));
        }

        executor.exec(&cmd.cmd, &cmd.workdir).await
    }
}

#[derive(Debug, Serialize)]
pub struct RunnerOptions {
    pub commands: Vec<RunnerCommand>,
    pub mode: RunnerMode,
    pub openai: RunnerOpenai,
    pub prefix: RunnerPrefix,
    pub tmux: RunnerTmux,
}

#[derive(Debug, Serialize)]
pub struct RunnerCommand {
    pub cmd: Vec<String>,
    pub description: Option<String>,
    pub envs: Vec<String>,
    pub name: String,
    pub tags: Vec<String>,
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
pub enum RunnerPrefix {
    Disabled,
    Enabled,
}

#[derive(Debug, Serialize)]
pub struct RunnerTmux {
    pub kill_duplicate_session: bool,
    pub program: String,
    pub session_prefix: String,
    pub socket_path: String,
}
