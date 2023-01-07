use itertools::Itertools;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct IbkrClient;

#[mockall::automock]
impl IbkrClient {
    pub async fn market_snapshot(
        &self,
        conids: &[i32],
        fields: &[i32],
    ) -> anyhow::Result<Vec<HashMap<String, Value>>> {
        let conids_text = conids.iter().join(",");
        let fields_text = fields.iter().join(",");
        let endpoint = format!(
            "iserver/marketdata/snapshot?conids={}&fields={}",
            conids_text, fields_text
        );
        fetch(&endpoint).await
    }

    pub async fn i_server_accounts(&self) -> anyhow::Result<IServerAccount> {
        fetch("iserver/accounts").await
    }

    pub async fn portfolio_accounts(&self) -> anyhow::Result<Vec<PortfolioAccount>> {
        fetch("portfolio/accounts").await
    }

    pub async fn portfolio(
        &self,
        account_id: &str,
        page_index: usize,
    ) -> anyhow::Result<Vec<PortfolioPosition>> {
        let endpoint = format!("portfolio/{}/positions/{}", account_id, page_index);
        fetch(&endpoint).await
    }

    pub async fn market_history(
        &self,
        conid: i32,
        chart_period: &str,
        chart_bar: &str,
    ) -> anyhow::Result<Vec<HistoricalMarketDataEntry>> {
        let endpoint = format!(
            "iserver/marketdata/history?conid={}&period={}&bar={}&outsideRth=false",
            conid, chart_period, chart_bar
        );
        let market_history: HistoricalMarketData = fetch(&endpoint).await?;
        Ok(market_history.data)
    }
}

async fn fetch<T>(endpoint: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let endpoint_full = format!("https://127.0.0.1:5000/v1/api/{}", endpoint);
    let response = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?
        .get(&endpoint_full)
        .header("User-Agent", "IBKR Toy")
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await?;
        anyhow::bail!("REST endpoint {} error {}: {}", endpoint_full, status, text);
    }
    let text = response.text().await?;
    Ok(serde_json::from_str(&text).unwrap())
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
    pub conid: i32,
    pub ticker: String,
    pub position: f64,
    pub assetClass: String,
}

#[derive(Deserialize)]
pub struct HistoricalMarketData {
    pub data: Vec<HistoricalMarketDataEntry>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct HistoricalMarketDataEntry {
    /// Price at market close
    pub c: f64,

    /// Timestamp
    pub t: i64,
}
