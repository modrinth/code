use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    path::PathBuf,
};

use color_eyre::eyre::{eyre, Result, WrapErr};

/// Runtime configuration loaded from the active Core environment profile.
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory where all instance data is stored.
    pub data_dir: PathBuf,
    /// Backend that owns accounts; Core asks it who an account token belongs to.
    pub account_api_url: String,
    /// Address clients use to reach this Core, reported to the backend.
    /// `AMBERITE_PUBLIC_URL` when set, otherwise this machine's network address and port.
    pub public_url: String,
    /// HTTP port for the Core API.
    pub port: u16,
    /// Host/IP the Core API binds to.
    pub bind_host: String,
    /// Allowed CORS origin.
    pub allowed_origin: String,
    /// Debug build: relaxes CORS for local development.
    pub dev_mode: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        load_environment_profile()?;
        let port: u16 = required_env("PORT")?
            .parse()
            .wrap_err("PORT must be a valid port number")?;
        let bind_host = required_env("AMBERITE_BIND_HOST")?;
        let public_url = match std::env::var("AMBERITE_PUBLIC_URL") {
            Ok(url) if !url.trim().is_empty() => {
                url.trim().trim_end_matches('/').to_string()
            }
            _ => default_address(&bind_host, port)?,
        };
        Ok(Self {
            data_dir: PathBuf::from(required_env("CORE_DATA_DIR")?),
            account_api_url: required_env("ACCOUNT_API_URL")?
                .trim_end_matches('/')
                .to_string(),
            public_url,
            port,
            bind_host,
            allowed_origin: required_env("ALLOWED_ORIGIN")?,
            dev_mode: cfg!(debug_assertions),
        })
    }
}

/// Where this Core can be reached when no public address is configured: the bound address, or
/// this machine's local network address when Core listens on every interface.
fn default_address(bind_host: &str, port: u16) -> Result<String> {
    let bound: IpAddr = bind_host
        .parse()
        .wrap_err("AMBERITE_BIND_HOST must be an IP address")?;
    let host = if bound.is_unspecified() {
        local_network_ip().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
    } else {
        bound
    };
    Ok(format!("http://{}", SocketAddr::new(host, port)))
}

/// The address this machine uses on its local network. Connecting a UDP socket picks the
/// outgoing interface without sending anything.
fn local_network_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("10.255.255.255:1").ok()?;
    Some(socket.local_addr().ok()?.ip())
}

fn required_env(name: &str) -> Result<String> {
    let value = std::env::var(name)
        .map_err(|_| eyre!("Missing required environment variable: {name}"))?;
    if value.trim().is_empty() {
        return Err(eyre!("Required environment variable is empty: {name}"));
    }
    Ok(value)
}

fn load_environment_profile() -> Result<()> {
    let filename = if cfg!(debug_assertions) {
        ".env.local"
    } else {
        ".env.prod"
    };
    let path = std::env::current_dir()
        .map_err(|error| {
            eyre!("Unable to resolve the Core working directory: {error}")
        })?
        .join(filename);
    if path.is_file() {
        dotenvy::from_path(path).map_err(|error| {
            eyre!("Unable to load the Core environment profile: {error}")
        })?;
    }
    Ok(())
}
