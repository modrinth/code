//! Example/testing binary for checking if a MaxMind database can be loaded from
//! the current environment.

use std::net::IpAddr;

use eyre::Result;
use maxminddb::geoip2;
use modrinth_util::Context;
use tracing::info;

/// Looks up country details for an IP using the MaxMind database
#[derive(Debug, clap::Parser)]
struct Args {
    /// IP address to look up
    ip: IpAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = <Args as clap::Parser>::parse();
    tracing_subscriber::fmt().init();

    let maxmind = modrinth_maxmind::init_reader()
        .await
        .wrap_err("failed to create reader")?;

    let ip = args.ip;
    let country = maxmind
        .lookup::<geoip2::Country>(ip)
        .wrap_err("failed to lookup country")?;

    info!("Country details for {ip:?}:\n{country:#?}");

    Ok(())
}
