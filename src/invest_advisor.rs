use crate::arithmetic_renderer::ArithmeticRenderer;
use crate::stock_ranker::Notional;
use crate::stock_ranker::Score;
use crate::stock_ranker::Ticker;
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;

pub struct InvestAdvisor {
    pub arithmetic_renderer: ArithmeticRenderer,
}

impl InvestAdvisor {
    pub fn render_advice(&self, scores: &HashMap<Ticker, Score>) -> Vec<InvestAdviceEntry> {
        let candidates: Vec<_> = scores
            .iter()
            .sorted_unstable_by(|(_, score_a), (_, score_b)| {
                score_b.value.total_cmp(&score_a.value)
            })
            .take(16)
            .collect();
        let total_score = candidates.iter().map(|(_, score)| score.value).sum();
        candidates
            .into_iter()
            .map(|(ticker, score)| self.build_entry(ticker, score, total_score))
            .collect()
    }

    fn build_entry(&self, ticker: &Ticker, score: &Score, total_score: f64) -> InvestAdviceEntry {
        let percentage = Notional {
            value: score.value / total_score,
        };
        let percentage = self.arithmetic_renderer.render_percentage(&percentage);
        InvestAdviceEntry {
            ticker: ticker.to_string(),
            percentage,
        }
    }
}

#[derive(Serialize)]
pub struct InvestAdviceEntry {
    ticker: String,
    percentage: String,
}
