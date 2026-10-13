#[tokio::main]
async fn main() -> anyhow::Result<()> {
    modrinth_minecraft_meta::main().await
}
