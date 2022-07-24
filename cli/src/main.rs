mod toy;

use toy::Toy;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Toy::default().run().await
}
