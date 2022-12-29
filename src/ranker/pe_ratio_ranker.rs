use super::FactorRanker;
use super::Name;
use super::Score;
use super::ScoringFactor;
use super::StockCandidates;
use std::collections::HashMap;

#[mockall_double::double]
use super::notional_ranker::NotionalRanker;

#[derive(Default)]
pub struct PeRatioRanker {
    notional_ranker: NotionalRanker,
}

impl FactorRanker for PeRatioRanker {
    fn rank(&self, candidates: &StockCandidates) -> HashMap<Name, Score> {
        let notional_candidates: HashMap<_, _> = candidates
            .iter()
            .filter_map(|(name, factors)| {
                factors
                    .get(&ScoringFactor::PeRatio)
                    .filter(|notional| notional.value > 0.0)
                    .cloned()
                    .map(|notional| (name.clone(), notional))
            })
            .collect();
        self.notional_ranker.rank_reversed(&notional_candidates)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ranker::Notional;

    #[test]
    fn rank_correct_candidates() {
        // Given
        let stock_candidates: StockCandidates = [
            (
                "A".into(),
                HashMap::from([(ScoringFactor::PeRatio, Notional::from(1.0))]),
            ),
            (
                "B".into(),
                HashMap::from([(ScoringFactor::ShortTermChange, Notional::from(1.0))]),
            ),
            (
                "C".into(),
                HashMap::from([(ScoringFactor::PeRatio, Notional::from(-1.0))]),
            ),
            (
                "D".into(),
                HashMap::from([(ScoringFactor::PeRatio, Notional::from(0.0))]),
            ),
        ]
        .into();
        let expected_notional_candidates: HashMap<_, _> = [("A".into(), 1.0.into())].into();
        let dummy_scores = HashMap::default();
        let mut notional_ranker = NotionalRanker::default();
        notional_ranker
            .expect_rank_reversed()
            .withf_st(move |arg| arg == &expected_notional_candidates)
            .return_const_st(dummy_scores.clone());
        let service = PeRatioRanker { notional_ranker };

        // When
        let actual_scores = service.rank(&stock_candidates);

        // Then
        assert_eq!(dummy_scores, actual_scores);
    }

    #[test]
    fn rank_no_candidate() {
        // Given
        let stock_candidates = StockCandidates::default();
        let expected_notional_candidates = HashMap::default();
        let dummy_scores = HashMap::default();
        let mut notional_ranker = NotionalRanker::default();
        notional_ranker
            .expect_rank_reversed()
            .withf_st(move |arg| arg == &expected_notional_candidates)
            .return_const_st(dummy_scores.clone());
        let service = PeRatioRanker { notional_ranker };

        // When
        let actual_scores = service.rank(&stock_candidates);

        // Then
        assert_eq!(dummy_scores, actual_scores);
    }
}
