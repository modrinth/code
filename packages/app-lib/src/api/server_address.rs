use crate::{Error, ErrorKind, Result};
use std::fmt::Display;
use std::mem;
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
pub enum ServerAddress {
    Unresolved(String),
    Resolved {
        original_host: String,
        original_port: u16,
        resolved_host: String,
        resolved_port: u16,
    },
}

impl ServerAddress {
    pub async fn resolve(&mut self) -> Result<()> {
        match self {
            Self::Unresolved(address) => {
                let (host, port) = parse_server_address(address)?;
                let (resolved_host, resolved_port) =
                    resolve_server_address(host, port).await?;
                *self = Self::Resolved {
                    original_host: if host.len() == address.len() {
                        mem::take(address)
                    } else {
                        host.to_owned()
                    },
                    original_port: port,
                    resolved_host,
                    resolved_port,
                }
            }
            Self::Resolved { .. } => {}
        }
        Ok(())
    }

    pub fn require_resolved(&self) -> Result<(&str, u16)> {
        match self {
            Self::Resolved {
                resolved_host,
                resolved_port,
                ..
            } => Ok((resolved_host, *resolved_port)),
            Self::Unresolved(address) => Err(ErrorKind::InputError(format!(
                "Unexpected unresolved server address: {address}"
            ))
            .into()),
        }
    }
}

impl Display for ServerAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unresolved(address) => write!(f, "{address}"),
            Self::Resolved {
                resolved_host,
                resolved_port,
                ..
            } => {
                if resolved_host.contains(':') {
                    write!(f, "[{resolved_host}]:{resolved_port}")
                } else {
                    write!(f, "{resolved_host}:{resolved_port}")
                }
            }
        }
    }
}

pub fn parse_server_address(address: &str) -> Result<(&str, u16)> {
    parse_server_address_inner(address)
        .map_err(|e| Error::from(ErrorKind::InputError(e)))
}

// Reimplementation of Guava's HostAndPort#fromString with a default port of 25565
fn parse_server_address_inner(
    address: &str,
) -> std::result::Result<(&str, u16), String> {
    let (host, port_str) = if address.starts_with("[") {
        let colon_index = address.find(':');
        let close_bracket_index = address.rfind(']');
        if colon_index.is_none() || close_bracket_index.is_none() {
            return Err(format!("Invalid bracketed host/port: {address}"));
        }
        let close_bracket_index = close_bracket_index.unwrap();

        let host = &address[1..close_bracket_index];
        if close_bracket_index + 1 == address.len() {
            (host, "")
        } else {
            if address.as_bytes().get(close_bracket_index).copied()
                != Some(b':')
            {
                return Err(format!(
                    "Only a colon may follow a close bracket: {address}"
                ));
            }
            let port_str = &address[close_bracket_index + 2..];
            for c in port_str.chars() {
                if !c.is_ascii_digit() {
                    return Err(format!("Port must be numeric: {address}"));
                }
            }
            (host, port_str)
        }
    } else {
        let colon_pos = address.find(':');
        if let Some(colon_pos) = colon_pos {
            (&address[..colon_pos], &address[colon_pos + 1..])
        } else {
            (address, "")
        }
    };

    let mut port = None;
    if !port_str.is_empty() {
        if port_str.starts_with('+') {
            return Err(format!("Unparseable port number: {port_str}"));
        }
        port = port_str.parse::<u16>().ok();
        if port.is_none() {
            return Err(format!("Unparseable port number: {port_str}"));
        }
    }

    Ok((host, port.unwrap_or(25565)))
}

pub async fn resolve_server_address(
    host: &str,
    port: u16,
) -> Result<(String, u16)> {
    static SIMULTANEOUS_DNS_QUERIES: Semaphore = Semaphore::const_new(24);

    if port != 25565
        || host.parse::<Ipv4Addr>().is_ok()
        || host.parse::<Ipv6Addr>().is_ok()
    {
        return Ok((host.to_owned(), port));
    }

    let _permit = SIMULTANEOUS_DNS_QUERIES.acquire().await?;
    let resolver = hickory_resolver::TokioResolver::builder_tokio()?.build();
    Ok(
        match resolver.srv_lookup(format!("_minecraft._tcp.{host}")).await {
            Err(e)
                if e.proto()
                    .filter(|x| x.kind().is_no_records_found())
                    .is_some() =>
            {
                None
            }
            Err(e) => return Err(e.into()),
            Ok(lookup) => lookup
                .into_iter()
                .next()
                .map(|r| (r.target().to_string(), r.port())),
        }
        .unwrap_or_else(|| (host.to_owned(), port)),
    )
}
