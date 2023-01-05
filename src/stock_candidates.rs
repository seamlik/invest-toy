use std::collections::HashMap;

use crate::stock_ranker::Notional;
use crate::stock_ranker::ScoringFactor;
use crate::stock_ranker::Ticker;

#[derive(Default)]
pub struct StockCandidates {
    map: HashMap<Ticker, HashMap<ScoringFactor, Notional>>,
}

impl StockCandidates {
    pub fn from_config_overrides(overrides: &HashMap<String, HashMap<ScoringFactor, f64>>) -> Self {
        let map = overrides
            .iter()
            .map(|(ticker, factors)| {
                (
                    ticker.as_str().into(),
                    convert_config_factors_for_ranker(factors),
                )
            })
            .collect();
        Self { map }
    }

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

fn convert_config_factors_for_ranker(
    factors: &HashMap<ScoringFactor, f64>,
) -> HashMap<ScoringFactor, Notional> {
    factors
        .iter()
        .map(|(factor, notional)| (*factor, (*notional).into()))
        .collect()
}
