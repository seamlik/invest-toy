use itertools::Itertools;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;

pub struct IbkrClient;

impl IbkrClient {
    pub async fn market_snapshot(
        &self,
        conids: &[i32],
        fields: &[i32],
    ) -> reqwest::Result<Vec<HashMap<String, String>>> {
        let conids_text = conids.iter().join(",");
        let fields_text = fields.iter().join(",");
        let endpoint = format!(
            "iserver/marketdata/snapshot?conids={}&fields={}",
            conids_text, fields_text
        );
        fetch(&endpoint).await
    }

    pub async fn i_server_accounts(&self) -> reqwest::Result<IServerAccount> {
        fetch("iserver/accounts").await
    }

    pub async fn portfolio_accounts(&self) -> reqwest::Result<Vec<PortfolioAccount>> {
        fetch("portfolio/accounts").await
    }

    pub async fn portfolio(
        &self,
        account_id: &str,
        page_index: usize,
    ) -> reqwest::Result<Vec<PortfolioPosition>> {
        let endpoint = format!("portfolio/{}/positions/{}", account_id, page_index);
        fetch(&endpoint).await
    }

    pub async fn long_term_market_history(
        &self,
        conid: i32,
        period_years: usize,
    ) -> reqwest::Result<Vec<HistoricalMarketDataEntry>> {
        let result = self
            .market_history(conid, &format!("{}y", period_years), "1m")
            .await?
            .data;
        Ok(result)
    }

    pub async fn short_term_market_history(
        &self,
        conid: i32,
    ) -> reqwest::Result<Vec<HistoricalMarketDataEntry>> {
        let result = self.market_history(conid, "2m", "1d").await?.data;
        Ok(result)
    }

    async fn market_history(
        &self,
        conid: i32,
        chart_period: &str,
        chart_bar: &str,
    ) -> reqwest::Result<HistoricalMarketData> {
        let endpoint = format!(
            "iserver/marketdata/history?conid={}&period={}&bar={}&outsideRth=false",
            conid, chart_period, chart_bar
        );
        fetch(&endpoint).await
    }
}

async fn fetch<T>(endpoint: &str) -> reqwest::Result<T>
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
        .await?
        .json()
        .await?;
    Ok(response)
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

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct HistoricalMarketDataEntry {
    ///Price at market close
    pub c: f64,

    ///Timestamp
    pub t: i64,
}
