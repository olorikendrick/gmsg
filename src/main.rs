#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = gmsg::run().await?;

    Ok(())
}
