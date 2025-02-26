#[derive(Default)]
pub struct ArithmeticRenderer;

impl ArithmeticRenderer {
    pub fn render_float(&self, value: f64) -> String {
        format!("{:.2}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .into()
    }

    pub fn render_percentage(&self, percentage: f64) -> String {
        format!("{}%", self.render_float(percentage * 100.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::case;

    #[case(0.0 => "0"         ; "0")]
    #[case(0.1 => "0.1"       ; "No rounding")]
    #[case(12.3456 => "12.35" ; "Full decimal with rounding")]
    fn render_float(value: f64) -> String {
        ArithmeticRenderer.render_float(value)
    }

    #[test]
    fn render_change() {
        assert_eq!("28.45%", ArithmeticRenderer.render_percentage(0.284513));
    }
}
