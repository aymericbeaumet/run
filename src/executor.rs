use async_trait::async_trait;
use tokio::io::AsyncRead;
use tokio::io::{AsyncBufReadExt, BufReader};

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
    pub fn push_out(&mut self, processor: Box<dyn Processor + Send + Sync>) {
        self.out_processors.push(processor);
    }

    pub fn push_err(&mut self, processor: Box<dyn Processor + Send + Sync>) {
        self.err_processors.push(processor);
    }

    pub async fn process(
        &mut self,
        out: impl AsyncRead + Unpin,
        err: impl AsyncRead + Unpin,
    ) -> anyhow::Result<()> {
        let mut out = BufReader::new(out).lines();
        let mut err = BufReader::new(err).lines();

        // TODO: read both streams in parallel
        // TODO: do not catch stdout/stderr if no processors for stream

        while let Some(mut line) = out.next_line().await? {
            for processor in &mut self.out_processors  {
                line = processor.process(line)?;
            }
            println!("{}", line);
        }

        for processor in &mut self.out_processors {
            processor.flush().await?;
        }

        while let Some(mut line) = err.next_line().await? {
            for processor in &mut self.err_processors  {
                line = processor.process(line)?;
            }
            eprintln!("{}", line);
        }

        for processor in &mut self.err_processors {
            processor.flush().await?;
        }

        Ok(())
    }
}
