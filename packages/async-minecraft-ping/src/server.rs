//! This module defines a wrapper around Minecraft's
//! [ServerListPing](https://wiki.vg/Server_List_Ping)

use std::time::Duration;

use serde::Deserialize;
use thiserror::Error;
use tokio::net::TcpStream;

use crate::protocol::{self, AsyncReadRawPacket, AsyncWriteRawPacket};

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("error reading or writing data")]
    ProtocolError,

    #[error("failed to connect to server")]
    FailedToConnect,

    #[error("connection timed out")]
    ConnectionTimedOut,

    #[error("invalid JSON response: \"{0}\"")]
    InvalidJson(String),

    #[error("mismatched pong payload (expected \"{expected}\", got \"{actual}\")")]
    MismatchedPayload { expected: u64, actual: u64 },
}

impl From<protocol::ProtocolError> for ServerError {
    fn from(_err: protocol::ProtocolError) -> Self {
        ServerError::ProtocolError
    }
}

/// Contains information about the server version.
#[derive(Debug, Deserialize)]
pub struct ServerVersion {
    /// The server's Minecraft version, i.e. "1.15.2".
    pub name: String,

    /// The server's ServerListPing protocol version.
    pub protocol: i32,
}

/// Contains information about a player.
#[derive(Debug, Deserialize)]
pub struct ServerPlayer {
    /// The player's in-game name.
    pub name: String,

    /// The player's UUID.
    pub id: String,
}

/// Contains information about the currently online
/// players.
#[derive(Debug, Deserialize)]
pub struct ServerPlayers {
    /// The configured maximum number of players for the
    /// server.
    pub max: i32,

    /// The number of players currently online.
    pub online: i32,

    /// An optional list of player information for
    /// currently online players.
    pub sample: Option<Vec<ServerPlayer>>,
}

/// Contains the server's MOTD.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ServerDescription {
    Plain(String),
    Object { text: String },
}

/// The decoded JSON response from a status query over
/// ServerListPing.
#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    /// Information about the server's version.
    pub version: ServerVersion,

    /// Information about currently online players.
    pub players: ServerPlayers,

    /// Single-field struct containing the server's MOTD.
    pub description: ServerDescription,

    /// Optional field containing a path to the server's
    /// favicon.
    pub favicon: Option<String>,
}

const LATEST_PROTOCOL_VERSION: usize = 578;
const DEFAULT_PORT: u16 = 25565;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

/// Builder for a Minecraft
/// ServerListPing connection.
pub struct ConnectionConfig {
    protocol_version: usize,
    address: String,
    port: u16,
    timeout: Duration,
    #[cfg(feature = "srv")]
    srv_lookup: bool,
}

impl ConnectionConfig {
    /// Initiates the Minecraft server
    /// connection build process.
    pub fn build<T: Into<String>>(address: T) -> Self {
        ConnectionConfig {
            protocol_version: LATEST_PROTOCOL_VERSION,
            address: address.into(),
            port: DEFAULT_PORT,
            timeout: DEFAULT_TIMEOUT,
            #[cfg(feature = "srv")]
            srv_lookup: false,
        }
    }

    /// Sets a specific
    /// protocol version for the connection to
    /// use. If not specified, the latest version
    /// will be used.
    pub fn with_protocol_version(mut self, protocol_version: usize) -> Self {
        self.protocol_version = protocol_version;
        self
    }

    /// Sets a specific port for the
    /// connection to use. If not specified, the
    /// default port of 25565 will be used.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets a specific timeout for the
    /// connection to use. If not specified, the
    /// timeout defaults to two seconds.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enables SRV record lookup for the connection.
    ///
    /// When enabled, the library will query DNS for an SRV record
    /// at `_minecraft._tcp.<address>`. If found, the target host
    /// and port from the SRV record will be used instead of the
    /// configured address and port.
    ///
    /// This feature requires the `srv` feature to be enabled.
    #[cfg(feature = "srv")]
    pub fn with_srv_lookup(mut self) -> Self {
        self.srv_lookup = true;
        self
    }

    /// Connects to the server and consumes the builder.
    pub async fn connect(self) -> Result<StatusConnection, ServerError> {
        let (address, port) = self.resolve_address().await;

        let stream = tokio::time::timeout(
            self.timeout,
            TcpStream::connect(format!("{}:{}", address, port)),
        )
        .await
        .map_err(|_| ServerError::ConnectionTimedOut)?
        .map_err(|_| ServerError::FailedToConnect)?;

        Ok(StatusConnection {
            stream,
            protocol_version: self.protocol_version,
            address,
            port,
            timeout: self.timeout,
        })
    }

    #[cfg(feature = "srv")]
    async fn resolve_address(&self) -> (String, u16) {
        if !self.srv_lookup {
            return (self.address.clone(), self.port);
        }

        // Try to resolve SRV record, fall back to original address on any failure
        match self.lookup_srv().await {
            Some((host, port)) => (host, port),
            None => (self.address.clone(), self.port),
        }
    }

    #[cfg(not(feature = "srv"))]
    async fn resolve_address(&self) -> (String, u16) {
        (self.address.clone(), self.port)
    }

    #[cfg(feature = "srv")]
    async fn lookup_srv(&self) -> Option<(String, u16)> {
        use hickory_resolver::TokioAsyncResolver;

        let resolver = TokioAsyncResolver::tokio_from_system_conf().ok()?;
        let srv_name = format!("_minecraft._tcp.{}", self.address);

        let lookup = tokio::time::timeout(self.timeout, resolver.srv_lookup(&srv_name))
            .await
            .ok()?
            .ok()?;

        let record = lookup.iter().next()?;
        let target = record.target().to_string();
        // Remove trailing dot from DNS name
        let host = target.trim_end_matches('.').to_string();
        let port = record.port();

        Some((host, port))
    }
}

/// Convenience wrapper for easily connecting
/// to a server on the default port with
/// the latest protocol version.
pub async fn connect(address: String) -> Result<StatusConnection, ServerError> {
    ConnectionConfig::build(address).connect().await
}

/// Wraps a built connection
pub struct StatusConnection {
    stream: TcpStream,
    protocol_version: usize,
    address: String,
    port: u16,
    timeout: Duration,
}

impl StatusConnection {
    /// Sends and reads the packets for the
    /// ServerListPing status call.
    ///
    /// Consumes the connection and returns a type
    /// that can only issue pings. The resulting
    /// status body is accessible via the `status`
    /// property on `PingConnection`.
    pub async fn status(mut self) -> Result<PingConnection, ServerError> {
        let handshake = protocol::HandshakePacket::new(
            self.protocol_version,
            self.address.to_string(),
            self.port,
        );

        self.stream
            .write_packet_with_timeout(handshake, self.timeout)
            .await?;

        self.stream
            .write_packet_with_timeout(protocol::RequestPacket::new(), self.timeout)
            .await?;

        let response: protocol::ResponsePacket =
            self.stream.read_packet_with_timeout(self.timeout).await?;

        let status: StatusResponse = serde_json::from_str(&response.body)
            .map_err(|_| ServerError::InvalidJson(response.body))?;

        Ok(PingConnection {
            stream: self.stream,
            protocol_version: self.protocol_version,
            address: self.address,
            port: self.port,
            status,
            timeout: self.timeout,
        })
    }
}

/// Wraps a built connection
///
/// Constructed by calling `status()` on
/// a `StatusConnection` struct.
#[allow(dead_code)]
pub struct PingConnection {
    stream: TcpStream,
    protocol_version: usize,
    address: String,
    port: u16,
    timeout: Duration,
    pub status: StatusResponse,
}

impl PingConnection {
    /// Sends a ping to the Minecraft server with the
    /// provided payload and asserts that the returned
    /// payload is the same.
    ///
    /// Server closes the connection after a ping call,
    /// so this method consumes the connection.
    pub async fn ping(mut self, payload: u64) -> Result<(), ServerError> {
        let ping = protocol::PingPacket::new(payload);

        self.stream
            .write_packet_with_timeout(ping, self.timeout)
            .await?;

        let pong: protocol::PongPacket = self.stream.read_packet_with_timeout(self.timeout).await?;

        if pong.payload != payload {
            return Err(ServerError::MismatchedPayload {
                expected: payload,
                actual: pong.payload,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_description_plain() {
        let json = r#""A Minecraft Server""#;
        let desc: ServerDescription = serde_json::from_str(json).unwrap();
        assert!(matches!(desc, ServerDescription::Plain(s) if s == "A Minecraft Server"));
    }

    #[test]
    fn test_server_description_object() {
        let json = r#"{"text":"A Minecraft Server"}"#;
        let desc: ServerDescription = serde_json::from_str(json).unwrap();
        assert!(matches!(desc, ServerDescription::Object { text } if text == "A Minecraft Server"));
    }

    #[test]
    fn test_status_response_minimal() {
        let json = r#"{
            "version": {"name": "1.20.4", "protocol": 765},
            "players": {"max": 20, "online": 5},
            "description": "Welcome to the server"
        }"#;

        let response: StatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.version.name, "1.20.4");
        assert_eq!(response.version.protocol, 765);
        assert_eq!(response.players.max, 20);
        assert_eq!(response.players.online, 5);
        assert!(response.players.sample.is_none());
        assert!(response.favicon.is_none());
    }

    #[test]
    fn test_status_response_with_players() {
        let json = r#"{
            "version": {"name": "1.20.4", "protocol": 765},
            "players": {
                "max": 20,
                "online": 2,
                "sample": [
                    {"name": "Player1", "id": "uuid-1"},
                    {"name": "Player2", "id": "uuid-2"}
                ]
            },
            "description": {"text": "Welcome"}
        }"#;

        let response: StatusResponse = serde_json::from_str(json).unwrap();
        let sample = response.players.sample.unwrap();
        assert_eq!(sample.len(), 2);
        assert_eq!(sample[0].name, "Player1");
        assert_eq!(sample[1].name, "Player2");
    }

    #[test]
    fn test_status_response_with_favicon() {
        let json = r#"{
            "version": {"name": "1.20.4", "protocol": 765},
            "players": {"max": 20, "online": 0},
            "description": "Test",
            "favicon": "data:image/png;base64,iVBORw0KGgo="
        }"#;

        let response: StatusResponse = serde_json::from_str(json).unwrap();
        assert!(response.favicon.is_some());
        assert!(response.favicon.unwrap().starts_with("data:image/png"));
    }

    #[test]
    fn test_connection_config_defaults() {
        let config = ConnectionConfig::build("localhost");
        assert_eq!(config.address, "localhost");
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.timeout, DEFAULT_TIMEOUT);
        assert_eq!(config.protocol_version, LATEST_PROTOCOL_VERSION);
    }

    #[test]
    fn test_connection_config_with_port() {
        let config = ConnectionConfig::build("localhost").with_port(12345);
        assert_eq!(config.port, 12345);
    }

    #[test]
    fn test_connection_config_with_timeout() {
        let config = ConnectionConfig::build("localhost").with_timeout(Duration::from_secs(10));
        assert_eq!(config.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_connection_config_with_protocol_version() {
        let config = ConnectionConfig::build("localhost").with_protocol_version(47);
        assert_eq!(config.protocol_version, 47);
    }

    #[cfg(feature = "srv")]
    #[test]
    fn test_connection_config_with_srv_lookup() {
        let config = ConnectionConfig::build("localhost").with_srv_lookup();
        assert!(config.srv_lookup);
    }
}
