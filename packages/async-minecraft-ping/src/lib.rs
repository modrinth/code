mod protocol;
mod server;
use std::num::ParseIntError;

pub use server::{
    connect, ConnectionConfig, ServerError, ServerPlayer, ServerPlayers, ServerVersion,
    StatusConnection, StatusResponse,
};

pub const DEFAULT_PORT: u16 = 25565;

#[derive(Debug, thiserror::Error)]
pub enum ParseAddressError {
    #[error("failed to parse port")]
    ParsePort(#[source] ParseIntError),
}

pub fn parse_host_and_port(addr: &str) -> Result<(&str, u16), ParseAddressError> {
    match addr.rsplit_once(':') {
        Some((addr, port)) => {
            let port = port.parse::<u16>().map_err(ParseAddressError::ParsePort)?;
            Ok((addr, port))
        }
        None => Ok((addr, DEFAULT_PORT)),
    }
}
