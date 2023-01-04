use crate::stock_ranker::ScoringFactor;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, PartialEq, Default)]
pub struct Config {
    pub r#override: HashMap<String, HashMap<ScoringFactor, f64>>,
}

impl Config {
    pub fn parse(yaml: &str) -> serde_yaml::Result<Self> {
        serde_yaml::from_str(yaml)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() -> anyhow::Result<()> {
        // Given
        let sample_yaml = r#"
          override:
            NESN:
              PeRatio: 1
              LongTermChange: 2
              ShortTermChange: 3
            TSLA:
              PeRatio: 4
        "#;
        let expected_config = Config {
            r#override: [
                (
                    "NESN".into(),
                    [
                        (ScoringFactor::PeRatio, 1.0),
                        (ScoringFactor::LongTermChange, 2.0),
                        (ScoringFactor::ShortTermChange, 3.0),
                    ]
                    .into(),
                ),
                ("TSLA".into(), [(ScoringFactor::PeRatio, 4.0)].into()),
            ]
            .into(),
        };

        // When
        let actual_config = Config::parse(sample_yaml)?;

        // Then
        assert_eq!(expected_config, actual_config);
        Ok(())
    }
}
