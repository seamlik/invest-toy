mod negative_least_winning_ranker;
mod notional_ranker;
mod positive_greatest_winning_ranker;
mod positive_least_winning_ranker;

use self::negative_least_winning_ranker::NegativeLeastWinningRanker;
use self::positive_greatest_winning_ranker::PositiveGreatestWinningRanker;
use self::positive_least_winning_ranker::PositiveLeastWinningRanker;
use crate::scoring_factor_extractor::ScoringFactor;
use crate::stock_candidates::StockCandidates;
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
                Box::new(PositiveLeastWinningRanker::new(ScoringFactor::PeRatio)),
                Box::new(NegativeLeastWinningRanker::new(
                    ScoringFactor::PriceEma20Change,
                )),
                Box::new(PositiveGreatestWinningRanker::new(
                    ScoringFactor::PriceEma200Change,
                )),
            ],
            factor_weight: HashMap::from([
                // Half of my stocks don't pay dividend, and even they do, it's not a significant
                // income. Let's not make it too pronounced in to decision making.
                (ScoringFactor::DividendYield, 1.0),
                // P/E ratio of some companies (especially PAH3, merely 3!) feel artificial.
                (ScoringFactor::PeRatio, 0.0),
                (ScoringFactor::PriceEma20Change, 4.0),
                (ScoringFactor::PriceEma200Change, 5.0),
            ]),
        }
    }
}

impl StockRanker {
    pub fn rank(&self, candidates: &StockCandidates) -> HashMap<Ticker, Score> {
        self.rankers
            .iter()
            .flat_map(|ranker| self.calculate_weighted_rank(ranker.as_ref(), candidates))
            .into_grouping_map()
            .sum()
    }
    fn calculate_weighted_rank(
        &self,
        ranker: &dyn FactorRanker,
        candidates: &StockCandidates,
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
    fn rank(&self, candidates: &StockCandidates) -> HashMap<Ticker, Score>;
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

#[derive(Clone, Copy, From, PartialEq)]
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
            .return_const_st(ScoringFactor::PeRatio);

        let expected_scores: HashMap<_, _> =
            [("A".into(), 70.0.into()), ("B".into(), 20.0.into())].into();
        let service = StockRanker {
            rankers: vec![Box::new(ranker1), Box::new(ranker2)],
            factor_weight: HashMap::from([
                (ScoringFactor::DividendYield, 0.1),
                (ScoringFactor::PeRatio, 0.2),
            ]),
        };

        // When
        let actual_scores = service.rank(&Default::default());

        // Then
        assert_eq!(expected_scores, actual_scores);
    }
}
