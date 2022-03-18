use eyre::Result;
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
    theseus::init().await?;

    let res = args.dispatch().await;
    if res.is_err() {
        error!("An error has occurred!\n");
    } else {
        theseus::save().await?;
    }

    res
}
