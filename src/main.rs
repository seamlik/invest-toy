mod config;
mod ibkr_client;
mod report_renderer;
mod stock_ranker;
mod table_printer;
mod toy;

use crate::config::Config;
use crate::toy::Toy;
use clap::Parser;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let config = if let Some(config_path) = args.config {
        let config_yaml = tokio::fs::read_to_string(config_path).await?;
        Config::parse(&config_yaml)?
    } else {
        Default::default()
    };
    Toy::new(config).run().await?;
    Ok(())
}

#[derive(Parser)]
struct Cli {
    config: Option<PathBuf>,
}
