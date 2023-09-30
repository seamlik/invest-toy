use crate::arithmetic_renderer::ArithmeticRenderer;
use crate::ibkr_client::IbkrClient;
use crate::invest_advisor::InvestAdvisor;
use crate::report_renderer::ReportRenderer;
use crate::scoring_factor_extractor::ScoringFactorExtractor;
use crate::stock_data_cacher::StockDataCacher;
use crate::stock_ranker::StockRanker;
use crate::table_printer::TablePrinter;
use anyhow::Context;
use clap::Parser;

pub struct Toy {
    args: Cli,
    ranker: StockRanker,
    table_printer: TablePrinter,
    report_renderer: ReportRenderer,
    ibkr_client: IbkrClient,
    stock_data_cacher: StockDataCacher,
    scoring_factor_extractor: ScoringFactorExtractor,
    invest_advisor: InvestAdvisor,
}

impl Toy {
    pub fn new(args: Cli) -> Self {
        Self {
            args,
            ranker: Default::default(),
            table_printer: TablePrinter,
            report_renderer: ReportRenderer {
                arithmetic_renderer: ArithmeticRenderer,
            },
            ibkr_client: Default::default(),
            stock_data_cacher: StockDataCacher::default(),
            scoring_factor_extractor: ScoringFactorExtractor,
            invest_advisor: InvestAdvisor {
                arithmetic_renderer: ArithmeticRenderer,
            },
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
            .fetch(&account_id, self.args.use_cache)
            .await
            .context("Failed to fetch stock data")?;
        let candidates = self
            .scoring_factor_extractor
            .extract_scoring_factors(&stock_data);
        let scores = self.ranker.rank(&candidates);
        let report = self.report_renderer.render(&candidates, &scores);

        println!();
        println!("=============");
        println!("Score details");
        println!("=============");
        self.table_printer.print(&report).await?;

        let invest_advices = self
            .invest_advisor
            .render_advice(&scores, self.args.invest_num);

        println!();
        println!("==================");
        println!("Investment advices");
        println!("==================");
        self.table_printer.print(&invest_advices).await?;

        Ok(())
    }
}

#[derive(Parser)]
pub struct Cli {
    /// Generates report using cached data
    #[arg(long)]
    pub use_cache: bool,

    /// Number of stocks to invest.
    #[arg(long, default_value = "16")]
    pub invest_num: usize,
}
