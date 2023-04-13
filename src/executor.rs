use anyhow::Context;
use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[async_trait]
pub trait Processor: Send + Sync {
    fn process(&mut self, line: String) -> anyhow::Result<String>;

    async fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct Executor {
    out_processors: Vec<Box<dyn Processor + Send + Sync>>,
    err_processors: Vec<Box<dyn Processor + Send + Sync>>,
}

impl Executor {
    pub fn push_out<P: Processor + Send + Sync + 'static>(&mut self, processor: P) {
        self.out_processors
            .push(Box::new(processor) as Box<dyn Processor + Send + Sync>);
    }

    pub fn push_err<P: Processor + Send + Sync + 'static>(&mut self, processor: P) {
        self.err_processors
            .push(Box::new(processor) as Box<dyn Processor + Send + Sync>);
    }

    pub async fn exec<P>(mut self, cmd: &[String], workdir: P) -> anyhow::Result<()>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let mut child = Command::new(&cmd[0]);
        let child = child
            .env_clear()
            .args(&cmd[1..])
            .current_dir(workdir.as_ref());

        if !self.out_processors.is_empty() {
            child.stdout(Stdio::piped());
        }

        if !self.err_processors.is_empty() {
            child.stderr(Stdio::piped());
        }

        let child = child
            .spawn()
            .with_context(|| format!("could not spawn {:?} in {:?}", &cmd, &workdir))?;

        // TODO: read both streams in parallel

        if !self.out_processors.is_empty() {
            if let Some(stdout) = child.stdout {
                let mut out = BufReader::new(stdout).lines();

                while let Some(mut line) = out.next_line().await? {
                    for processor in &mut self.out_processors {
                        line = processor.process(line)?;
                    }
                    println!("{}", line);
                }

                for processor in &mut self.out_processors {
                    processor.flush().await?;
                }
            }
        }

        if !self.err_processors.is_empty() {
            if let Some(stderr) = child.stderr {
                let mut err = BufReader::new(stderr).lines();

                while let Some(mut line) = err.next_line().await? {
                    for processor in &mut self.err_processors {
                        line = processor.process(line)?;
                    }
                    eprintln!("{}", line);
                }

                for processor in &mut self.err_processors {
                    processor.flush().await?;
                }
            }
        }

        Ok(())
    }
}
