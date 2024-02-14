use super::FactorRanker;
use super::Score;
use super::StockCandidates;
use super::Ticker;
use crate::scoring_factor_extractor::ScoringFactor;
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
    fn rank(&self, candidates: &StockCandidates) -> HashMap<Ticker, Score> {
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
    use crate::stock_ranker::Notional;

    #[test]
    fn rank_correct_candidates() {
        let stock_candidates: StockCandidates = [
            (
                "A",
                HashMap::from([(ScoringFactor::PriceEma20Change, Notional::from(-1.0))]),
            ),
            (
                "B",
                HashMap::from([(ScoringFactor::PriceEma200Change, Notional::from(-1.0))]),
            ),
            (
                "C",
                HashMap::from([(ScoringFactor::PriceEma20Change, Notional::from(1.0))]),
            ),
            (
                "D",
                HashMap::from([(ScoringFactor::PriceEma20Change, Notional::from(0.0))]),
            ),
        ]
        .into();
        let expected_notional_candidates: HashMap<_, _> = [("A".into(), 1.0.into())].into();
        let expected_scores = HashMap::default();
        let mut notional_ranker = NotionalRanker::default();
        notional_ranker
            .expect_rank()
            .withf_st(move |arg| arg == &expected_notional_candidates)
            .return_const_st(expected_scores.clone());
        let ranker = NegativeLeastWinningRanker {
            notional_ranker,
            factor_type: ScoringFactor::PriceEma20Change,
        };

        // When
        let actual_scores = ranker.rank(&stock_candidates);

        // Then
        assert_eq!(expected_scores, actual_scores);
    }

    #[test]
    fn rank_no_candidate() {
        // Given
        let stock_candidates = StockCandidates::default();
        let expected_notional_candidates = HashMap::default();
        let expected_scores = HashMap::default();
        let mut notional_ranker = NotionalRanker::default();
        notional_ranker
            .expect_rank()
            .withf_st(move |arg| arg == &expected_notional_candidates)
            .return_const_st(expected_scores.clone());
        let ranker = NegativeLeastWinningRanker {
            notional_ranker,
            factor_type: ScoringFactor::PriceEma20Change,
        };

        // When
        let actual_scores = ranker.rank(&stock_candidates);

        // Then
        assert_eq!(expected_scores, actual_scores);
    }
}
