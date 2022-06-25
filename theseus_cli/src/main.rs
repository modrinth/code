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

    args.dispatch()
        .inspect_err(|_| error!("An error has occurred!\n"))
        .await
}
