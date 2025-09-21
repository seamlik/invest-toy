use crate::arithmetic_renderer::ArithmeticRenderer;
use crate::ranker::Notional;
use crate::ranker::Score;
use crate::ranker::Ticker;
use crate::scoring_candidate::ScoringCandidates;
use crate::scoring_candidate::ScoringFactor;
use itertools::Itertools;
use schema::StockReport;
use std::collections::HashMap;

#[derive(Default)]
pub struct ReportRenderer {
    arithmetic_renderer: ArithmeticRenderer,
}

impl ReportRenderer {
    pub fn render(
        &self,
        candidates: &ScoringCandidates,
        scores: &HashMap<Ticker, Score>,
    ) -> Vec<StockReport> {
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
    ) -> StockReport {
        let none = "None".to_string();
        StockReport {
            ticker,
            score: self.render_score(score),
            one_month_price_change: factors
                .get(&ScoringFactor::OneMonthPriceChange)
                .map_or_else(
                    || none.clone(),
                    |v| self.arithmetic_renderer.render_percentage(v.value),
                ),
            long_term_total_return: factors
                .get(&ScoringFactor::LongTermTotalReturn)
                .map_or_else(
                    || none.clone(),
                    |v| self.arithmetic_renderer.render_percentage(v.value),
                ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn entries_sorted_by_score_descendingly() {
        // Given
        let renderer = ReportRenderer {
            arithmetic_renderer: ArithmeticRenderer,
        };
        let candidates: ScoringCandidates =
            [("A", Default::default()), ("B", Default::default())].into();
        let scores: HashMap<_, _> = [("A".into(), 1.0.into()), ("B".into(), 2.0.into())].into();
        let expected_tickers = vec!["B".to_string(), "A".to_string()];

        // When
        let actual_report = renderer.render(&candidates, &scores);
        let actual_tickers: Vec<_> = actual_report
            .into_iter()
            .map(|entry| entry.ticker)
            .collect();

        // Then
        assert_eq!(expected_tickers, actual_tickers);
    }
}
