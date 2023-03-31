use crate::arithmetic_renderer::ArithmeticRenderer;
use crate::scoring_factor_extractor::ScoringFactor;
use crate::stock_candidates::StockCandidates;
use crate::stock_ranker::Notional;
use crate::stock_ranker::Score;
use crate::stock_ranker::Ticker;
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;

pub struct ReportRenderer {
    pub arithmetic_renderer: ArithmeticRenderer,
}

impl ReportRenderer {
    pub fn render(
        &self,
        candidates: &StockCandidates,
        scores: &HashMap<Ticker, Score>,
    ) -> Vec<ReportEntry> {
        candidates
            .iter()
            .map(|(ticker, factors)| {
                (
                    ticker.to_string(),
                    factors,
                    scores.get(ticker).cloned().unwrap_or_default().value,
                )
            })
            .sorted_unstable_by(|(_, _, x), (_, _, y)| y.total_cmp(x))
            .map(|(ticker, factors, score)| self.render_entry(ticker, factors, score))
            .collect()
    }

    fn render_score(&self, score: f64) -> String {
        self.arithmetic_renderer.render_float(score * 100.0)
    }

    fn render_entry(
        &self,
        ticker: String,
        factors: &HashMap<ScoringFactor, Notional>,
        score: f64,
    ) -> ReportEntry {
        let none = "None".to_string();
        ReportEntry {
            ticker,
            score: self.render_score(score),
            pe_ratio: factors.get(&ScoringFactor::PeRatio).map_or_else(
                || none.clone(),
                |notional| self.arithmetic_renderer.render_float(notional.value),
            ),
            dividend_yield: factors.get(&ScoringFactor::DividendYield).map_or_else(
                || none.clone(),
                |v| self.arithmetic_renderer.render_percentage(v),
            ),
            short_term_change: factors.get(&ScoringFactor::ShortTermChange).map_or_else(
                || none.clone(),
                |v| self.arithmetic_renderer.render_percentage(v),
            ),
            long_term_change: factors.get(&ScoringFactor::LongTermChange).map_or_else(
                || none.clone(),
                |v| self.arithmetic_renderer.render_percentage(v),
            ),
        }
    }
}

#[derive(Serialize, Default, PartialEq, Eq, Debug)]
pub struct ReportEntry {
    ticker: String,
    score: String,
    pe_ratio: String,
    dividend_yield: String,
    short_term_change: String,
    long_term_change: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn entries_sorted_descendingly() {
        // Given
        let service = ReportRenderer {
            arithmetic_renderer: ArithmeticRenderer,
        };
        let candidates: StockCandidates =
            [("A", Default::default()), ("B", Default::default())].into();
        let scores: HashMap<_, _> = [("A".into(), 1.0.into()), ("B".into(), 2.0.into())].into();
        let expected_tickers = vec!["B".to_string(), "A".to_string()];

        // When
        let actual_report = service.render(&candidates, &scores);
        let actual_tickers: Vec<_> = actual_report
            .into_iter()
            .map(|entry| entry.ticker)
            .collect();

        // Then
        assert_eq!(expected_tickers, actual_tickers);
    }
}
