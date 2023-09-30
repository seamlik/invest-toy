use std::collections::HashMap;

use crate::scoring_factor_extractor::ScoringFactor;
use crate::stock_ranker::Notional;
use crate::stock_ranker::Ticker;

#[derive(Default)]
pub struct StockCandidates {
    map: HashMap<Ticker, HashMap<ScoringFactor, Notional>>,
}

impl StockCandidates {
    pub fn add_candidate(
        &mut self,
        ticker: Ticker,
        factor_type: ScoringFactor,
        notional: Notional,
    ) {
        if let Some(factors) = self.map.get_mut(&ticker) {
            factors.insert(factor_type, notional);
        } else {
            let factors = [(factor_type, notional)].into();
            self.map.insert(ticker, factors);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Ticker, &HashMap<ScoringFactor, Notional>)> {
        self.map.iter()
    }
}

impl<const N: usize> From<[(&'static str, HashMap<ScoringFactor, Notional>); N]>
    for StockCandidates
{
    fn from(value: [(&'static str, HashMap<ScoringFactor, Notional>); N]) -> Self {
        let map: HashMap<_, _> = value
            .into_iter()
            .map(|(ticker, factors)| (ticker.into(), factors))
            .collect();
        Self { map }
    }
}
