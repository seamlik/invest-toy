mod ranker;
mod report;

use crate::ranker::StockCandidates;
use crate::ranker::StockRanker;
use crate::report::ReportRenderer;

fn main() -> anyhow::Result<()> {
    let candidates = StockCandidates::default();
    let scores = StockRanker::default().rank(&candidates);
    let report = ReportRenderer::render(&candidates, &scores);
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
