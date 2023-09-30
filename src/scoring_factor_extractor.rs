use crate::config::Config;
use crate::stock_candidates::StockCandidates;
use crate::stock_data_downloader::StockData;
use crate::stock_ranker::Ticker;
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

                // Extract dividend yield
                if let Some(notional) = snapshot.dividend_yield {
                    candidates.add_candidate(
                        ticker.clone(),
                        ScoringFactor::DividendYield,
                        notional.into(),
                    );
                }
            }
        }
        candidates
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Deserialize)]
pub enum ScoringFactor {
    /// Price over earnings.
    PeRatio,

    DividendYield,
}
