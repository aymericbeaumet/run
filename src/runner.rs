/*
use crate::config::Mode;
use crate::pipeline::Pipeline;
use crate::selector::Selector;
use anyhow::Context;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};

pub struct Runner {
    openai: bool,
    openai_api_key: Option<String>,
    selector: Selector,
}

impl Runner {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self.config.mode {
            Mode::Sequential => self.run_sequential().await,
            Mode::Parallel => self.run_parallel().await,
            Mode::Tmux => self.run_tmux().await,
        }
    }

    async fn run_sequential(&self) -> anyhow::Result<()> {
        for (id, run) in self.filtered_runs() {
            // TODO: clean + refactor
            let mut workdir = self.config.workdir.as_ref().to_path_buf();
            if let Some(wd) = run.workdir.as_ref() {
                let mut abs = workdir.to_path_buf();
                abs.push(wd);
                workdir = abs;
            }

            let mut child = self.exec(&run.cmd, &workdir).await?;

            Pipeline::new(format!("[{}]", id), self.openapi_api_key.clone())
                .process(child.stdout.take().unwrap(), child.stderr.take().unwrap())
                .await?;

            child.wait().await?;
        }

        Ok(())
    }

    async fn run_parallel(&self) -> anyhow::Result<()> {
        let mut children = vec![];

        for (id, run) in self.filtered_runs() {
            // TODO: clean + refactor
            let mut workdir = self.config.workdir.as_ref().to_path_buf();
            if let Some(wd) = run.workdir.as_ref() {
                let mut abs = workdir.to_path_buf();
                abs.push(wd);
                workdir = abs;
            }

            let child = self.exec(&run.cmd, &workdir).await?;

            let _pipeline = Pipeline::new(format!("[{}]", id), self.openapi_api_key.clone());
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
            self.config.tmux.session_prefix, "01" /* uuid::Uuid::new_v4() */
        );

        if self.config.tmux.kill_duplicate_session {
            if let Err(err) = self.tmux(["kill-session", "-t", &session]).await {
                println!("[debug] failed to kill duplicate session: {}", err); // TODO: use log library
            }
        }

        for (i, (id, run)) in self.filtered_runs().enumerate() {
            let program = &run.cmd[0];
            let args = &run.cmd[1..];

            // TODO: clean + refactor
            let mut workdir = self.config.workdir.as_ref().to_path_buf();
            if let Some(wd) = run.workdir.as_ref() {
                let mut abs = workdir.to_path_buf();
                abs.push(wd);
                workdir = abs;
            }

            let workdir = &workdir.to_string_lossy();
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
            self.tmux(["select-pane", "-t", &session, "-T", &run.title(id)])
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
        let mut cmd = Command::new(&self.config.tmux.program);
        cmd.args(["-S", &self.config.tmux.socket_path]);
        cmd.args(args);

        let mut child = cmd
            .spawn()
            .with_context(|| format!("could not spawn {:?}", &self.config.tmux.program))?;

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

    //fn filtered_runs(&self) -> impl Iterator<Item = (&String, &Run)> {
    //self.config.runs.iter().filter(|(_, run)| {
    //self.required_tags.is_empty()
    //|| self
    //.required_tags
    //.iter()
    //.all(|required_tag| run.tags.contains(required_tag))
    //})
    //}
}
*/
