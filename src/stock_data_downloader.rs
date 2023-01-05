use crate::ibkr_client::HistoricalMarketDataEntry;
use crate::ibkr_client::IbkrClient;
use crate::ibkr_client::PortfolioPosition;
use anyhow::Context;
use derive_more::From;
use std::collections::HashMap;

const FIELD_ID_LAST_PRICE: i32 = 31;
const FIELD_ID_PE_RATIO: i32 = 7290;

#[derive(Default)]
pub struct StockDataDownloader {
    ibkr_client: IbkrClient,
}

impl StockDataDownloader {
    pub async fn download_long_term_market_history(
        &self,
        conids: &[i32],
    ) -> reqwest::Result<HashMap<ContractId, Vec<HistoricalMarketDataEntry>>> {
        let mut result = HashMap::default();
        for conid in conids.iter().cloned() {
            let history = self.ibkr_client.market_history(conid, "6y", "1m").await?;
            result.insert(conid.into(), history);
        }
        Ok(result)
    }

    pub async fn download_short_term_market_history(
        &self,
        conids: &[i32],
    ) -> reqwest::Result<HashMap<ContractId, Vec<HistoricalMarketDataEntry>>> {
        let mut result = HashMap::default();
        for conid in conids.iter().cloned() {
            let history = self.ibkr_client.market_history(conid, "2m", "1d").await?;
            result.insert(conid.into(), history);
        }
        Ok(result)
    }

    pub async fn download_market_snapshot(
        &self,
        conids: &[i32],
    ) -> anyhow::Result<HashMap<ContractId, MarketSnapshot>> {
        let fields = [FIELD_ID_PE_RATIO];
        let market_snapshot_raw = self.ibkr_client.market_snapshot(conids, &fields).await?;
        let market_snapshot = market_snapshot_raw
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?;
        if market_snapshot.len() != conids.len() {
            anyhow::bail!("Number of market snapshot entries does not match `conids`")
        }
        let contract_ids = conids.iter().cloned().map(From::from);
        let result = std::iter::zip(contract_ids, market_snapshot).collect();
        Ok(result)
    }

    pub async fn download_portfolio(
        &self,
        account_id: &str,
    ) -> reqwest::Result<Vec<PortfolioPosition>> {
        // Fetch the first page always
        let mut current_page_index = 0;
        let mut positions = self.download_portfolio_at_page(account_id, 0).await?;

        let mut current_page_size = positions.len();
        while current_page_size >= 30 {
            current_page_index += 1;
            let next_page = self
                .download_portfolio_at_page(account_id, current_page_index)
                .await?;
            current_page_size = next_page.len();
            positions.extend(next_page.into_iter())
        }

        Ok(positions)
    }

    async fn download_portfolio_at_page(
        &self,
        account_id: &str,
        page_index: usize,
    ) -> reqwest::Result<Vec<PortfolioPosition>> {
        let mut portfolio = self.ibkr_client.portfolio(account_id, page_index).await?;

        // Filter out non-stock entries because IBKR somehow keeps showing forex in my portfolio.
        // Filter out entries with 0 position because IBKR still include stocks I recently sold.
        portfolio.retain(|entry| entry.assetClass == "STK" && entry.position != 0.0);

        Ok(portfolio)
    }
}

pub struct MarketSnapshot {
    pub last_price: Option<f64>,
    pub pe_ratio: Option<f64>,
}

impl TryFrom<HashMap<String, String>> for MarketSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let pe_ratio = extract_pe_ratio(&value)?;
        let last_price = extract_last_price(&value)?;
        let result = Self {
            pe_ratio,
            last_price,
        };
        Ok(result)
    }
}

fn extract_pe_ratio(data: &HashMap<String, String>) -> anyhow::Result<Option<f64>> {
    data.get(&FIELD_ID_PE_RATIO.to_string())
        .map(|raw| raw.parse())
        .transpose()
        .context("Failed to parse P/E")
}

fn extract_last_price(data: &HashMap<String, String>) -> anyhow::Result<Option<f64>> {
    data.get(&FIELD_ID_LAST_PRICE.to_string())
        .map(|raw| raw.trim_start_matches('C').trim_start_matches('H'))
        .map(|raw| raw.parse())
        .transpose()
        .context("Failed to parse last price")
}

#[derive(From, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ContractId {
    value: i32,
}
