use crate::config::{Config, Mode};
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::process::Command;

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

    pub async fn run(&self) -> anyhow::Result<()> {
        match self.config.mode {
            Mode::Sequential => self.run_sequential().await,
            Mode::Parallel => self.run_parallel().await,
            Mode::Tmux => self.run_tmux().await,
        }
    }

    async fn run_sequential(&self) -> anyhow::Result<()> {
        for run in &self.config.runs {
            let (program, args) = run.cmd.parse()?;
            Command::new(program)
                .args(args)
                .current_dir(&self.cwd)
                .spawn()?
                .wait()
                .await?;
        }

        Ok(())
    }

    async fn run_parallel(&self) -> anyhow::Result<()> {
        let mut children = vec![];

        for run in &self.config.runs {
            let (program, args) = run.cmd.parse()?;
            children.push(
                Command::new(program)
                    .args(args)
                    .current_dir(&self.cwd)
                    .spawn()?,
            );
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

        for (i, run) in self.config.runs.iter().enumerate() {
            let (program, args) = run.cmd.parse()?;

            let cwd = &self.cwd.to_string_lossy();
            let title = format!("{} {}", program, args.join(" ")); // TODO: make it more robust
            let title = title.trim();
            let cmd_str = &format!("{} {}; read", program, args.join(" ")); // TODO: make it more robust

            // create the pane
            if i == 0 {
                self.tmux(["new-session", "-s", &session, "-d", "-c", cwd, cmd_str])
                    .await?;
            } else {
                self.tmux(["split-window", "-t", &session, "-v", "-c", cwd, cmd_str])
                    .await?;
            }

            // set pane title
            self.tmux(["select-pane", "-t", &session, "-T", title])
                .await?;
        }

        self.tmux(["select-layout", "-t", &session, "even-vertical"])
            .await?;

        // TODO: unbind-key -a

        for options in [
            ["mouse", "on"],
            // status
            ["status", "on"],
            ["status-justify", "absolute-centre"],
            ["status-left", ""],
            ["status-left-length", "0"],
            ["status-right", ""],
            ["status-right-length", "0"],
            ["window-status-current-format", "~ WORKBENCH ~"],
            // pane
            ["pane-border-format", "[#{pane_title}]"],
            ["pane-border-indicators", "off"],
            ["pane-border-lines", "single"],
            ["pane-border-status", "top"],
            // theme
            ["status-style", "fg=white bg=orange"],
            ["pane-border-style", "fg=white bg=orange"],
            ["pane-active-border-style", "fg=white bg=orange"],
        ] {
            self.tmux([["set-option", "-t", &session, "-s"].as_ref(), &options].concat())
                .await?;
        }

        self.tmux(["attach-session", "-t", &session]).await?;

        Ok(())
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

        Ok(())
    }
}
