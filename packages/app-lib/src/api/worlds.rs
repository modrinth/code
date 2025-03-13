use crate::util::io;
use crate::{Error, ErrorKind, Result};
use chrono::{DateTime, Utc};
use craftping::{Chat, Player};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct World {
    pub name: String,
    pub last_played: DateTime<Utc>,
    pub icon: Option<Url>,
    pub pinned: bool,
    #[serde(flatten)]
    pub details: WorldDetails,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldDetails {
    Singleplayer {
        path: PathBuf,
        game_mode: SingleplayerGameMode,
        hardcore: bool,
    },
    Server {
        address: String,
    },
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SingleplayerGameMode {
    Creative,
    Survival,
    Adventure,
    Spectator,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerStatus {
    #[serde(flatten)]
    pub ping_response: craftping::Response,
    pub ping: Option<i64>,
}

pub async fn get_profile_worlds(path: &str) -> Result<Vec<World>> {
    let mut result = vec![];
    let path = PathBuf::from(path);
    get_singleplayer_worlds(&path, &mut result).await?;
    get_server_worlds(&path, &mut result).await?;
    Ok(result)
}

async fn get_singleplayer_worlds(
    instance_dir: &PathBuf,
    worlds: &mut Vec<World>,
) -> Result<()> {
    let mut saves_dir = io::read_dir(instance_dir.join("saves")).await?;
    while let Some(world_dir) = saves_dir.next_entry().await? {}

    Ok(())
}

async fn get_server_worlds(
    instance_dir: &PathBuf,
    worlds: &mut Vec<World>,
) -> Result<()> {
    Ok(())
}

pub async fn get_server_status(address: &str) -> Result<ServerStatus> {
    let (hostname, port) = match parse_server_address(address) {
        Ok(x) => x,
        Err(e) => return Err(Error::from(ErrorKind::InputError(e))),
    };
    let mut stream = TcpStream::connect((hostname, port)).await?;
    let ping_response =
        craftping::tokio::ping(&mut stream, &hostname, port).await?;
    let ping = ping_server(&mut stream).await.ok();
    Ok(ServerStatus {
        ping_response,
        ping,
    })
}

// Reimplementation of Guava's HostAndPort#fromString with a default port of 25565
fn parse_server_address(
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
            if address.bytes().nth(close_bracket_index) != Some(b':') {
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

async fn ping_server(stream: &mut TcpStream) -> Result<i64> {
    let start_time = Utc::now();
    let ping_magic = start_time.timestamp_millis();

    stream.write_all(&[0x09, 0x01]).await?;
    stream.write_i64(ping_magic).await?;
    stream.flush().await?;

    let mut response_prefix = [0_u8; 2];
    stream.read_exact(&mut response_prefix).await?;
    let response_magic = stream.read_i64().await?;
    if response_prefix != [0x09, 0x01] || response_magic != ping_magic {
        return Err(Error::from(ErrorKind::InputError(
            "Unexpected ping response".to_string(),
        )));
    }

    let response_time = Utc::now();
    Ok((response_time - start_time).num_milliseconds())
}
