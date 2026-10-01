use std::path::Path;

use anyhow::Result;
use toasty_cli::ToastyCli;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_anyhow::ErrorLayer::default())
        .init();

    let config = toasty_cli::Config::load_from(Path::new(
        "apps/modrinth-minecraft-meta/Toasty.toml",
    ))?;
    let db = modrinth_minecraft_meta::connect_to_db().await?;
    let cli = ToastyCli::with_config(db, config);
    cli.parse_and_run().await?;
    Ok(())
}
