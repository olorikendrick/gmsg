#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gmsg::run().await?;

    Ok(())
}
