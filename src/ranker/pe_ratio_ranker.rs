use super::notional_ranker::NotionalRanker;
use super::Candidates;
use super::FactorRanker;
use super::Name;
use super::Score;
use super::ScoringFactor;
use std::collections::HashMap;

#[derive(Default)]
pub struct PeRatioRanker {
    notional_ranker: NotionalRanker,
}

impl FactorRanker for PeRatioRanker {
    fn rank(&self, candidates: &Candidates) -> HashMap<Name, Score> {
        let notional_candidates: HashMap<_, _> = candidates
            .iter()
            .filter_map(|(name, factors)| {
                factors
                    .get(&ScoringFactor::PeRatio)
                    .cloned()
                    .map(|notional| (name.clone(), notional))
            })
            .collect();
        self.notional_ranker.rank_reversed(&notional_candidates)
    }
}
