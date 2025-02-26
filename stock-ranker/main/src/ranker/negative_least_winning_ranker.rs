use super::FactorRanker;
use super::Score;
use super::ScoringCandidates;
use super::Ticker;
use crate::scoring_candidate::ScoringFactor;
use std::collections::HashMap;

#[mockall_double::double]
use super::notional_ranker::NotionalRanker;

pub struct NegativeLeastWinningRanker {
    notional_ranker: NotionalRanker,
    factor_type: ScoringFactor,
}

impl NegativeLeastWinningRanker {
    pub fn new(factor_type: ScoringFactor) -> Self {
        Self {
            notional_ranker: Default::default(),
            factor_type,
        }
    }
}

impl FactorRanker for NegativeLeastWinningRanker {
    fn rank(&self, candidates: &ScoringCandidates) -> HashMap<Ticker, Score> {
        let notional_candidates: HashMap<_, _> = candidates
            .iter()
            .filter_map(|(name, factors)| {
                factors
                    .get(&self.factor_type)
                    .filter(|notional| notional.value < 0.0)
                    .map(|notional| notional.value.abs().into())
                    .map(|notional| (name.clone(), notional))
            })
            .collect();
        self.notional_ranker.rank(&notional_candidates)
    }
    fn get_factor(&self) -> ScoringFactor {
        self.factor_type
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ranker::Notional;

    #[test]
    fn rank_correct_candidates() {
        let stock_candidates: ScoringCandidates = [
            (
                "A",
                HashMap::from([(ScoringFactor::PriceChangeIn1Month, Notional::from(-1.0))]),
            ),
            (
                "B",
                HashMap::from([(ScoringFactor::PriceChangeIn1Month, Notional::from(-2.0))]),
            ),
            (
                "C",
                HashMap::from([(ScoringFactor::PriceChangeIn1Month, Notional::from(1.0))]),
            ),
            (
                "D",
                HashMap::from([(ScoringFactor::PriceChangeIn1Month, Notional::from(0.0))]),
            ),
        ]
        .into();
        let expected_notional_candidates: HashMap<_, _> =
            [("A".into(), 1.0.into()), ("B".into(), 2.0.into())].into();
        let mut notional_ranker = NotionalRanker::default();
        notional_ranker
            .expect_rank()
            .withf_st(move |arg| arg == &expected_notional_candidates)
            .return_const_st(HashMap::default());
        let ranker = NegativeLeastWinningRanker {
            notional_ranker,
            factor_type: ScoringFactor::PriceChangeIn1Month,
        };

        // Then
        ranker.rank(&stock_candidates);
    }

    #[test]
    fn rank_no_candidate() {
        // Given
        let stock_candidates = ScoringCandidates::default();
        let expected_notional_candidates = HashMap::default();
        let expected_scores = HashMap::default();
        let mut notional_ranker = NotionalRanker::default();
        notional_ranker
            .expect_rank()
            .withf_st(move |arg| arg == &expected_notional_candidates)
            .return_const_st(expected_scores.clone());
        let ranker = NegativeLeastWinningRanker {
            notional_ranker,
            factor_type: ScoringFactor::PriceChangeIn1Month,
        };

        // When
        let actual_scores = ranker.rank(&stock_candidates);

        // Then
        assert_eq!(expected_scores, actual_scores);
    }
}
