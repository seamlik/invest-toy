use crate::config::Config;
use crate::ibkr::HistoricalMarketDataEntry;
use crate::ibkr::IbkrClient;
use crate::ibkr::PortfolioPosition;
use crate::ranker::Name;
use crate::ranker::Notional;
use crate::ranker::ScoringFactor;
use crate::ranker::StockCandidates;
use crate::ranker::StockRanker;
use crate::report::ReportRenderer;
use crate::table::TablePrinter;
use anyhow::Context;
use std::collections::HashMap;

const FIELD_ID_PE_RATIO: i32 = 7290;
const LONG_TERM_YEARS: usize = 6;

pub struct Toy {
    config: Config,
    ranker: StockRanker,
    table_printer: TablePrinter,
    report_renderer: ReportRenderer,
    ibkr_client: IbkrClient,
}

impl Toy {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ranker: Default::default(),
            table_printer: TablePrinter,
            report_renderer: ReportRenderer,
            ibkr_client: IbkrClient,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        // Some API requires querying this endpoint first
        let iserver_accounts = self.ibkr_client.i_server_accounts().await?;
        if iserver_accounts.accounts.is_empty() {
            anyhow::bail!("No brokerage account found");
        }

        let portfolio_accounts = self.ibkr_client.portfolio_accounts().await?;
        let account_id = portfolio_accounts
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No default account found"))?
            .accountId;
        println!("Account ID: {}", &account_id);

        let portfolio = self.fetch_portfolio(&account_id).await?;
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
        for (ticker, factors) in self.fetch_pe_ratio(&portfolio).await?.into_iter() {
            merge_stock_candidates(&mut candidates, &ticker, factors)
        }

        for position in portfolio.iter() {
            let factors = self.fetch_price_changes(position).await?;
            merge_stock_candidates(&mut candidates, &position.ticker.as_str().into(), factors);
        }

        let scores = self.ranker.rank(&candidates);
        let report = self.report_renderer.render(&candidates, &scores);
        self.table_printer.print(&report).await?;

        Ok(())
    }

    async fn fetch_pe_ratio(
        &self,
        portfolio: &[PortfolioPosition],
    ) -> anyhow::Result<StockCandidates> {
        let conids: Vec<_> = portfolio.iter().map(|position| position.conid).collect();
        let fields = [FIELD_ID_PE_RATIO];
        let market_snapshot = self.ibkr_client.market_snapshot(&conids, &fields).await?;
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

    async fn fetch_portfolio(&self, account_id: &str) -> reqwest::Result<Vec<PortfolioPosition>> {
        // Fetch the first page always
        let mut current_page_index = 0;
        let mut positions = self.fetch_portfolio_at_page(account_id, 0).await?;

        let mut current_page_size = positions.len();
        while current_page_size >= 30 {
            current_page_index += 1;
            let next_page = self
                .fetch_portfolio_at_page(account_id, current_page_index)
                .await?;
            current_page_size = next_page.len();
            positions.extend(next_page.into_iter())
        }

        Ok(positions)
    }

    async fn fetch_portfolio_at_page(
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

    async fn fetch_price_changes(
        &self,
        position: &PortfolioPosition,
    ) -> reqwest::Result<HashMap<ScoringFactor, Notional>> {
        let market_history_since_last_month = self
            .ibkr_client
            .short_term_market_history(position.conid)
            .await?;
        let market_history_since_long_term = self
            .ibkr_client
            .long_term_market_history(position.conid, LONG_TERM_YEARS)
            .await?;
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
}

fn extract_pe_ratio(data: HashMap<String, String>) -> anyhow::Result<Option<f64>> {
    data.get(&FIELD_ID_PE_RATIO.to_string())
        .map(|raw| raw.parse())
        .transpose()
        .context("Failed to parse P/E")
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
