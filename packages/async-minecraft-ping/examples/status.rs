use anyhow::Result;
use structopt::StructOpt;

use async_minecraft_ping::ConnectionConfig;

#[derive(Debug, StructOpt)]
#[structopt(name = "example")]
struct Args {
    /// Server to connect to
    #[structopt()]
    address: String,

    /// Port to connect to
    #[structopt(short = "p", long = "port")]
    port: Option<u16>,

    /// Enable SRV record lookup (requires 'srv' feature)
    #[cfg(feature = "srv")]
    #[structopt(long = "srv")]
    srv_lookup: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    let mut config = ConnectionConfig::build(args.address);
    if let Some(port) = args.port {
        config = config.with_port(port);
    }
    #[cfg(feature = "srv")]
    if args.srv_lookup {
        config = config.with_srv_lookup();
    }

    let connection = config.connect().await?;

    let connection = connection.status().await?;

    println!(
        "{} of {} player(s) online",
        connection.status.players.online, connection.status.players.max
    );

    connection.ping(42).await?;

    Ok(())
}
