use crate::executor::Executor;
use crate::processors;
use anyhow::Context;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::process::Command;

pub struct Runner {
    commands: Vec<RunnerCommand>,
    mode: RunnerMode,
    openai: RunnerOpenai,
    prefix: RunnerPrefix,
    tmux: RunnerTmux,
}

impl Runner {
    pub fn new(options: RunnerOptions) -> Self {
        let tags_priority: HashMap<String, usize> = options
            .tags
            .into_iter()
            .enumerate()
            .rev()
            .map(|(i, tag)| (tag, i))
            .collect();

        let commands = if tags_priority.is_empty() {
            options.commands
        } else {
            options
                .commands
                .into_iter()
                .filter(|cmd| {
                    !cmd.tags.is_empty() && cmd.tags.iter().any(|t| tags_priority.contains_key(t))
                })
                .sorted_by(|a, b| {
                    let a_tags = a.tags.iter().filter_map(|t| tags_priority.get(t));
                    let b_tags = b.tags.iter().filter_map(|t| tags_priority.get(t));
                    a_tags.cmp(b_tags)
                })
                .collect()
        };

        Self {
            commands,
            mode: options.mode,
            openai: options.openai,
            prefix: options.prefix,
            tmux: options.tmux,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        match self.mode {
            RunnerMode::Sequential => self.run_sequential().await,
            RunnerMode::Parallel => self.run_parallel().await,
            RunnerMode::Tmux => self.run_tmux().await,
        }
    }

    async fn run_sequential(&self) -> anyhow::Result<()> {
        for cmd in &self.commands {
            self.exec(cmd).await?;
        }

        Ok(())
    }

    async fn run_parallel(&self) -> anyhow::Result<()> {
        let mut waits = FuturesUnordered::new();
        for cmd in &self.commands {
            waits.push(self.exec(cmd));
        }

        while let Some(res) = waits.next().await {
            res?;
        }

        Ok(())
    }

    async fn run_tmux(&self) -> anyhow::Result<()> {
        let session_id = "01"; // TODO: make this configurable/unique
        let session = format!("{}{}", self.tmux.session_prefix, session_id);

        if self.tmux.kill_duplicate_session {
            if let Err(err) = self.tmux(["kill-session", "-t", &session]).await {
                println!("[debug] failed to kill duplicate session: {}", err); // TODO: use log library
            }
        }

        for (i, cmd) in self.commands.iter().enumerate() {
            let workdir = &cmd.workdir.to_string_lossy();
            let cmd_str = &format!("{}; read", cmd.to_command_line());

            // create the pane
            if i == 0 {
                self.tmux(["new-session", "-s", &session, "-d", "-c", workdir, cmd_str])
                    .await?;
            } else {
                self.tmux(["split-window", "-t", &session, "-v", "-c", workdir, cmd_str])
                    .await?;
            }

            // set pane title
            self.tmux(["select-pane", "-t", &session, "-T", &cmd.name])
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
        let mut cmd = Command::new(&self.tmux.program);
        cmd.args(["-S"]);
        cmd.args([&self.tmux.socket_path]);
        cmd.args(args);

        let mut child = cmd
            .spawn()
            .with_context(|| format!("could not spawn {:?}", &self.tmux.program))?;

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
        } = &self.openai
        {
            executor.push_err(processors::Openai::new(
                api_base_url.clone(),
                api_key.clone(),
            ));
        }

        if let RunnerPrefix::Enabled = self.prefix {
            executor.push_out(processors::Prefix::new(format!("[{}]", &cmd.name)));
            executor.push_err(processors::Prefix::new(format!("[{}]", &cmd.name)));
        }

        executor
            .exec(&cmd.program, &cmd.args, &cmd.workdir, cmd.envs.clone())
            .await
    }
}

#[derive(Debug, Serialize)]
pub struct RunnerOptions {
    pub commands: Vec<RunnerCommand>,
    pub mode: RunnerMode,
    pub openai: RunnerOpenai,
    pub prefix: RunnerPrefix,
    pub tags: Vec<String>,
    pub tmux: RunnerTmux,
}

#[derive(Debug, Serialize)]
pub struct RunnerCommand {
    pub program: String,
    pub args: Vec<String>,
    pub description: Option<String>,
    pub envs: Vec<(String, String)>,
    pub name: String,
    pub tags: Vec<String>,
    pub workdir: PathBuf,
}

impl RunnerCommand {
    fn to_command_line(&self) -> String {
        let mut args = vec![];

        args.push(self.program.as_str());

        for arg in &self.args {
            args.push(arg.as_str());
        }

        shell_words::join(args)
    }
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
    pub socket_path: PathBuf,
}
