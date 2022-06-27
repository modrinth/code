use eyre::Result;
use futures::TryFutureExt;
use paris::*;

mod subcommands;
mod util;

#[derive(argh::FromArgs)]
/// The official Modrinth CLI
pub struct Args {
    #[argh(subcommand)]
    pub subcommand: subcommands::SubCommand,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = argh::from_env::<Args>();
    pretty_env_logger::formatted_builder()
        .filter_module("theseus", log::LevelFilter::Info)
        .target(pretty_env_logger::env_logger::Target::Stderr)
        .init();

    args.dispatch()
        .inspect_err(|_| error!("An error has occurred!\n"))
        .await
}
