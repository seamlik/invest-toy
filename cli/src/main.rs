mod cli;
mod serde;
mod toy;

use log::LevelFilter;
use toy::Toy;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();
    Toy::default().run().await
}
