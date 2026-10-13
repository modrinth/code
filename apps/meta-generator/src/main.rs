#[tokio::main]
async fn main() -> anyhow::Result<()> {
    meta_generator::main().await
}
