use crate::config::Config;
use crate::stock_data_downloader::StockData;
use crate::stock_data_downloader::StockDataDownloader;
use chrono::DateTime;
use chrono::Utc;
use std::path::PathBuf;
use std::rc::Rc;

pub struct StockDataCacher {
    downloader: StockDataDownloader,
    cache_path: PathBuf,
    config: Rc<Config>,
}

impl StockDataCacher {
    pub fn new(config: Rc<Config>) -> Self {
        let mut cache_path = std::env::temp_dir();
        cache_path.push("ibkr-toy-cache.bson");
        Self {
            downloader: Default::default(),
            cache_path,
            config,
        }
    }

    pub async fn fetch(&self, account_id: &str, force_download: bool) -> anyhow::Result<StockData> {
        if force_download {
            println!("Force download stock data")
        } else if let Ok(stock_data) = self.read_cache().await {
            if !cache_outdated(stock_data.timestamp) {
                return Ok(stock_data);
            } else {
                println!("Cache is outdated");
            }
        } else {
            println!("Stock data not found in cache");
        }

        println!("Downloading stock data from IBKR");
        let stock_data = self
            .downloader
            .download_stock_data(account_id, &self.config)
            .await?;

        let stock_data_bson = bson::to_vec(&stock_data)?;
        if let Err(e) = tokio::fs::write(&self.cache_path, stock_data_bson).await {
            println!("Failed to write cache: {}", e)
        }

        Ok(stock_data)
    }

    async fn read_cache(&self) -> anyhow::Result<StockData> {
        let cache_bytes = tokio::fs::read(&self.cache_path).await?;
        let stock_data = bson::from_slice(&cache_bytes)?;
        Ok(stock_data)
    }
}

fn cache_outdated(timstamp: DateTime<Utc>) -> bool {
    (Utc::now() - timstamp).num_days() >= 1
}
