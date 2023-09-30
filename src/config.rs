use crate::scoring_factor_extractor::ScoringFactor;
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
