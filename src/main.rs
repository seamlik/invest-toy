mod arithmetic_renderer;
mod clock;
mod file_writer;
mod ibkr_client;
mod invest_advisor;
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
use toy::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    Toy::new(args).run().await?;
    Ok(())
}
