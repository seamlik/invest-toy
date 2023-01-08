mod clock;
mod config;
mod ibkr_client;
mod progress_bar;
mod report_renderer;
mod scoring_factor_extractor;
mod stock_candidates;
mod stock_data_cacher;
mod stock_data_downloader;
mod stock_ranker;
mod table_printer;
mod toy;

use crate::toy::Toy;
use clap::Parser;
use config::Config;
use std::path::PathBuf;
use toy::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let default_config_path = default_config_path()?;
    let config_path = args.config.as_ref().cloned().unwrap_or(default_config_path);
    let config_yaml = tokio::fs::read_to_string(config_path).await?;
    let config = Config::parse(&config_yaml)?;

    Toy::new(args, config.into()).run().await?;
    Ok(())
}

fn default_config_path() -> anyhow::Result<PathBuf> {
    let mut path =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?;
    path.push("ibkr-toy.yaml");
    Ok(path)
}
