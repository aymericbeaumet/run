mod config;
//mod pipeline;
//mod runner;
//mod selector;

use clap::Parser;
use config::Config;
use merge::Merge;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse arguments
    let args = Config::try_parse_from(std::env::args_os())?;
    let command_print_config = args.command_print_config;

    // First load the default arguments
    let mut config = Config::default();

    // Then load the config files
    for file in &args.files {
        config.merge(Config::load(file).await?);
    }

    // Then load the CLI args + env variables
    config.merge(args);

    if command_print_config {
        println!("{}", toml::to_string(&config)?);
        return Ok(());
    }

    Ok(())
    //Runner::new(options).run().await
}
