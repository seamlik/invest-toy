use crate::scoring_factor_extractor::ScoringFactor;
use crate::stock_candidates::StockCandidates;
use crate::stock_ranker::Notional;
use crate::stock_ranker::Score;
use crate::stock_ranker::Ticker;
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;

pub struct ReportRenderer;

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
            .map(|(ticker, factors, score)| render_entry(ticker, factors, score))
            .collect()
    }
}

fn render_entry(
    ticker: String,
    factors: &HashMap<ScoringFactor, Notional>,
    score: f64,
) -> ReportEntry {
    let none = "None".to_string();
    ReportEntry {
        ticker,
        score: render_score(score),
        pe_ratio: factors
            .get(&ScoringFactor::PeRatio)
            .map_or_else(|| none.clone(), |notional| render_float(notional.value)),
        short_term_change: factors
            .get(&ScoringFactor::ShortTermChange)
            .map_or_else(|| none.clone(), render_change),
        long_term_change: factors
            .get(&ScoringFactor::LongTermChange)
            .map_or_else(|| none.clone(), render_change),
    }
}

fn render_float(value: f64) -> String {
    format!("{:.2}", value)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .into()
}

fn render_change(change: &Notional) -> String {
    format!("{}%", render_float(change.value * 100.0))
}

fn render_score(score: f64) -> String {
    render_float(score * 100.0)
}

#[derive(Serialize, Default, PartialEq, Eq, Debug)]
pub struct ReportEntry {
    ticker: String,
    score: String,
    pe_ratio: String,
    short_term_change: String,
    long_term_change: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn entries_sorted_descendingly() {
        // Given
        let service = ReportRenderer;
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

    #[test]
    fn render_change() {
        assert_eq!("28.45%", super::render_change(&0.284513.into()));
    }

    #[test]
    fn render_score() {
        assert_eq!("123.46", super::render_score(1.23456));
    }

    #[test]
    fn render_float() {
        assert_eq!("0", super::render_float(0.0));
        assert_eq!("0.1", super::render_float(0.1));
        assert_eq!("12.35", super::render_float(12.3456));
    }
}
