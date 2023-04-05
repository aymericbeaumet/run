use crate::config::{Config, Mode, Run};
use std::ffi::OsStr;
use tokio::process::Command;

pub struct RunnerOpts {
    pub tags: Vec<String>,
}

pub struct Runner {
    config: Config,
    opts: RunnerOpts,
}

impl Runner {
    pub fn new(config: Config, opts: RunnerOpts) -> Self {
        Runner { config, opts }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        match self.config.mode {
            Mode::Sequential => self.run_sequential().await,
            Mode::Parallel => self.run_parallel().await,
            Mode::Tmux => self.run_tmux().await,
        }
    }

    async fn run_sequential(&mut self) -> anyhow::Result<()> {
        // TODO: print a message to announce which commands we are going to run

        for run in self.filtered_runs() {
            let (program, args) = run.cmd.parse()?; // TODO: move .parse() step during config parsing

            let mut child = Command::new(program)
                .args(args)
                .current_dir(&self.config.workdir)
                .spawn()?;

            child.wait().await?;
        }

        Ok(())
    }

    async fn run_parallel(&mut self) -> anyhow::Result<()> {
        // TODO: print a message to announce which commands we are going to run

        let mut children = vec![];

        for run in self.filtered_runs() {
            let (program, args) = run.cmd.parse()?; // TODO: move .parse() step during config parsing
            children.push(
                Command::new(program)
                    .args(args)
                    .current_dir(&self.config.workdir)
                    .spawn()?,
            );
        }

        for mut child in children {
            child.wait().await?;
        }

        Ok(())
    }

    async fn run_tmux(&mut self) -> anyhow::Result<()> {
        let session = format!(
            "{}{}",
            self.config.tmux.session_prefix, "01" /* uuid::Uuid::new_v4() */
        );

        if self.config.tmux.kill_duplicate_session {
            if let Err(err) = self.tmux(["kill-session", "-t", &session]).await {
                println!("[debug] failed to kill duplicate session: {}", err); // TODO: use log library
            }
        }

        for (i, run) in self.filtered_runs().enumerate() {
            let (program, args) = run.cmd.parse()?; // TODO: move .parse() step during config parsing

            let workdir = &self.config.workdir.to_string_lossy();
            let title = format!("{} {}", program, args.join(" ")); // TODO: make it more robust
            let title = title.trim();
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
            self.tmux(["select-pane", "-t", &session, "-T", title])
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
            ["window-status-current-format", "~ WORKBENCH ~"],
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

        Ok(self.tmux(["attach-session", "-t", &session]).await?)
    }

    async fn tmux<I, S>(&self, args: I) -> std::io::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new(&self.config.tmux.program);
        cmd.args(["-S", &self.config.tmux.socket_path]);
        cmd.args(args);

        let mut child = cmd.spawn()?;

        let status = child.wait().await?;
        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "tmux command failed",
            ));
        }

        // TODO: report status code in the pane title
        // TODO: report time to finish in the pane title

        Ok(())
    }

    fn filtered_runs(&self) -> impl Iterator<Item = &Run> {
        self.config.runs.iter().filter(|run| {
            run.tags.is_empty() || run.tags.iter().all(|tag| self.opts.tags.contains(tag))
        })
    }
}
