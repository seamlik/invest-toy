use crate::config::Config;
use crate::ibkr_client::HistoricalMarketDataEntry;
use crate::ibkr_client::PortfolioPosition;
use anyhow::Context;
use chrono::DateTime;
use chrono::Utc;
use derive_more::From;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::rc::Rc;

#[mockall_double::double]
use crate::ibkr_client::IbkrClient;

const FIELD_ID_LAST_PRICE: i32 = 31;
const FIELD_ID_PE_RATIO: i32 = 7290;
const PORTFOLIO_PAGE_SIZE: usize = 30;

pub struct StockDataDownloader {
    config: Rc<Config>,
    ibkr_client: IbkrClient,
}

impl StockDataDownloader {
    pub fn new(config: Rc<Config>) -> Self {
        Self {
            config,
            ibkr_client: Default::default(),
        }
    }

    pub async fn download_stock_data(&self, account_id: &str) -> anyhow::Result<StockData> {
        let portfolio = self.download_portfolio(account_id).await?;
        println!("Found {} stocks", portfolio.len());
        let portfolio: Vec<_> = portfolio
            .into_iter()
            .filter(|position| !self.config.r#override.contains_key(&position.ticker))
            .collect();

        let timestamp = Utc::now();
        let conids: Vec<_> = portfolio.iter().map(|position| position.conid).collect();
        let market_snapshot = self.download_market_snapshot(&conids).await?;
        let short_term_market_history = self.download_short_term_market_history(&conids).await?;
        let long_term_market_history = self.download_long_term_market_history(&conids).await?;

        let result = StockData {
            portfolio,
            market_snapshot,
            long_term_market_history,
            short_term_market_history,
            timestamp,
        };
        Ok(result)
    }

    async fn download_long_term_market_history(
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

    async fn download_short_term_market_history(
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

    async fn download_market_snapshot(
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

    async fn download_portfolio(
        &self,
        account_id: &str,
    ) -> reqwest::Result<Vec<PortfolioPosition>> {
        // Fetch the first page always
        let mut current_page_index = 0;
        let mut positions = self.download_portfolio_at_page(account_id, 0).await?;

        let mut current_page_size = positions.len();
        while current_page_size >= PORTFOLIO_PAGE_SIZE {
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

#[derive(Deserialize, Serialize)]
pub struct StockData {
    pub portfolio: Vec<PortfolioPosition>,
    pub market_snapshot: HashMap<ContractId, MarketSnapshot>,
    pub short_term_market_history: HashMap<ContractId, Vec<HistoricalMarketDataEntry>>,
    pub long_term_market_history: HashMap<ContractId, Vec<HistoricalMarketDataEntry>>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
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

#[derive(From, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct ContractId {
    value: i32,
}

#[cfg(test)]
mod test {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn download_portfolio_from_multiple_pages() {
        // Given
        let entries_at_last_page = 5;
        let mut ibkr_client = IbkrClient::default();
        ibkr_client
            .expect_portfolio()
            .with(always(), eq(0))
            .return_once(|_, _| Ok(build_portfolio_with_n_entries(PORTFOLIO_PAGE_SIZE)));
        ibkr_client
            .expect_portfolio()
            .with(always(), eq(1))
            .return_once(|_, _| Ok(build_portfolio_with_n_entries(PORTFOLIO_PAGE_SIZE)));
        ibkr_client
            .expect_portfolio()
            .with(always(), eq(2))
            .return_once(move |_, _| Ok(build_portfolio_with_n_entries(entries_at_last_page)));
        let service = StockDataDownloader {
            config: Default::default(),
            ibkr_client,
        };
        let expected_portfolio =
            build_portfolio_with_n_entries(PORTFOLIO_PAGE_SIZE * 2 + entries_at_last_page);

        // When
        let actual_portfolio = service.download_portfolio("").await.unwrap();

        // Then
        assert_eq!(expected_portfolio, actual_portfolio);
    }

    #[tokio::test]
    async fn download_portfolio_from_1_page() {
        // Given
        let entries_at_last_page = 3;
        let mut ibkr_client = IbkrClient::default();
        ibkr_client
            .expect_portfolio()
            .with(always(), eq(0))
            .return_once(move |_, _| Ok(build_portfolio_with_n_entries(entries_at_last_page)));
        let service = StockDataDownloader {
            config: Default::default(),
            ibkr_client,
        };
        let expected_portfolio = build_portfolio_with_n_entries(entries_at_last_page);

        // When
        let actual_portfolio = service.download_portfolio("").await.unwrap();

        // Then
        assert_eq!(expected_portfolio, actual_portfolio);
    }

    #[tokio::test]
    async fn download_portfolio_at_page() {
        // Given
        let raw_portfolio = vec![
            PortfolioPosition {
                conid: 0,
                ticker: "TICKER".into(),
                position: 10.0,
                assetClass: "STK".into(),
            },
            PortfolioPosition {
                conid: 0,
                ticker: "SOLD".into(),
                position: 0.0,
                assetClass: "STK".into(),
            },
            PortfolioPosition {
                conid: 0,
                ticker: "CZK".into(),
                position: 10.0,
                assetClass: "FX".into(),
            },
        ];
        let expected_portfolio = vec![PortfolioPosition {
            conid: 0,
            ticker: "TICKER".into(),
            position: 10.0,
            assetClass: "STK".into(),
        }];
        let mut ibkr_client = IbkrClient::default();
        ibkr_client
            .expect_portfolio()
            .return_once(move |_, _| Ok(raw_portfolio));
        let service = StockDataDownloader {
            config: Default::default(),
            ibkr_client,
        };

        // When
        let actual_portfolio = service.download_portfolio_at_page("", 0).await.unwrap();

        // Then
        assert_eq!(expected_portfolio, actual_portfolio);
    }

    #[test]
    fn market_snapshot_try_from() {
        // Given
        let raw: HashMap<_, _> = [
            (FIELD_ID_PE_RATIO.to_string(), "1".into()),
            (FIELD_ID_LAST_PRICE.to_string(), "2".into()),
        ]
        .into();
        let expected_market_snapshot = MarketSnapshot {
            pe_ratio: Some(1.0),
            last_price: Some(2.0),
        };

        // When
        let actual_market_snapshot: MarketSnapshot = raw.try_into().unwrap();

        // Then
        assert_eq!(expected_market_snapshot, actual_market_snapshot);
    }

    #[test_case::case(FIELD_ID_PE_RATIO.to_string() => Some(123.0))]
    #[test_case::case("unknown".into()              => None)]
    fn extract_pe_ratio(field_id: String) -> Option<f64> {
        let raw: HashMap<_, _> = [(field_id, "123".into())].into();
        super::extract_pe_ratio(&raw).unwrap()
    }

    #[test_case::case("C123" => 123.0)]
    #[test_case::case("H123" => 123.0)]
    #[test_case::case("123"  => 123.0)]
    fn extract_last_price_ok(value: &'static str) -> f64 {
        let raw: HashMap<_, _> = [(FIELD_ID_LAST_PRICE.to_string(), value.into())].into();
        super::extract_last_price(&raw).unwrap().unwrap()
    }

    #[test_case::case("A123")]
    #[test_case::case("ABC")]
    #[test_case::case("123C")]
    fn extract_last_price_err(value: &'static str) {
        let raw: HashMap<_, _> = [(FIELD_ID_LAST_PRICE.to_string(), value.into())].into();
        let parsed = super::extract_last_price(&raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn extract_last_price_none() {
        let raw: HashMap<_, _> = [("unknown".into(), "123".into())].into();
        let parsed = super::extract_last_price(&raw).unwrap();
        assert_eq!(None, parsed);
    }

    fn build_portfolio_with_n_entries(n: usize) -> Vec<PortfolioPosition> {
        (0..n)
            .map(|_| PortfolioPosition {
                position: 1.0,
                assetClass: "STK".into(),
                ..Default::default()
            })
            .collect()
    }
}
