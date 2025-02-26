use super::Notional;
use super::Score;
use super::Ticker;
use std::collections::HashMap;

#[derive(Default)]
pub struct NotionalRanker;

#[mockall::automock]
impl NotionalRanker {
    pub fn rank(&self, candidates: &HashMap<Ticker, Notional>) -> HashMap<Ticker, Score> {
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rank() {
        // Given
        let candidates: HashMap<_, _> = [
            ("A".into(), 1.0.into()),
            ("B".into(), 2.0.into()),
            ("C".into(), 3.0.into()),
            ("D".into(), 4.0.into()),
        ]
        .into();
        let expected_scores: HashMap<_, _> = [
            ("A".into(), 0.1.into()),
            ("B".into(), 0.2.into()),
            ("C".into(), 0.3.into()),
            ("D".into(), 0.4.into()),
        ]
        .into();

        // When
        let actual_sores = NotionalRanker.rank(&candidates);

        // Then
        assert_eq!(expected_scores, actual_sores);
    }

    #[test]
    fn rank_empty() {
        // Given
        let candidates = HashMap::default();
        let expected_scores = HashMap::default();

        // When
        let actual_sores = NotionalRanker.rank(&candidates);

        // Then
        assert_eq!(expected_scores, actual_sores);
    }
}
