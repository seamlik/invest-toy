use crate::config::Config;
use crate::ibkr_client::HistoricalMarketDataEntry;
use crate::stock_candidates::StockCandidates;
use crate::stock_data_downloader::ContractId;
use crate::stock_data_downloader::StockData;
use crate::stock_ranker::Ticker;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use std::rc::Rc;

pub struct ScoringFactorExtractor {
    config: Rc<Config>,
}

impl ScoringFactorExtractor {
    pub fn new(config: Rc<Config>) -> Self {
        Self { config }
    }

    pub fn extract_scoring_factors(&self, stock_data: &StockData) -> StockCandidates {
        let mut candidates = StockCandidates::from_config_overrides(&self.config.r#override);
        for position in &stock_data.portfolio {
            let conid = position.conid.into();
            let ticker: Ticker = position.ticker.as_str().into();

            // Extract P/E
            if let Some(snapshot) = stock_data.market_snapshot.get(&conid) {
                if let Some(notional) = snapshot.pe_ratio {
                    candidates.add_candidate(
                        ticker.clone(),
                        ScoringFactor::PeRatio,
                        notional.into(),
                    );
                }
                if let Some(notional) = snapshot.dividend_yield {
                    candidates.add_candidate(
                        ticker.clone(),
                        ScoringFactor::DividendYield,
                        notional.into(),
                    );
                }
            }

            // Long-term price change
            if let Some(notional) = extract_long_term_price_change(conid, stock_data) {
                candidates.add_candidate(
                    ticker.clone(),
                    ScoringFactor::LongTermChange,
                    notional.into(),
                )
            }

            // Short-term price change
            if let Some(notional) = extract_short_term_price_change(conid, stock_data) {
                candidates.add_candidate(
                    ticker.clone(),
                    ScoringFactor::ShortTermChange,
                    notional.into(),
                )
            }
        }
        candidates
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Deserialize)]
pub enum ScoringFactor {
    /// Price over earnings.
    PeRatio,

    /// Change of the stock price in the long term.
    LongTermChange,

    /// Change of the stock price in the short term.
    ShortTermChange,

    DividendYield,
}

fn extract_long_term_price_change(conid: ContractId, stock_data: &StockData) -> Option<f64> {
    let last_price = stock_data.market_snapshot.get(&conid)?.last_price?;
    let oldest_market_data = stock_data.long_term_market_history.get(&conid)?.first()?;
    let five_years = 1000 * 60 * 60 * 24 * 365 * 5;
    let now = Utc::now().timestamp_millis();
    if now - oldest_market_data.t < five_years {
        None
    } else {
        price_change(oldest_market_data.c, last_price)
    }
}

fn extract_short_term_price_change(conid: ContractId, stock_data: &StockData) -> Option<f64> {
    let last_price = stock_data.market_snapshot.get(&conid)?.last_price?;
    let history = stock_data.short_term_market_history.get(&conid)?;
    let price_on_last_month = last_month_entry(history)?.c;
    price_change(price_on_last_month, last_price)
}

fn price_change(old_price: f64, new_price: f64) -> Option<f64> {
    if old_price == 0.0 {
        None
    } else {
        Some((new_price - old_price) / old_price)
    }
}

fn last_month_entry(history: &[HistoricalMarketDataEntry]) -> Option<&HistoricalMarketDataEntry> {
    let now = Utc::now();
    history
        .iter()
        .rev()
        .find(|entry| within_1_month(entry.t, now))
}

fn within_1_month(timestamp: i64, now: DateTime<Utc>) -> bool {
    let duration = now.timestamp_millis() - timestamp;
    let one_month = 1000 * 60 * 60 * 24 * 30;
    one_month <= duration && duration <= one_month * 2
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;

    #[test_case::case(100.0 , 200.0 => Some(1.0)  ; "Positive change")]
    #[test_case::case(200.0 , 100.0 => Some(-0.5) ; "Negative change")]
    #[test_case::case(0.0   , 200.0 => None       ; "Division by 0")]
    fn price_change(old_price: f64, new_price: f64) -> Option<f64> {
        super::price_change(old_price, new_price)
    }

    #[test_case::case(vec![300, 200, 100]   => None      ; "All entries are too old")]
    #[test_case::case(vec![5, 4, 3]         => None      ; "All entries are too new")]
    #[test_case::case(vec![100, 40, 30, 10] => Some(2.0) ; "Found the latest matching entry")]
    fn last_month_entry(entries_on_days_before: Vec<i32>) -> Option<f64> {
        let history: Vec<_> = entries_on_days_before
            .into_iter()
            .enumerate()
            .map(|(index, days_before)| HistoricalMarketDataEntry {
                c: (index as u16).into(),
                t: (Utc::now() - Duration::days(days_before.into())).timestamp_millis(),
            })
            .collect();
        super::last_month_entry(&history).map(|entry| entry.c)
    }
}
