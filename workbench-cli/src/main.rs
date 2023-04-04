use clap::Parser;
use std::path::{Path, PathBuf};
use workbench::{Config, Runner};

#[derive(Parser, Debug)]
struct Args {
    config_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::try_parse()?;
    let (config, config_file) = Config::load(args.config_file).await?;
    let cwd = Path::parent(&config_file).expect("the config file must be in a directory");

    let mut runner = Runner::new(config, cwd);
    runner.run().await
}
