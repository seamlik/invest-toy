use itertools::Itertools;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

#[mockall_double::double]
use crate::file_writer::FileWriter;

#[derive(Default)]
pub struct IbkrClient {
    file_writer: FileWriter,
}

#[mockall::automock]
impl IbkrClient {
    pub async fn market_snapshot(
        &self,
        conids: &[i64],
        fields: &[i32],
    ) -> anyhow::Result<Vec<HashMap<String, Value>>> {
        let conids_text = conids.iter().join(",");
        let fields_text = fields.iter().join(",");
        let endpoint = format!(
            "iserver/marketdata/snapshot?conids={}&fields={}",
            conids_text, fields_text
        );
        let data = fetch(&endpoint).await?;
        self.file_writer
            .write(&write_path("ibkr-market-snapshot.json"), data.as_bytes())
            .await?;
        serde_json::from_str(&data).map_err(Into::into)
    }

    pub async fn i_server_accounts(&self) -> anyhow::Result<IServerAccount> {
        let data = fetch("iserver/accounts").await?;
        serde_json::from_str(&data).map_err(Into::into)
    }

    pub async fn portfolio_accounts(&self) -> anyhow::Result<Vec<PortfolioAccount>> {
        let data = fetch("portfolio/accounts").await?;
        serde_json::from_str(&data).map_err(Into::into)
    }

    pub async fn portfolio(
        &self,
        account_id: &str,
        page_index: usize,
    ) -> anyhow::Result<Vec<PortfolioPosition>> {
        let endpoint = format!("portfolio/{}/positions/{}", account_id, page_index);
        let data = fetch(&endpoint).await?;
        serde_json::from_str(&data).map_err(Into::into)
    }
}

async fn fetch(endpoint: &str) -> anyhow::Result<String> {
    let endpoint_full = format!("https://127.0.0.1:5000/v1/api/{}", endpoint);
    let response = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?
        .get(&endpoint_full)
        .header("User-Agent", "IBKR Toy")
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        anyhow::bail!("REST endpoint {} error {}: {}", endpoint_full, status, text);
    }
    Ok(text)
}

fn write_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(name);
    path
}

#[derive(Deserialize)]
pub struct IServerAccount {
    pub accounts: Vec<String>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct PortfolioAccount {
    pub accountId: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[allow(non_snake_case)]
pub struct PortfolioPosition {
    pub conid: i64,
    pub ticker: String,
    pub position: f64,
    pub assetClass: String,
}
