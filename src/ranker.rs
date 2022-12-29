mod long_term_change_ranker;
mod notional_ranker;
mod pe_ratio_ranker;
mod short_term_change_ranker;

use self::long_term_change_ranker::LongTermChangeRanker;
use self::pe_ratio_ranker::PeRatioRanker;
use self::short_term_change_ranker::ShortTermChangeRanker;
use derive_more::Add;
use derive_more::From;
use itertools::Itertools;
use std::collections::HashMap;
use std::rc::Rc;

pub struct StockRanker {
    rankers: Vec<Box<dyn FactorRanker>>,
}

impl Default for StockRanker {
    fn default() -> Self {
        Self {
            rankers: vec![
                Box::new(LongTermChangeRanker::default()),
                Box::new(PeRatioRanker::default()),
                Box::new(ShortTermChangeRanker::default()),
            ],
        }
    }
}

impl StockRanker {
    pub fn rank(&self, candidates: &StockCandidates) -> HashMap<Name, Score> {
        self.rankers
            .iter()
            .map(|ranker| ranker.rank(candidates))
            .flat_map(IntoIterator::into_iter)
            .into_grouping_map()
            .sum()
    }
}

#[mockall::automock]
trait FactorRanker {
    fn rank(&self, candidates: &StockCandidates) -> HashMap<Name, Score>;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Name {
    value: Rc<str>,
}

#[derive(Clone, Copy, From, PartialEq)]
pub struct Notional {
    value: f64,
}

impl Eq for Notional {}

#[derive(Debug, From, PartialEq, Add, Clone, Copy)]
pub struct Score {
    value: f64,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum ScoringFactor {
    /// Price over earnings.
    PeRatio,

    /// Change of the stock price in the long term.
    LongTermChange,

    /// Change of the stock price in the short term.
    ShortTermChange,
}

pub type StockCandidates = HashMap<Name, HashMap<ScoringFactor, Notional>>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sum_scores() {
        // Given

        let score1: HashMap<_, _> = [("A".into(), 0.1.into()), ("B".into(), 0.2.into())].into();
        let mut ranker1 = MockFactorRanker::default();
        ranker1.expect_rank().return_const_st(score1);

        let score2: HashMap<_, _> = [("A".into(), 0.3.into())].into();
        let mut ranker2 = MockFactorRanker::default();
        ranker2.expect_rank().return_const_st(score2);

        let expected_scores: HashMap<_, _> =
            [("A".into(), 0.4.into()), ("B".into(), 0.2.into())].into();
        let service = StockRanker {
            rankers: vec![Box::new(ranker1), Box::new(ranker2)],
        };

        // When
        let actual_scores = service.rank(&Default::default());

        // Then
        assert_eq!(expected_scores, actual_scores);
    }
}
