use crate::arithmetic_renderer::ArithmeticRenderer;
use crate::ranker::Score;
use crate::ranker::Ticker;
use itertools::Itertools;
use schema::StockAdvice;
use std::collections::HashMap;

#[derive(Default)]
pub struct InvestAdvisor {
    arithmetic_renderer: ArithmeticRenderer,
}

impl InvestAdvisor {
    pub fn render_advice(
        &self,
        scores: &HashMap<Ticker, Score>,
        invest_num: usize,
    ) -> Vec<StockAdvice> {
        let candidates: Vec<_> = scores
            .iter()
            .sorted_unstable_by(|(_, score_a), (_, score_b)| {
                score_b.value.total_cmp(&score_a.value)
            })
            .take(invest_num)
            .collect();
        let total_score = candidates.iter().map(|(_, score)| score.value).sum();
        candidates
            .into_iter()
            .map(|(ticker, score)| self.build_entry(ticker, score, total_score))
            .collect()
    }

    fn build_entry(&self, ticker: &Ticker, score: &Score, total_score: f64) -> StockAdvice {
        let ratio = score.value / total_score;
        let ratio_text = self.arithmetic_renderer.render_percentage(ratio);
        StockAdvice {
            ticker: ticker.to_string(),
            ratio: ratio_text,
        }
    }
}
