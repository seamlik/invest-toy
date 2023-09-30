use crate::config::Config;
use crate::ibkr_client::PortfolioPosition;
use anyhow::bail;
use anyhow::Context;
use chrono::DateTime;
use chrono::Utc;
use derive_more::From;
use serde::de::Unexpected;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::rc::Rc;

#[mockall_double::double]
use crate::ibkr_client::IbkrClient;

#[mockall_double::double]
use crate::clock::Clock;

const ASSERT_CLASS_STOCK: &str = "STK";
const PORTFOLIO_PAGE_SIZE: usize = 30;

const FIELD_ID_DIVIDEND_YIELD: i32 = 7287;
const FIELD_ID_PE_RATIO: i32 = 7290;
const FIELD_ID_SYMBOL: i32 = 55;

#[derive(Default)]
pub struct StockDataDownloader {
    config: Rc<Config>,
    ibkr_client: IbkrClient,
    clock: Clock,
}

impl StockDataDownloader {
    pub fn new(config: Rc<Config>) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub async fn download_stock_data(&self, account_id: &str) -> anyhow::Result<StockData> {
        let portfolio = self.download_portfolio(account_id).await?;
        println!("Found {} stocks", portfolio.len());
        let portfolio: Vec<_> = portfolio
            .into_iter()
            .filter(|position| !self.config.r#override.contains_key(&position.ticker))
            .collect();

        let timestamp = self.clock.now();
        let conids: Vec<_> = portfolio.iter().map(|position| position.conid).collect();

        if conids.is_empty() {
            let result = StockData {
                timestamp,
                ..Default::default()
            };
            return Ok(result);
        }

        let market_snapshot = self.download_market_snapshot(&conids).await?;
        let result = StockData {
            portfolio,
            market_snapshot,
            timestamp,
        };
        Ok(result)
    }

    async fn download_market_snapshot(
        &self,
        conids: &[i64],
    ) -> anyhow::Result<HashMap<ContractId, MarketSnapshot>> {
        let fields = [FIELD_ID_PE_RATIO, FIELD_ID_DIVIDEND_YIELD, FIELD_ID_SYMBOL];
        let market_snapshot_raw = self.ibkr_client.market_snapshot(conids, &fields).await?;
        let market_snapshot = market_snapshot_raw
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<MarketSnapshot>>>()
            .context("Failed to parse market snapshot")?;
        let market_snapshot_map: HashMap<_, _> = market_snapshot
            .into_iter()
            .map(|snapshot| (snapshot.conid.into(), snapshot))
            .collect();
        Ok(market_snapshot_map)
    }

    async fn download_portfolio(&self, account_id: &str) -> anyhow::Result<Vec<PortfolioPosition>> {
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
    ) -> anyhow::Result<Vec<PortfolioPosition>> {
        let mut portfolio = self.ibkr_client.portfolio(account_id, page_index).await?;

        // Filter out non-stock entries because IBKR somehow keeps showing forex in my portfolio.
        // Filter out entries with 0 position because IBKR still include stocks I recently sold.
        portfolio.retain(|entry| entry.assetClass == ASSERT_CLASS_STOCK && entry.position != 0.0);

        Ok(portfolio)
    }
}

#[derive(Deserialize, Serialize, Default, PartialEq, Debug)]
pub struct StockData {
    pub portfolio: Vec<PortfolioPosition>,
    pub market_snapshot: HashMap<ContractId, MarketSnapshot>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct MarketSnapshot {
    pub conid: i64,
    pub pe_ratio: Option<f64>,
    pub dividend_yield: Option<f64>,
}

impl TryFrom<HashMap<String, Value>> for MarketSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: HashMap<String, Value>) -> Result<Self, Self::Error> {
        let result = Self {
            conid: extract_conid(&value)?,
            pe_ratio: extract_pe_ratio(&value)?,
            dividend_yield: extract_dividend_yield(&value)?,
        };
        Ok(result)
    }
}

fn extract_pe_ratio(data: &HashMap<String, Value>) -> anyhow::Result<Option<f64>> {
    data.get(&FIELD_ID_PE_RATIO.to_string())
        .map(unwrap_string_value)
        .transpose()?
        .map(|raw| raw.parse())
        .transpose()
        .context("Failed to parse P/E")
}

fn extract_conid(data: &HashMap<String, Value>) -> anyhow::Result<i64> {
    data.get("conid")
        .ok_or_else(|| anyhow::anyhow!("No `conid`"))?
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("`conid` does not contain an `i64`"))
}

fn extract_dividend_yield(data: &HashMap<String, Value>) -> anyhow::Result<Option<f64>> {
    data.get(&FIELD_ID_DIVIDEND_YIELD.to_string())
        .map(unwrap_string_value)
        .transpose()?
        .map(|raw| raw.trim_end_matches('%').to_string())
        .map(|raw| raw.parse::<f64>().map(|notional| notional / 100.0))
        .transpose()
        .context("Failed to parse P/E")
}

fn unwrap_string_value(value: &Value) -> anyhow::Result<String> {
    if let Value::String(text) = value {
        Ok(text.clone())
    } else {
        bail!("Expects a string")
    }
}

#[derive(From, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ContractId {
    value: i64,
}

impl Serialize for ContractId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value.to_string())
    }
}

impl<'de> Deserialize<'de> for ContractId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ContractIdVisitor)
    }
}

pub struct ContractIdVisitor;

impl<'de> Visitor<'de> for ContractIdVisitor {
    type Value = ContractId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an `i64`")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(parsed) = v.parse::<i64>() {
            Ok(parsed.into())
        } else {
            Err(serde::de::Error::invalid_value(Unexpected::Str(v), &self))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn download_stock_data() {
        let portfolio = vec![PortfolioPosition {
            conid: 1,
            assetClass: ASSERT_CLASS_STOCK.into(),
            position: 1.0,
            ..Default::default()
        }];
        let market_snapshot = vec![HashMap::from([
            ("conid".into(), 1.into()),
            (FIELD_ID_PE_RATIO.to_string(), "2".into()),
            (FIELD_ID_DIVIDEND_YIELD.to_string(), "3%".into()),
        ])];
        let expected_stock_data = StockData {
            portfolio: portfolio.clone(),
            market_snapshot: HashMap::from([(
                1.into(),
                MarketSnapshot {
                    conid: 1,
                    pe_ratio: 2.0.into(),
                    dividend_yield: 0.03.into(),
                },
            )]),
            ..Default::default()
        };

        let mut clock = Clock::default();
        clock
            .expect_now()
            .return_const(<DateTime<Utc> as Default>::default());

        let mut ibkr_client = IbkrClient::default();
        ibkr_client
            .expect_portfolio()
            .return_once(move |_, _| Ok(portfolio));
        ibkr_client
            .expect_market_snapshot()
            .return_once(move |_, _| Ok(market_snapshot));

        let downloader = StockDataDownloader {
            ibkr_client,
            clock,
            ..Default::default()
        };

        // When
        let actual_stock_data = downloader.download_stock_data("").await.unwrap();

        // Then
        assert_eq!(expected_stock_data, actual_stock_data);
    }

    #[tokio::test]
    async fn download_stock_data_from_no_portfolio() {
        let mut clock = Clock::default();
        clock
            .expect_now()
            .return_const(<DateTime<Utc> as Default>::default());

        let mut ibkr_client = IbkrClient::default();
        ibkr_client.expect_portfolio().returning(|_, _| Ok(vec![]));

        let downloader = StockDataDownloader {
            ibkr_client,
            clock,
            ..Default::default()
        };

        // When
        let actual_stock_data = downloader.download_stock_data("").await.unwrap();

        // Then
        assert_eq!(StockData::default(), actual_stock_data);
    }

    #[tokio::test]
    async fn download_market_snapshot() {
        let expected_market_snapshot: HashMap<_, _> = [
            (
                1.into(),
                MarketSnapshot {
                    conid: 1,
                    pe_ratio: 1.0.into(),
                    dividend_yield: 0.03.into(),
                },
            ),
            (
                2.into(),
                MarketSnapshot {
                    conid: 2,
                    pe_ratio: 3.0.into(),
                    dividend_yield: 0.05.into(),
                },
            ),
        ]
        .into();

        let mut ibkr_client = IbkrClient::default();
        ibkr_client
            .expect_market_snapshot()
            .returning(|_, _| Ok(sample_raw_market_snapshot()));

        let downloader = StockDataDownloader {
            ibkr_client,
            ..Default::default()
        };

        // When
        let actual_market_snapshot = downloader
            .download_market_snapshot(&[1, 2, 3, 4])
            .await
            .unwrap();

        // Then
        assert_eq!(expected_market_snapshot, actual_market_snapshot)
    }

    #[tokio::test]
    async fn download_portfolio_from_multiple_pages() {
        let entries_at_last_page = 5;
        let expected_portfolio =
            build_portfolio_with_n_entries(PORTFOLIO_PAGE_SIZE * 2 + entries_at_last_page);

        let mut ibkr_client = IbkrClient::default();
        ibkr_client
            .expect_portfolio()
            .with(always(), eq(0))
            .returning(|_, _| Ok(build_portfolio_with_n_entries(PORTFOLIO_PAGE_SIZE)));
        ibkr_client
            .expect_portfolio()
            .with(always(), eq(1))
            .returning(|_, _| Ok(build_portfolio_with_n_entries(PORTFOLIO_PAGE_SIZE)));
        ibkr_client
            .expect_portfolio()
            .with(always(), eq(2))
            .returning(move |_, _| Ok(build_portfolio_with_n_entries(entries_at_last_page)));

        let downloader = StockDataDownloader {
            ibkr_client,
            ..Default::default()
        };

        // When
        let actual_portfolio = downloader.download_portfolio("").await.unwrap();

        // Then
        assert_eq!(expected_portfolio, actual_portfolio);
    }

    #[tokio::test]
    async fn download_portfolio_from_1_page() {
        let entries_at_last_page = 3;
        let expected_portfolio = build_portfolio_with_n_entries(entries_at_last_page);

        let mut ibkr_client = IbkrClient::default();
        ibkr_client
            .expect_portfolio()
            .with(always(), eq(0))
            .returning(move |_, _| Ok(build_portfolio_with_n_entries(entries_at_last_page)));

        let downloader = StockDataDownloader {
            ibkr_client,
            ..Default::default()
        };

        // When
        let actual_portfolio = downloader.download_portfolio("").await.unwrap();

        // Then
        assert_eq!(expected_portfolio, actual_portfolio);
    }

    #[tokio::test]
    async fn download_portfolio_at_page() {
        let raw_portfolio = vec![
            PortfolioPosition {
                conid: 0,
                ticker: "TICKER".into(),
                position: 10.0,
                assetClass: ASSERT_CLASS_STOCK.into(),
            },
            PortfolioPosition {
                conid: 0,
                ticker: "SOLD".into(),
                position: 0.0,
                assetClass: ASSERT_CLASS_STOCK.into(),
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
            assetClass: ASSERT_CLASS_STOCK.into(),
        }];

        let mut ibkr_client = IbkrClient::default();
        ibkr_client
            .expect_portfolio()
            .return_once(move |_, _| Ok(raw_portfolio));

        let downloader = StockDataDownloader {
            ibkr_client,
            ..Default::default()
        };

        // When
        let actual_portfolio = downloader.download_portfolio_at_page("", 0).await.unwrap();

        // Then
        assert_eq!(expected_portfolio, actual_portfolio);
    }

    #[test]
    fn market_snapshot_try_from() {
        // Given
        let raw: HashMap<_, _> = [
            ("conid".into(), 1.into()),
            (FIELD_ID_PE_RATIO.to_string(), "1".into()),
            (FIELD_ID_DIVIDEND_YIELD.to_string(), "3%".into()),
        ]
        .into();
        let expected_market_snapshot = MarketSnapshot {
            conid: 1,
            pe_ratio: 1.0.into(),
            dividend_yield: 0.03.into(),
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

    fn build_portfolio_with_n_entries(n: usize) -> Vec<PortfolioPosition> {
        (0..n)
            .map(|_| PortfolioPosition {
                position: 1.0,
                assetClass: ASSERT_CLASS_STOCK.into(),
                ..Default::default()
            })
            .collect()
    }

    fn sample_raw_market_snapshot() -> Vec<HashMap<String, Value>> {
        vec![
            HashMap::from([
                ("conid".into(), 1.into()),
                (FIELD_ID_PE_RATIO.to_string(), "1".into()),
                (FIELD_ID_DIVIDEND_YIELD.to_string(), "3%".into()),
            ]),
            HashMap::from([
                ("conid".into(), 2.into()),
                (FIELD_ID_PE_RATIO.to_string(), "3".into()),
                (FIELD_ID_DIVIDEND_YIELD.to_string(), "5%".into()),
            ]),
        ]
    }
}
