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
    pub fn rank(&self, candidates: &Candidates) -> HashMap<Name, Score> {
        self.rankers
            .iter()
            .map(|ranker| ranker.rank(candidates))
            .flat_map(IntoIterator::into_iter)
            .into_grouping_map()
            .sum()
    }
}

trait FactorRanker {
    fn rank(&self, candidates: &Candidates) -> HashMap<Name, Score>;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Name {
    value: Rc<str>,
}

#[derive(Clone, Copy, From)]
pub struct Notional {
    value: f64,
}

#[derive(Debug, From, PartialEq, Add)]
pub struct Score {
    value: f64,
}

#[derive(Hash, PartialEq, Eq)]
pub enum ScoringFactor {
    /// Price over earnings.
    PeRatio,

    /// Change of the stock price in the long term.
    LongTermChange,

    /// Change of the stock price in the short term.
    ShortTermChange,
}

pub type Candidates = HashMap<Name, HashMap<ScoringFactor, Notional>>;
