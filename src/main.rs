mod config;
mod ranker;
mod report;

use crate::config::Config;
use crate::ranker::StockCandidates;
use crate::ranker::StockRanker;
use crate::report::ReportRenderer;
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

    // Dummy code to avoid unreachable warnings
    println!("{:?}", config);
    let candidates = StockCandidates::default();
    let scores = StockRanker::default().rank(&candidates);
    let report = ReportRenderer::render(&candidates, &scores);
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

#[derive(Parser)]
struct Cli {
    config: Option<PathBuf>,
}
