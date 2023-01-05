use chrono::Utc;

use crate::config::Config;
use crate::ibkr_client::HistoricalMarketDataEntry;
use crate::ibkr_client::IbkrClient;
use crate::report_renderer::ReportRenderer;
use crate::stock_candidates::StockCandidates;
use crate::stock_data_downloader::ContractId;
use crate::stock_data_downloader::MarketSnapshot;
use crate::stock_data_downloader::StockDataDownloader;
use crate::stock_ranker::Name;
use crate::stock_ranker::ScoringFactor;
use crate::stock_ranker::StockRanker;
use crate::table_printer::TablePrinter;
use std::collections::HashMap;

pub struct Toy {
    config: Config,
    ranker: StockRanker,
    table_printer: TablePrinter,
    report_renderer: ReportRenderer,
    ibkr_client: IbkrClient,
    stock_data_downloader: StockDataDownloader,
}

impl Toy {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ranker: Default::default(),
            table_printer: TablePrinter,
            report_renderer: ReportRenderer,
            ibkr_client: Default::default(),
            stock_data_downloader: Default::default(),
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

        let portfolio = self
            .stock_data_downloader
            .download_portfolio(&account_id)
            .await?;
        println!("Found {} stocks", portfolio.len());
        let portfolio: Vec<_> = portfolio
            .into_iter()
            .filter(|position| !self.config.r#override.contains_key(&position.ticker))
            .collect();

        let mut candidates = StockCandidates::from_config_overrides(&self.config.r#override);

        // Download stock data
        let conids: Vec<_> = portfolio.iter().map(|position| position.conid).collect();
        let market_snapshot = self
            .stock_data_downloader
            .download_market_snapshot(&conids)
            .await?;
        let short_term_market_history = self
            .stock_data_downloader
            .download_short_term_market_history(&conids)
            .await?;
        let long_term_market_history = self
            .stock_data_downloader
            .download_long_term_market_history(&conids)
            .await?;

        for position in portfolio {
            let conid = position.conid.into();
            let ticker: Name = position.ticker.into();

            // Extract P/E
            if let Some(notional) = market_snapshot
                .get(&conid)
                .and_then(|snapshot| snapshot.pe_ratio)
            {
                candidates.add_candidate(ticker.clone(), ScoringFactor::PeRatio, notional.into());
            }

            // Long-term price change
            if let Some(notional) = extract_long_term_price_change(
                &position.conid.into(),
                &market_snapshot,
                &long_term_market_history,
            ) {
                candidates.add_candidate(
                    ticker.clone(),
                    ScoringFactor::LongTermChange,
                    notional.into(),
                )
            }

            // Short-term price change
            if let Some(notional) = extract_short_term_price_change(
                &position.conid.into(),
                &market_snapshot,
                &short_term_market_history,
            ) {
                candidates.add_candidate(
                    ticker.clone(),
                    ScoringFactor::ShortTermChange,
                    notional.into(),
                )
            }
        }

        let scores = self.ranker.rank(&candidates);
        let report = self.report_renderer.render(&candidates, &scores);
        self.table_printer.print(&report).await?;

        Ok(())
    }
}

fn extract_long_term_price_change(
    conid: &ContractId,
    market_snapshot: &HashMap<ContractId, MarketSnapshot>,
    market_history: &HashMap<ContractId, Vec<HistoricalMarketDataEntry>>,
) -> Option<f64> {
    let last_price = market_snapshot.get(conid)?.last_price?;
    let oldest_market_data = market_history.get(conid)?.first()?;
    let milliseconds_of_5_years = 1000 * 60 * 60 * 24 * 365 * 5;
    let now = Utc::now().timestamp_millis();
    if oldest_market_data.c == 0.0 || now - oldest_market_data.t < milliseconds_of_5_years {
        None
    } else {
        Some((last_price - oldest_market_data.c) / oldest_market_data.c)
    }
}

fn extract_short_term_price_change(
    conid: &ContractId,
    market_snapshot: &HashMap<ContractId, MarketSnapshot>,
    market_history: &HashMap<ContractId, Vec<HistoricalMarketDataEntry>>,
) -> Option<f64> {
    let last_price = market_snapshot.get(conid)?.last_price?;
    let history = market_history.get(conid)?;
    let price_on_last_month = last_month_entry_of(history)?.c;
    if price_on_last_month == 0.0 {
        None
    } else {
        Some((last_price - price_on_last_month) / price_on_last_month)
    }
}

fn last_month_entry_of(
    history: &[HistoricalMarketDataEntry],
) -> Option<&HistoricalMarketDataEntry> {
    if history.len() < 30 {
        return None;
    }
    let milliseconds_of_1_month = 1000 * 60 * 60 * 24 * 30;
    let now = Utc::now().timestamp_millis();
    history
        .iter()
        .rev()
        .find(|entry| now - entry.t >= milliseconds_of_1_month)
}
