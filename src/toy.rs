use crate::config::Config;
use crate::ibkr_client::IbkrClient;
use crate::report_renderer::ReportRenderer;
use crate::scoring_factor_extractor::ScoringFactorExtractor;
use crate::stock_data_cacher::StockDataCacher;
use crate::stock_ranker::StockRanker;
use crate::table_printer::TablePrinter;
use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;
use std::rc::Rc;

pub struct Toy {
    args: Cli,
    ranker: StockRanker,
    table_printer: TablePrinter,
    report_renderer: ReportRenderer,
    ibkr_client: IbkrClient,
    stock_data_cacher: StockDataCacher,
    scoring_factor_extractor: ScoringFactorExtractor,
}

impl Toy {
    pub fn new(args: Cli, config: Rc<Config>) -> Self {
        Self {
            args,
            ranker: Default::default(),
            table_printer: TablePrinter,
            report_renderer: ReportRenderer,
            ibkr_client: Default::default(),
            stock_data_cacher: StockDataCacher::new(config.clone()),
            scoring_factor_extractor: ScoringFactorExtractor::new(config),
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

        let stock_data = self
            .stock_data_cacher
            .fetch(&account_id, self.args.force_download)
            .await
            .context("Failed to fetch stock data")?;
        let candidates = self
            .scoring_factor_extractor
            .extract_scoring_factors(&stock_data);
        let scores = self.ranker.rank(&candidates);
        let report = self.report_renderer.render(&candidates, &scores);
        self.table_printer.print(&report).await?;

        Ok(())
    }
}

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub force_download: bool,
}
