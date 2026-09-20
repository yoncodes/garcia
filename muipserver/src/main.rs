#[tokio::main]
async fn main() -> anyhow::Result<()> {
    muipserver::run().await
}
