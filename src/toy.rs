use crate::config::Config;
use crate::ranker::Name;
use crate::ranker::Notional;
use crate::ranker::ScoringFactor;
use crate::ranker::StockCandidates;
use crate::ranker::StockRanker;
use crate::report::ReportRenderer;
use crate::table::TablePrinter;
use anyhow::Context;
use itertools::Itertools;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;

const FIELD_ID_PE_RATIO: u32 = 7290;
const LONG_TERM_YEARS: usize = 6;

pub struct Toy {
    config: Config,
    ranker: StockRanker,
    table_printer: TablePrinter,
}

impl Toy {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ranker: Default::default(),
            table_printer: TablePrinter,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        // Some API requires querying this endpoint first
        let iserver_accounts: IserverAccount = fetch_ibkr("iserver/accounts").await?;
        if iserver_accounts.accounts.is_empty() {
            anyhow::bail!("No brokerage account found");
        }

        let portfolio_accounts: Vec<PortfolioAccount> = fetch_ibkr("portfolio/accounts").await?;
        let account_id = portfolio_accounts
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No default account found"))?
            .accountId;
        println!("Account ID: {}", &account_id);

        let portfolio = fetch_portfolio(&account_id).await?;
        println!("Found {} stocks", portfolio.len());
        let portfolio: Vec<_> = portfolio
            .into_iter()
            .filter(|position| !self.config.r#override.contains_key(&position.ticker))
            .collect();

        let mut candidates: StockCandidates = self
            .config
            .r#override
            .iter()
            .map(|(ticker, factors)| {
                (
                    ticker.as_str().into(),
                    convert_config_factors_for_ranker(factors),
                )
            })
            .collect();
        for (ticker, factors) in fetch_pe_ratio(&portfolio).await?.into_iter() {
            merge_stock_candidates(&mut candidates, &ticker, factors)
        }

        for position in portfolio.iter() {
            let factors = fetch_price_changes(position).await?;
            merge_stock_candidates(&mut candidates, &position.ticker.as_str().into(), factors);
        }

        let scores = self.ranker.rank(&candidates);
        let report = ReportRenderer::render(&candidates, &scores);
        self.table_printer.print(&report).await?;

        Ok(())
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct PortfolioAccount {
    accountId: String,
}

#[derive(Deserialize)]
struct IserverAccount {
    accounts: Vec<String>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct PortfolioPosition {
    conid: i32,
    ticker: String,
    position: f64,
    assetClass: String,
}

#[derive(Deserialize)]
struct HistoricalMarketData {
    data: Vec<HistoricalMarketDataEntry>,
}

#[derive(Deserialize)]
struct HistoricalMarketDataEntry {
    ///Price at market close
    c: f64,

    ///Timestamp
    t: i64,
}

async fn fetch_portfolio(account_id: &str) -> anyhow::Result<Vec<PortfolioPosition>> {
    // Fetch the first page always
    let mut current_page_index = 0;
    let mut positions = fetch_portfolio_at_page(0, account_id).await?;

    let mut current_page_size = positions.len();
    while current_page_size >= 30 {
        current_page_index += 1;
        let next_page = fetch_portfolio_at_page(current_page_index, account_id).await?;
        current_page_size = next_page.len();
        positions.extend(next_page.into_iter())
    }

    Ok(positions)
}

async fn fetch_portfolio_at_page(
    page_index: usize,
    account_id: &str,
) -> anyhow::Result<Vec<PortfolioPosition>> {
    let endpoint = format!("portfolio/{}/positions/{}", account_id, page_index);
    let mut portfolio: Vec<PortfolioPosition> = fetch_ibkr(&endpoint).await?;

    // Filter out non-stock entries because IBKR somehow keeps showing forex in my portfolio
    // Filter out entries with 0 position because IBKR still include stocks I recently sold
    portfolio.retain(|entry| entry.assetClass == "STK" && entry.position != 0.0);

    Ok(portfolio)
}

#[derive(Clone, Copy)]
enum MarketHistoryPeriod {
    LongTerm,
    ShortTerm,
}

async fn fetch_historical_market_data(
    conid: i32,
    period: MarketHistoryPeriod,
) -> anyhow::Result<Vec<HistoricalMarketDataEntry>> {
    let (chart_period, chart_bar) = match period {
        MarketHistoryPeriod::LongTerm => (format!("{}y", LONG_TERM_YEARS), "1m"),
        MarketHistoryPeriod::ShortTerm => ("2m".to_string(), "1d"),
    };
    let endpoint = format!(
        "iserver/marketdata/history?conid={}&period={}&bar={}&outsideRth=false",
        conid, chart_period, chart_bar
    );
    let mut result: HistoricalMarketData = fetch_ibkr(&endpoint).await?;
    result.data.sort_unstable_by(|x, y| y.t.cmp(&x.t));
    Ok(result.data)
}

async fn fetch_price_changes(
    position: &PortfolioPosition,
) -> anyhow::Result<HashMap<ScoringFactor, Notional>> {
    let market_history_since_last_month =
        fetch_historical_market_data(position.conid, MarketHistoryPeriod::ShortTerm).await?;
    let market_history_since_long_term =
        fetch_historical_market_data(position.conid, MarketHistoryPeriod::LongTerm).await?;

    let latest_entry = if let Some(v) = market_history_since_last_month.first() {
        v
    } else {
        println!(
            "{} has no market history, cannot calculate price changes.",
            position.ticker
        );
        return Ok(Default::default());
    };

    let mut factors = HashMap::default();

    if let Some(earliest_entry) = market_history_since_long_term.last() {
        if earliest_entry.c == 0.0 {
            println!(
                "{} has 0 in its earliest price, cannot calculate the long-term price change",
                position.ticker
            );
        } else if market_history_since_long_term.len() < LONG_TERM_YEARS * 12 - 4 {
            println!("{} has not enough long-term history ({} months), cannot calculate the long-term price change", position.ticker, market_history_since_long_term.len())
        } else {
            let change = (latest_entry.c - earliest_entry.c) / earliest_entry.c;
            factors.insert(ScoringFactor::LongTermChange, change.into());
        }
    }
    if let Some(last_month_entry) = last_month_entry_of(&market_history_since_last_month) {
        if last_month_entry.c == 0.0 {
            println!(
                "{} has 0 in its last-month price, cannot calculate the short-term price change",
                position.ticker
            );
        } else if market_history_since_last_month.len() < 30 {
            println!(
                "{} has not enough short-term history ({} days), cannot calculate the short-term price change",
                position.ticker,
                market_history_since_last_month.len()
            )
        } else {
            let change = (latest_entry.c - last_month_entry.c) / last_month_entry.c;
            factors.insert(ScoringFactor::ShortTermChange, change.into());
        }
    }

    Ok(factors)
}

async fn fetch_pe_ratio(portfolio: &[PortfolioPosition]) -> anyhow::Result<StockCandidates> {
    let conids_text = portfolio.iter().map(|position| position.conid).join(",");
    let endpoint = format!(
        "iserver/marketdata/snapshot?conids={}&fields={}",
        conids_text, FIELD_ID_PE_RATIO
    );
    let market_snapshot: Vec<HashMap<String, String>> = fetch_ibkr(&endpoint).await?;
    let pe_ratio_list = market_snapshot
        .into_iter()
        .map(extract_pe_ratio)
        .collect::<anyhow::Result<Vec<_>>>()?;
    if pe_ratio_list.len() != portfolio.len() {
        anyhow::bail!("Number of P/E entries does not match the portfolio")
    }
    let pe_ratio_map_by_ticker: HashMap<_, _> = portfolio
        .iter()
        .enumerate()
        .filter_map(|(index, position)| {
            pe_ratio_list
                .get(index)
                .cloned()
                .unwrap_or_default()
                .map(|pe_ratio| {
                    (
                        position.ticker.as_str().into(),
                        HashMap::from([(ScoringFactor::PeRatio, pe_ratio.into())]),
                    )
                })
        })
        .collect();
    Ok(pe_ratio_map_by_ticker)
}

fn extract_pe_ratio(data: HashMap<String, String>) -> anyhow::Result<Option<f64>> {
    data.get(&FIELD_ID_PE_RATIO.to_string())
        .map(|raw| raw.parse())
        .transpose()
        .context("Failed to parse P/E")
}

async fn fetch_ibkr<T>(endpoint: &str) -> anyhow::Result<T>
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
        .await
        .with_context(|| format!("Failed to fetch {}", endpoint_full))?;
    Ok(response)
}

fn last_month_entry_of(
    history: &[HistoricalMarketDataEntry],
) -> Option<&HistoricalMarketDataEntry> {
    let latest = history.first()?;
    let milliseconds_of_1_month = 1000 * 60 * 60 * 24 * 30;
    history
        .iter()
        .find(|entry| latest.t - entry.t >= milliseconds_of_1_month)
}

fn merge_stock_candidates(
    candidates: &mut StockCandidates,
    ticker: &Name,
    factors: HashMap<ScoringFactor, Notional>,
) {
    if factors.is_empty() {
        return;
    }
    if let Some(existing_factors) = candidates.get_mut(ticker) {
        existing_factors.extend(factors.into_iter());
    } else {
        candidates.insert(ticker.clone(), factors);
    }
}

fn convert_config_factors_for_ranker(
    factors: &HashMap<ScoringFactor, f64>,
) -> HashMap<ScoringFactor, Notional> {
    factors
        .iter()
        .map(|(factor, notional)| (*factor, (*notional).into()))
        .collect()
}
