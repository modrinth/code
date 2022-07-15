use eyre::Result;
use futures::TryFutureExt;
use paris::*;
use tracing_error::ErrorLayer;
use tracing_futures::WithSubscriber;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

#[macro_use]
mod util;

mod subcommands;

#[derive(argh::FromArgs, Debug)]
/// The official Modrinth CLI
pub struct Args {
    #[argh(subcommand)]
    pub subcommand: subcommands::Subcommand,
}

#[tracing::instrument]
fn main() -> Result<()> {
    let args = argh::from_env::<Args>();

    color_eyre::install()?;
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))?;

    let format = fmt::layer()
        .without_time()
        .with_writer(std::io::stderr)
        .with_target(false)
        .compact();

    tracing_subscriber::registry()
        .with(format)
        .with(filter)
        .with(ErrorLayer::default())
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(
            async move {
                args.dispatch()
                    .inspect_err(|_| error!("An error has occurred!\n"))
                    .await
            }
            .with_current_subscriber(),
        )
}
