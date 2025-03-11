mod advisor;
mod arithmetic_renderer;
mod ranker;
mod report;
mod scoring_candidate;

use crate::advisor::InvestAdvisor;
use crate::ranker::StockRanker;
use crate::report::ReportRenderer;
use crate::scoring_candidate::ScoringCandidateExtractor;
use anyhow::Context;
use anyhow::anyhow;
use schema::Output;
use schema::StockMetric;
use serde::Serialize;
use std::io::Write;
use std::io::stdin;
use std::process::Command;
use std::process::Stdio;

fn main() -> anyhow::Result<()> {
    let input: Vec<StockMetric> =
        serde_json::from_reader(stdin()).context("Failed to deserialize the input as JSON")?;
    let output = rank(input)?;

    println!("Stock performance report:");
    print_json_as_table(&output.report, include_str!("Print-Report.ps1"))?;
    println!("Investment advice for this month:");
    print_json_as_table(&output.advice, include_str!("Print-Advice.ps1"))?;

    Ok(())
}

fn rank(metrics: Vec<StockMetric>) -> anyhow::Result<Output> {
    if metrics.is_empty() {
        anyhow::bail!("No stock metric in the input")
    }

    let invest_count = metrics.len() / 2;
    let candidates = ScoringCandidateExtractor.extract_scoring_candidates(&metrics);
    let scores = StockRanker::default().rank(&candidates);
    let report = ReportRenderer::default().render(&candidates, &scores);
    let advice = InvestAdvisor::default().render_advice(&scores, invest_count);
    Ok(Output { report, advice })
}

fn print_json_as_table(data: impl Serialize, script: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string(&data).context("Failed to serialize the data as JSON")?;
    let mut process = Command::new("pwsh")
        .args(["-Command", script])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to launch PowerShell")?;
    process
        .stdin
        .take()
        .ok_or_else(|| anyhow!("Failed to get stdin"))?
        .write_all(json.as_bytes())
        .context("Failed to write string into PowerShell")?;
    process
        .wait()
        .context("Failed to wait for PowerShell process")?;
    Ok(())
}
