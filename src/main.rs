// main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gmsg::Gmsg::run().await

}
