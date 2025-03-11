use crate::ranker::Notional;
use crate::ranker::Ticker;
use schema::StockMetric;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct ScoringCandidates {
    map: HashMap<Ticker, HashMap<ScoringFactor, Notional>>,
}

impl ScoringCandidates {
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
    for ScoringCandidates
{
    fn from(value: [(&'static str, HashMap<ScoringFactor, Notional>); N]) -> Self {
        let map: HashMap<_, _> = value
            .into_iter()
            .map(|(ticker, factors)| (ticker.into(), factors))
            .collect();
        Self { map }
    }
}

pub struct ScoringCandidateExtractor;

impl ScoringCandidateExtractor {
    pub fn extract_scoring_candidates(&self, metrics: &[StockMetric]) -> ScoringCandidates {
        let mut candidates = ScoringCandidates::default();
        for stock in metrics {
            let ticker: Ticker = stock.ticker.as_str().into();

            if let Some(notional) = stock.dividend_yield {
                candidates.add_candidate(
                    ticker.clone(),
                    ScoringFactor::DividendYield,
                    notional.into(),
                );
            }
            if let Some(notional) = stock.price_change_in_one_month {
                candidates.add_candidate(
                    ticker.clone(),
                    ScoringFactor::PriceChangeIn1Month,
                    notional.into(),
                );
            }
            if let Some(notional) = stock.price_change_in_five_years {
                candidates.add_candidate(
                    ticker.clone(),
                    ScoringFactor::PriceChangeIn5Years,
                    notional.into(),
                );
            }
        }
        candidates
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum ScoringFactor {
    DividendYield,
    PriceChangeIn1Month,
    PriceChangeIn5Years,
}
