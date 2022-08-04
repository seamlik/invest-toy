use crate::cli::Cli;
use crate::cli::Format;
use chrono::prelude::*;
use clap::Parser;
use itertools::Itertools;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::Client;
use reqwest::ClientBuilder;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

const DEFAULT_GATEWAY: &str = "https://localhost:5000/v1/api/";

pub struct Toy {
    client: Client,
    header: HeaderMap,
    account_id: String,
    cli: Cli,
}

impl Toy {
    pub async fn run(&mut self) -> anyhow::Result<()> {
        // Account ID
        let accounts: Vec<Account> = self
            .client
            .get(format!("{}portfolio/accounts", DEFAULT_GATEWAY))
            .headers(self.header.clone())
            .send()
            .await?
            .json()
            .await?;
        if let Some(first_account) = accounts.first() {
            self.account_id = first_account.accountId.clone();
            log::info!("Account ID: {}", &self.account_id);
        } else {
            log::info!("No account found");
            return Ok(());
        }

        let positions_by_conid = self.portfolio().await?;
        log::info!("Found {} stocks", positions_by_conid.len());

        // Finalize report
        let mut report: Vec<ReportRecord> = Default::default();
        for conid in positions_by_conid.keys() {
            if let Some(position) = positions_by_conid.get(conid) {
                let mut from_date = None;
                let mut from_price = None;
                let mut to_date = None;
                let mut to_price = None;
                let mut change = None;
                let mut cursor = self.historical_price(*conid).await?.into_iter();
                if let Some(earlist_data) = cursor.next() {
                    from_date = Some(earlist_data.t);
                    from_price = Some(earlist_data.c);
                    if let Some(latest_data) = cursor.last() {
                        to_date = Some(latest_data.t);
                        to_price = Some(latest_data.c);
                        if earlist_data.c != 0.0 {
                            change = Some((latest_data.c - earlist_data.c) / earlist_data.c);
                        }
                    }
                }

                let record = ReportRecord {
                    ticker: position.ticker.clone(),
                    from_date,
                    from_price,
                    to_date,
                    to_price,
                    change,
                };
                report.push(record);
            }
        }
        report.sort_by_cached_key(|record| record.ticker.clone());

        match self.cli.format {
            Format::debug => report.iter().for_each(|record| println!("{:?}", record)),
            Format::bson => {
                let bson = bson::to_vec(&report)?;
                tokio::io::stdout().write_all(&bson).await?;
            }
        }

        Ok(())
    }

    async fn portfolio_at_page(&self, page_index: usize) -> anyhow::Result<Vec<PortfolioPosition>> {
        let response = self
            .client
            .get(format!(
                "{}portfolio/{}/positions/{}",
                DEFAULT_GATEWAY, &self.account_id, page_index,
            ))
            .headers(self.header.clone())
            .send()
            .await?;
        let positions: Vec<PortfolioPosition> = if response.status().is_success() {
            response.json().await?
        } else {
            anyhow::bail!("{}: {}", response.status(), response.text().await?)
        };
        Ok(positions)
    }

    /// Gets all positions mapped by their contract IDs.
    async fn portfolio(&self) -> anyhow::Result<HashMap<u64, PortfolioPosition>> {
        // Fetch the first page always
        let mut current_page_index = 0_usize;
        let mut positions: HashMap<u64, PortfolioPosition> = self
            .portfolio_at_page(current_page_index)
            .await?
            .into_iter()
            .map(Self::position_by_conid)
            .collect();
        let mut current_page_size = positions.len();

        while current_page_size >= 30 {
            current_page_index += 1;
            let next_page = self.portfolio_at_page(current_page_index).await?;
            current_page_size = next_page.len();
            positions.extend(next_page.into_iter().map(Self::position_by_conid));
        }

        Ok(positions)
    }

    fn position_by_conid(position: PortfolioPosition) -> (u64, PortfolioPosition) {
        (position.conid, position)
    }

    async fn historical_price(&self, conid: u64) -> anyhow::Result<Vec<HistoricalDataRecord>> {
        let response = self
            .client
            .get(format!(
                "{}iserver/marketdata/history?conid={}&period=1m&bar=1d&outsideRth=true",
                DEFAULT_GATEWAY, conid,
            ))
            .headers(self.header.clone())
            .send()
            .await?;
        let historical_data: HistoricalData = if response.status().is_success() {
            response.json().await?
        } else {
            anyhow::bail!("{}: {}", response.status(), response.text().await?)
        };
        let sorted_data = historical_data
            .data
            .into_iter()
            .sorted_by_key(|record| record.t)
            .collect_vec();
        Ok(sorted_data)
    }
}

impl Default for Toy {
    fn default() -> Self {
        let client = ClientBuilder::default()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let mut header: HeaderMap<HeaderValue> = HeaderMap::default();
        header.append("User-Agent", "Rust".parse().unwrap());

        Toy {
            client,
            header,
            account_id: Default::default(),
            cli: Cli::parse(),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct PortfolioPosition {
    conid: u64,
    ticker: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct Account {
    accountId: String,
}

#[derive(Deserialize)]
struct HistoricalData {
    data: Vec<HistoricalDataRecord>,
}

#[derive(Deserialize)]
struct HistoricalDataRecord {
    c: f64,

    #[serde(with = "chrono::serde::ts_milliseconds")]
    t: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct ReportRecord {
    ticker: String,

    #[serde(with = "crate::serde::option_time")]
    from_date: Option<DateTime<Utc>>,

    from_price: Option<f64>,

    #[serde(with = "crate::serde::option_time")]
    to_date: Option<DateTime<Utc>>,

    to_price: Option<f64>,
    change: Option<f64>,
}
