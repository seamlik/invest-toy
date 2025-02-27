mod advisor;
mod arithmetic_renderer;
mod ranker;
mod report;
mod scoring_candidate;

use crate::advisor::InvestAdvisor;
use crate::ranker::StockRanker;
use crate::report::ReportRenderer;
use crate::scoring_candidate::ScoringCandidateExtractor;
use schema::Input;
use schema::Output;

pub fn rank(input: Input) -> Option<Output> {
    let metrics = input.metrics?;
    if metrics.is_empty() {
        return None;
    }

    let invest_count = metrics.len() / 2;
    let candidates = ScoringCandidateExtractor.extract_scoring_candidates(&metrics);
    let scores = StockRanker::default().rank(&candidates);
    let report = ReportRenderer::default().render(&candidates, &scores);
    let advice = InvestAdvisor::default().render_advice(&scores, invest_count);
    Some(Output { report, advice })
}
