use crate::config::{Config, Mode};
use std::ffi::OsStr;
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

    pub fn run(&self) -> anyhow::Result<()> {
        match self.config.mode() {
            Mode::Sequential => self.run_sequential(),
            Mode::Parallel => self.run_parallel(),
            Mode::Tmux => self.run_tmux(),
        }
    }

    fn run_sequential(&self) -> anyhow::Result<()> {
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

    fn run_parallel(&self) -> anyhow::Result<()> {
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

    fn run_tmux(&self) -> anyhow::Result<()> {
        let socket = "/tmp/tmux.workbench.sock";
        let session = format!("workbench-{}", "01" /* uuid::Uuid::new_v4() */);

        for (i, run) in self.config.runs().enumerate() {
            let (program, args) = run.cmd()?;

            let title = &format!("{} {}", program, args.join(" ")); // TODO: make it more robust
            let cmd_str = &format!("{} {}; read", program, args.join(" ")); // TODO: make it more robust

            // create the pane
            if i == 0 {
                self.tmux([
                    "-S",
                    socket,
                    "new-session",
                    "-d",
                    "-s",
                    &session,
                    "-c",
                    &self.cwd.to_string_lossy(),
                    cmd_str,
                ])?;
            } else {
                self.tmux([
                    "-S",
                    socket,
                    "split-window",
                    "-t",
                    &session,
                    "-v",
                    "-c",
                    &self.cwd.to_string_lossy(),
                    cmd_str,
                ])?;
            }

            // set pane title
            self.tmux(["-S", socket, "select-pane", "-t", &session, "-T", title])?;
        }

        for options in [
            ["status", "off"],
            ["pane-border-status", "top"],
            ["pane-border-format", "[#{pane_title}]"],
        ] {
            self.tmux(
                [
                    ["-S", socket, "set-option", "-s", "-t", &session].as_ref(),
                    &options,
                ]
                .concat(),
            )?;
        }

        self.tmux([
            "-S",
            socket,
            "attach-session",
            "-t",
            &session,
            "-f",
            "read-only",
        ])?;

        Ok(())
    }

    fn tmux<I, S>(&self, args: I) -> std::io::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut child = Command::new("tmux").args(args).spawn()?;
        let status = child.wait()?;

        // TODO: report status code in the pane title

        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "tmux command failed",
            ));
        }

        Ok(())
    }
}
