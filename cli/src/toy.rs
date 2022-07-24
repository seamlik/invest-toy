use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::Client;
use reqwest::ClientBuilder;
use serde::Deserialize;
use std::collections::HashMap;

const DEFAULT_GATEWAY: &str = "https://localhost:5000/v1/api/";

pub struct Toy {
    client: Client,
    header: HeaderMap,
    account_id: String,
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
            println!("Account ID: {}", &self.account_id);
        } else {
            println!("No account found");
            return Ok(());
        }

        let positions_by_conid = self.portfolio().await?;
        println!("Found {} stocks", positions_by_conid.len());

        // Calculate changes
        let mut changes_by_conid: HashMap<u64, f64> = Default::default();
        for conid in positions_by_conid.keys() {
            if let Some(historical_price) = self.historical_price(*conid).await? {
                if historical_price != 0.0 {
                    // To avoid division by 0
                    if let Some(position) = positions_by_conid.get(conid) {
                        let change = (position.mktPrice - historical_price) / historical_price;
                        changes_by_conid.insert(*conid, change);
                    }
                }
            }
        }

        // Finalize report
        let mut report: Vec<ReportRecord> = Default::default();
        for conid in positions_by_conid.keys() {
            if let Some(position) = positions_by_conid.get(conid) {
                let record = ReportRecord {
                    ticker: position.ticker.clone(),
                    change: changes_by_conid.get(conid).cloned(),
                };
                report.push(record);
            }
        }
        report.sort_by(|x, y| x.change.partial_cmp(&y.change).expect("One change is NaN"));
        for record in report.into_iter() {
            if let Some(change) = record.change {
                println!("Stock {} changed {:.2}%", &record.ticker, change * 100.0);
            } else {
                println!("Stock {} had unknown changes", &record.ticker);
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

    async fn historical_price(&self, conid: u64) -> anyhow::Result<Option<f64>> {
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
        let earliest_price = sorted_data.first().map(|record| record.c);
        Ok(earliest_price)
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
        }
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct PortfolioPosition {
    conid: u64,
    mktPrice: f64,
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

struct ReportRecord {
    ticker: String,
    change: Option<f64>,
}
