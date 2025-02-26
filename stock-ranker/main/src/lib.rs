mod advisor;
mod arithmetic_renderer;
mod ranker;
mod report;
mod scoring_candidate;

use crate::advisor::InvestAdvisor;
use crate::advisor::StockAdvice;
use crate::ranker::StockRanker;
use crate::report::ReportRenderer;
use crate::report::StockReport;
use crate::scoring_candidate::ScoringCandidateExtractor;
use serde::Serialize;

pub struct StockMetric {
    pub ticker: String,
    pub price_change_in_1_month: Option<f64>,
    pub price_change_in_5_years: Option<f64>,
    pub dividend_yield: Option<f64>,
}

#[derive(Serialize)]
pub struct Output {
    pub report: Vec<StockReport>,
    pub advice: Vec<StockAdvice>,
}

pub fn rank(metrics: Vec<StockMetric>) -> Output {
    let invest_count = metrics.len() / 2;
    let candidates = ScoringCandidateExtractor.extract_scoring_candidates(&metrics);
    let scores = StockRanker::default().rank(&candidates);
    let report = ReportRenderer::default().render(&candidates, &scores);
    let advice = InvestAdvisor::default().render_advice(&scores, invest_count);
    Output { report, advice }
}
