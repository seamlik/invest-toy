mod negative_least_winning_ranker;
mod notional_ranker;
mod positive_greatest_winning_ranker;

use self::negative_least_winning_ranker::NegativeLeastWinningRanker;
use self::positive_greatest_winning_ranker::PositiveGreatestWinningRanker;
use crate::scoring_candidate::ScoringCandidates;
use crate::scoring_candidate::ScoringFactor;
use derive_more::Add;
use derive_more::Display;
use derive_more::From;
use derive_more::Mul;
use itertools::Itertools;
use std::collections::HashMap;
use std::rc::Rc;

pub struct StockRanker {
    rankers: Vec<Box<dyn FactorRanker>>,
    factor_weight: HashMap<ScoringFactor, f64>,
}

impl Default for StockRanker {
    fn default() -> Self {
        Self {
            rankers: vec![
                Box::new(PositiveGreatestWinningRanker::new(
                    ScoringFactor::DividendYield,
                )),
                Box::new(NegativeLeastWinningRanker::new(
                    ScoringFactor::PriceChangeIn1Month,
                )),
                Box::new(PositiveGreatestWinningRanker::new(
                    ScoringFactor::PriceChangeIn5Years,
                )),
            ],
            factor_weight: HashMap::from([
                (ScoringFactor::DividendYield, 1.0),
                (ScoringFactor::PriceChangeIn1Month, 4.0),
                (ScoringFactor::PriceChangeIn5Years, 5.0),
            ]),
        }
    }
}

impl StockRanker {
    pub fn rank(&self, candidates: &ScoringCandidates) -> HashMap<Ticker, Score> {
        self.rankers
            .iter()
            .flat_map(|ranker| self.calculate_weighted_rank(ranker.as_ref(), candidates))
            .into_grouping_map()
            .sum()
    }
    fn calculate_weighted_rank(
        &self,
        ranker: &dyn FactorRanker,
        candidates: &ScoringCandidates,
    ) -> HashMap<Ticker, Score> {
        let weight = self
            .factor_weight
            .get(&ranker.get_factor())
            .unwrap_or_else(|| panic!("No weight registered for factor {:?}", ranker.get_factor()));
        ranker
            .rank(candidates)
            .into_iter()
            .map(|(ticker, score)| (ticker, score * weight))
            .collect()
    }
}

#[mockall::automock]
trait FactorRanker {
    fn rank(&self, candidates: &ScoringCandidates) -> HashMap<Ticker, Score>;
    fn get_factor(&self) -> ScoringFactor;
}

/// Code name of a stock.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Display)]
pub struct Ticker {
    value: Rc<str>,
}
impl From<&str> for Ticker {
    fn from(value: &str) -> Self {
        Self {
            value: value.into(),
        }
    }
}
impl From<String> for Ticker {
    fn from(value: String) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Clone, Copy, From, PartialEq, Debug)]
pub struct Notional {
    pub value: f64,
}

impl Eq for Notional {}

#[derive(Debug, From, PartialEq, Add, Clone, Copy, Default, Mul)]
pub struct Score {
    pub value: f64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sum_scores() {
        let score1: HashMap<_, _> = [("A".into(), 100.0.into()), ("B".into(), 200.0.into())].into();
        let mut ranker1 = MockFactorRanker::default();
        ranker1.expect_rank().return_const_st(score1);
        ranker1
            .expect_get_factor()
            .return_const_st(ScoringFactor::DividendYield);

        let score2: HashMap<_, _> = [("A".into(), 300.0.into())].into();
        let mut ranker2 = MockFactorRanker::default();
        ranker2.expect_rank().return_const_st(score2);
        ranker2
            .expect_get_factor()
            .return_const_st(ScoringFactor::PriceChangeIn1Month);

        let expected_scores: HashMap<_, _> =
            [("A".into(), 70.0.into()), ("B".into(), 20.0.into())].into();
        let service = StockRanker {
            rankers: vec![Box::new(ranker1), Box::new(ranker2)],
            factor_weight: HashMap::from([
                (ScoringFactor::DividendYield, 0.1),
                (ScoringFactor::PriceChangeIn1Month, 0.2),
            ]),
        };

        // When
        let actual_scores = service.rank(&Default::default());

        // Then
        assert_eq!(expected_scores, actual_scores);
    }
}
