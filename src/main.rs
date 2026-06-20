// main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
rustls_platform_verifier::init();
    gmsg::Gmsg::run().await
}
