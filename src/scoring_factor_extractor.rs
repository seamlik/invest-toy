use std::collections::HashMap;

use crate::config::Config;
use crate::ibkr_client::HistoricalMarketDataEntry;
use crate::stock_candidates::StockCandidates;
use crate::stock_data_downloader::ContractId;
use crate::stock_data_downloader::MarketSnapshot;
use crate::stock_data_downloader::StockData;
use crate::stock_ranker::Ticker;
use chrono::Utc;
use serde::Deserialize;

#[derive(Default)]
pub struct ScoringFactorExtractor;

impl ScoringFactorExtractor {
    pub fn extract_scoring_factors(
        &self,
        config: &Config,
        stock_data: &StockData,
    ) -> StockCandidates {
        let mut candidates = StockCandidates::from_config_overrides(&config.r#override);
        for position in &stock_data.portfolio {
            let conid = position.conid.into();
            let ticker: Ticker = position.ticker.as_str().into();

            // Extract P/E
            if let Some(notional) = stock_data
                .market_snapshot
                .get(&conid)
                .and_then(|snapshot| snapshot.pe_ratio)
            {
                candidates.add_candidate(ticker.clone(), ScoringFactor::PeRatio, notional.into());
            }

            // Long-term price change
            if let Some(notional) = extract_long_term_price_change(
                &position.conid.into(),
                &stock_data.market_snapshot,
                &stock_data.long_term_market_history,
            ) {
                candidates.add_candidate(
                    ticker.clone(),
                    ScoringFactor::LongTermChange,
                    notional.into(),
                )
            }

            // Short-term price change
            if let Some(notional) = extract_short_term_price_change(
                &position.conid.into(),
                &stock_data.market_snapshot,
                &stock_data.short_term_market_history,
            ) {
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
}

fn extract_long_term_price_change(
    conid: &ContractId,
    market_snapshot: &HashMap<ContractId, MarketSnapshot>,
    market_history: &HashMap<ContractId, Vec<HistoricalMarketDataEntry>>,
) -> Option<f64> {
    let last_price = market_snapshot.get(conid)?.last_price?;
    let oldest_market_data = market_history.get(conid)?.first()?;
    let milliseconds_of_5_years = 1000 * 60 * 60 * 24 * 365 * 5;
    let now = Utc::now().timestamp_millis();
    if oldest_market_data.c == 0.0 || now - oldest_market_data.t < milliseconds_of_5_years {
        None
    } else {
        Some((last_price - oldest_market_data.c) / oldest_market_data.c)
    }
}

fn extract_short_term_price_change(
    conid: &ContractId,
    market_snapshot: &HashMap<ContractId, MarketSnapshot>,
    market_history: &HashMap<ContractId, Vec<HistoricalMarketDataEntry>>,
) -> Option<f64> {
    let last_price = market_snapshot.get(conid)?.last_price?;
    let history = market_history.get(conid)?;
    let price_on_last_month = last_month_entry_of(history)?.c;
    if price_on_last_month == 0.0 {
        None
    } else {
        Some((last_price - price_on_last_month) / price_on_last_month)
    }
}

fn last_month_entry_of(
    history: &[HistoricalMarketDataEntry],
) -> Option<&HistoricalMarketDataEntry> {
    if history.len() < 30 {
        return None;
    }
    let milliseconds_of_1_month = 1000 * 60 * 60 * 24 * 30;
    let now = Utc::now().timestamp_millis();
    history
        .iter()
        .rev()
        .find(|entry| now - entry.t >= milliseconds_of_1_month)
}
