mod config;
mod ibkr_client;
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
use toy::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let config = if let Some(config_path) = &args.config {
        let config_yaml = tokio::fs::read_to_string(config_path).await?;
        Config::parse(&config_yaml)?
    } else {
        Default::default()
    };
    Toy::new(args, config.into()).run().await?;
    Ok(())
}
