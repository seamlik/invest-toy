use derive_more::From;
use std::collections::HashMap;
use std::rc::Rc;

pub struct NotionalRanker;

impl NotionalRanker {
    pub fn rank(candidates: &HashMap<Name, Notional>) -> HashMap<Name, Score> {
        let total_notional = candidates
            .values()
            .map(|x| x.value)
            .reduce(|x, y| x + y)
            .unwrap_or_default();
        candidates
            .iter()
            .map(|(name, notional)| (name.clone(), (notional.value / total_notional).into()))
            .collect()
    }

    pub fn rank_reversed(candidates: &HashMap<Name, Notional>) -> HashMap<Name, Score> {
        let mut names_sorted_by_notional: Vec<_> = candidates.keys().cloned().collect();
        names_sorted_by_notional.sort_unstable_by(|x, y| {
            let x_value = candidates.get(x).map_or(0.0, |notional| notional.value);
            let y_value = candidates.get(y).map_or(0.0, |notional| notional.value);
            x_value.total_cmp(&y_value)
        });

        let mut notional_sorted_reversed: Vec<_> = candidates.values().cloned().collect();
        notional_sorted_reversed.sort_unstable_by(|x, y| x.value.total_cmp(&y.value));
        notional_sorted_reversed.reverse();

        let candidates_reversed: HashMap<_, _> = names_sorted_by_notional
            .iter()
            .cloned()
            .zip(notional_sorted_reversed.iter().cloned())
            .collect();
        Self::rank(&candidates_reversed)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Name {
    value: Rc<str>,
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Clone, Copy, From)]
pub struct Notional {
    value: f64,
}

#[derive(Debug, From, PartialEq)]
pub struct Score {
    value: f64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rank() {
        // Given
        let candidates: HashMap<Name, Notional> = [
            ("A".into(), 1.0.into()),
            ("B".into(), 2.0.into()),
            ("C".into(), 3.0.into()),
            ("D".into(), 4.0.into()),
        ]
        .into();
        let expected_scores: HashMap<Name, Score> = [
            ("A".into(), 0.1.into()),
            ("B".into(), 0.2.into()),
            ("C".into(), 0.3.into()),
            ("D".into(), 0.4.into()),
        ]
        .into();

        // When
        let actual_sores = NotionalRanker::rank(&candidates);

        // Then
        assert_eq!(expected_scores, actual_sores);
    }

    #[test]
    fn rank_reversed() {
        // Given
        let candidates: HashMap<Name, Notional> = [
            ("A".into(), 1.0.into()),
            ("B".into(), 2.0.into()),
            ("C".into(), 3.0.into()),
            ("D".into(), 4.0.into()),
        ]
        .into();
        let expected_scores: HashMap<Name, Score> = [
            ("A".into(), 0.4.into()),
            ("B".into(), 0.3.into()),
            ("C".into(), 0.2.into()),
            ("D".into(), 0.1.into()),
        ]
        .into();

        // When
        let actual_sores = NotionalRanker::rank_reversed(&candidates);

        // Then
        assert_eq!(expected_scores, actual_sores);
    }
}
