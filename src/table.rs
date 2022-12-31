use serde::Serialize;
use tokio::process::Command;

pub struct TablePrinter;

impl TablePrinter {
    pub async fn print<T: Serialize>(&self, table: &T) -> anyhow::Result<()> {
        if !build_process_to_print(table)?.status().await?.success() {
            anyhow::bail!("Failed in printing the table in Node.js")
        }
        Ok(())
    }
}

fn build_process_to_print<T: Serialize>(table: &T) -> anyhow::Result<Command> {
    let json = serde_json::to_string(table)?;
    let b64_encoded = base64::encode(json);
    let js = format!(
        "console.table(JSON.parse(Buffer.from('{}', 'base64')))",
        b64_encoded
    );
    let mut command = Command::new("node");
    command.arg("--eval").arg(js);

    Ok(command)
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Serialize)]
    struct Row {
        ticker: &'static str,
        score: f64,
    }

    #[tokio::test]
    async fn foo() -> anyhow::Result<()> {
        // Given
        let expected_output = r#"
┌─────────┬────────┬───────┐
│ (index) │ ticker │ score │
├─────────┼────────┼───────┤
│    0    │ 'TSLA' │  1.2  │
│    1    │ 'NESN' │   2   │
└─────────┴────────┴───────┘
        "#;
        let table = vec![
            Row {
                ticker: "TSLA",
                score: 1.2,
            },
            Row {
                ticker: "NESN",
                score: 2.0,
            },
        ];

        // When
        let actual_output_utf8 = super::build_process_to_print(&table)?
            .output()
            .await?
            .stdout;
        let actual_output = String::from_utf8(actual_output_utf8)?;

        // Then
        assert_eq!(expected_output.trim(), actual_output.trim());

        Ok(())
    }
}
