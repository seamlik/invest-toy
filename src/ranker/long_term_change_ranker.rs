use super::notional_ranker::NotionalRanker;
use super::Candidates;
use super::FactorRanker;
use super::Name;
use super::Score;
use super::ScoringFactor;
use std::collections::HashMap;

#[derive(Default)]
pub struct LongTermChangeRanker {
    notional_ranker: NotionalRanker,
}

impl FactorRanker for LongTermChangeRanker {
    fn rank(&self, candidates: &Candidates) -> HashMap<Name, Score> {
        let notional_candidates: HashMap<_, _> = candidates
            .iter()
            .filter_map(|(name, factors)| {
                factors
                    .get(&ScoringFactor::LongTermChange)
                    .filter(|notional| notional.value > 0.0)
                    .cloned()
                    .map(|notional| (name.clone(), notional))
            })
            .collect();
        self.notional_ranker.rank(&notional_candidates)
    }
}
