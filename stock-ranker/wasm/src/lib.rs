use schema::{Input, Output};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn rank(input_json: String) -> String {
    let input: Input = match serde_json::from_str(&input_json) {
        Ok(i) => i,
        Err(e) => {
            return e.to_string();
        }
    };
    let output = stock_ranker::rank(input).unwrap_or_else(|| Output {
        advice: Default::default(),
        report: Default::default(),
    });
    serde_json::to_string(&output).unwrap_or_else(|e| e.to_string())
}
