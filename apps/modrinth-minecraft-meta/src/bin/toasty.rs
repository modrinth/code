use anyhow::{Context, Result};
use toasty_cli::ToastyCli;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_anyhow::ErrorLayer::default())
        .init();

    let toasty_config =
        toasty_cli::Config::load().context("loading toasty config")?;
    let app_config = modrinth_minecraft_meta::create_config()
        .context("loading app config")?;
    let db = modrinth_minecraft_meta::connect_to_db(&app_config).await?;
    let cli = ToastyCli::with_config(db, toasty_config);
    cli.parse_and_run().await?;
    Ok(())
}
