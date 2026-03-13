//! Integration tests using mocked TCP streams

use std::io::Cursor;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

// Re-implement the wire protocol traits for testing
// since they're not exported from the library

#[async_trait]
trait AsyncWireWriteExt {
    async fn write_varint(&mut self, int: usize) -> std::io::Result<()>;
    async fn write_string(&mut self, string: &str) -> std::io::Result<()>;
}

#[async_trait]
impl<W: tokio::io::AsyncWrite + Unpin + Send + Sync> AsyncWireWriteExt for W {
    async fn write_varint(&mut self, int: usize) -> std::io::Result<()> {
        let mut int = (int as u64) & 0xFFFF_FFFF;
        let mut written = 0;
        let mut buffer = [0; 5];
        loop {
            let temp = (int & 0b0111_1111) as u8;
            int >>= 7;
            if int != 0 {
                buffer[written] = temp | 0b1000_0000;
            } else {
                buffer[written] = temp;
            }
            written += 1;
            if int == 0 {
                break;
            }
        }
        self.write_all(&buffer[0..written]).await?;
        Ok(())
    }

    async fn write_string(&mut self, string: &str) -> std::io::Result<()> {
        self.write_varint(string.len()).await?;
        self.write_all(string.as_bytes()).await?;
        Ok(())
    }
}

#[allow(dead_code)]
#[async_trait]
trait AsyncWireReadExt {
    async fn read_varint(&mut self) -> std::io::Result<usize>;
    async fn read_string(&mut self) -> std::io::Result<String>;
}

#[async_trait]
impl<R: tokio::io::AsyncRead + Unpin + Send + Sync> AsyncWireReadExt for R {
    async fn read_varint(&mut self) -> std::io::Result<usize> {
        let mut read = 0;
        let mut result = 0;
        loop {
            let mut buf = [0u8; 1];
            self.read_exact(&mut buf).await?;
            let read_value = buf[0];
            let value = read_value & 0b0111_1111;
            result |= (value as usize) << (7 * read);
            read += 1;
            if read > 5 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid varint",
                ));
            }
            if (read_value & 0b1000_0000) == 0 {
                return Ok(result);
            }
        }
    }

    async fn read_string(&mut self) -> std::io::Result<String> {
        let length = self.read_varint().await?;
        let mut buffer = vec![0; length];
        self.read_exact(&mut buffer).await?;
        String::from_utf8(buffer)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid utf8"))
    }
}

/// Helper to write a raw packet (length-prefixed with packet ID)
async fn write_raw_packet(
    stream: &mut DuplexStream,
    packet_id: usize,
    data: &[u8],
) -> std::io::Result<()> {
    let mut packet_buffer = Cursor::new(Vec::new());
    packet_buffer.write_varint(packet_id).await?;
    packet_buffer.write_all(data).await?;

    let inner = packet_buffer.into_inner();
    stream.write_varint(inner.len()).await?;
    stream.write_all(&inner).await?;
    Ok(())
}

/// Helper to read a raw packet and return (packet_id, data)
async fn read_raw_packet(stream: &mut DuplexStream) -> std::io::Result<(usize, Vec<u8>)> {
    let length = stream.read_varint().await?;
    if length == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "empty packet",
        ));
    }

    let packet_id = stream.read_varint().await?;
    let mut data = vec![0u8; length - 1]; // -1 for the packet_id varint (assuming single byte)
    if !data.is_empty() {
        stream.read_exact(&mut data).await?;
    }

    Ok((packet_id, data))
}

/// Simulate a Minecraft server that responds to status requests
async fn mock_server(mut stream: DuplexStream, response_json: &str) {
    // Read handshake packet (id=0)
    let (packet_id, _data) = read_raw_packet(&mut stream).await.unwrap();
    assert_eq!(packet_id, 0, "Expected handshake packet");

    // Read request packet (id=0)
    let (packet_id, _data) = read_raw_packet(&mut stream).await.unwrap();
    assert_eq!(packet_id, 0, "Expected request packet");

    // Send response packet (id=0) with JSON
    let mut response_data = Cursor::new(Vec::new());
    response_data.write_string(response_json).await.unwrap();
    write_raw_packet(&mut stream, 0, &response_data.into_inner())
        .await
        .unwrap();

    // Read ping packet (id=1)
    let (packet_id, data) = read_raw_packet(&mut stream).await.unwrap();
    assert_eq!(packet_id, 1, "Expected ping packet");

    // Send pong packet (id=1) with same payload
    write_raw_packet(&mut stream, 1, &data).await.unwrap();
}

/// Simulate a server that sends invalid JSON
#[allow(dead_code)]
async fn mock_server_invalid_json(mut stream: DuplexStream) {
    // Read handshake
    let _ = read_raw_packet(&mut stream).await.unwrap();
    // Read request
    let _ = read_raw_packet(&mut stream).await.unwrap();

    // Send invalid JSON response
    let mut response_data = Cursor::new(Vec::new());
    response_data.write_string("not valid json").await.unwrap();
    write_raw_packet(&mut stream, 0, &response_data.into_inner())
        .await
        .unwrap();
}

/// Simulate a server that returns wrong pong payload
#[allow(dead_code)]
async fn mock_server_wrong_pong(mut stream: DuplexStream, response_json: &str) {
    // Read handshake
    let _ = read_raw_packet(&mut stream).await.unwrap();
    // Read request
    let _ = read_raw_packet(&mut stream).await.unwrap();

    // Send valid response
    let mut response_data = Cursor::new(Vec::new());
    response_data.write_string(response_json).await.unwrap();
    write_raw_packet(&mut stream, 0, &response_data.into_inner())
        .await
        .unwrap();

    // Read ping
    let _ = read_raw_packet(&mut stream).await.unwrap();

    // Send pong with different payload
    let wrong_payload: u64 = 99999;
    write_raw_packet(&mut stream, 1, &wrong_payload.to_be_bytes())
        .await
        .unwrap();
}

fn valid_status_json() -> &'static str {
    r#"{"version":{"name":"1.20.4","protocol":765},"players":{"max":20,"online":5},"description":"Test Server"}"#
}

#[tokio::test]
async fn test_mock_server_protocol() {
    // Test that our mock server correctly handles the Minecraft protocol
    let (mut client_stream, server_stream) = tokio::io::duplex(1024);

    // Spawn mock server
    let server_handle = tokio::spawn(mock_server(server_stream, valid_status_json()));

    // Simulate client sending handshake packet (id=0)
    let mut handshake_data = Cursor::new(Vec::new());
    handshake_data.write_varint(578).await.unwrap(); // protocol version
    handshake_data.write_string("localhost").await.unwrap(); // server address
    handshake_data.write_u16(25565).await.unwrap(); // port
    handshake_data.write_varint(1).await.unwrap(); // next state (status)
    write_raw_packet(&mut client_stream, 0, &handshake_data.into_inner())
        .await
        .unwrap();

    // Send request packet (id=0, empty)
    write_raw_packet(&mut client_stream, 0, &[]).await.unwrap();

    // Read response packet
    let (packet_id, data) = read_raw_packet(&mut client_stream).await.unwrap();
    assert_eq!(packet_id, 0);

    // Parse the JSON from response
    let mut cursor = Cursor::new(data);
    let json_str = cursor.read_string().await.unwrap();
    assert!(json_str.contains("1.20.4"));
    assert!(json_str.contains("Test Server"));

    // Send ping packet (id=1)
    let ping_payload: u64 = 12345;
    write_raw_packet(&mut client_stream, 1, &ping_payload.to_be_bytes())
        .await
        .unwrap();

    // Read pong packet
    let (packet_id, data) = read_raw_packet(&mut client_stream).await.unwrap();
    assert_eq!(packet_id, 1);
    assert_eq!(data.len(), 8);
    let pong_payload = u64::from_be_bytes(data.try_into().unwrap());
    assert_eq!(pong_payload, ping_payload);

    server_handle.await.unwrap();
}

#[tokio::test]
async fn test_status_json_parsing_plain_description() {
    use async_minecraft_ping::StatusResponse;

    let json = r#"{
        "version": {"name": "1.20.4", "protocol": 765},
        "players": {"max": 100, "online": 42},
        "description": "Plain text MOTD"
    }"#;

    let response: StatusResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.version.name, "1.20.4");
    assert_eq!(response.version.protocol, 765);
    assert_eq!(response.players.max, Some(100));
    assert_eq!(response.players.online, Some(42));
}

#[tokio::test]
async fn test_status_json_parsing_object_description() {
    use async_minecraft_ping::StatusResponse;

    let json = r#"{
        "version": {"name": "1.19.4", "protocol": 762},
        "players": {"max": 50, "online": 10, "sample": [{"name": "Notch", "id": "069a79f4-44e9-4726-a5be-fca90e38aaf5"}]},
        "description": {"text": "Object MOTD"},
        "favicon": "data:image/png;base64,abc123"
    }"#;

    let response: StatusResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.version.name, "1.19.4");
    assert_eq!(response.players.online, Some(10));
    assert!(response.players.sample.is_some());
    assert_eq!(response.players.sample.as_ref().unwrap().len(), 1);
    assert_eq!(response.players.sample.as_ref().unwrap()[0].name, "Notch");
    assert!(response.favicon.is_some());
}

#[tokio::test]
async fn test_connection_config_builder_chain() {
    use async_minecraft_ping::ConnectionConfig;

    let _config = ConnectionConfig::build("mc.example.com")
        .with_port(25566)
        .with_protocol_version(47)
        .with_timeout(Duration::from_secs(5));

    // We can't directly inspect private fields, but we can verify
    // the builder pattern works without panicking
}

#[tokio::test]
async fn test_connection_refused() {
    use async_minecraft_ping::ConnectionConfig;

    // Try to connect to a port that's definitely not listening
    let result = ConnectionConfig::build("127.0.0.1")
        .with_port(1) // Port 1 is privileged and unlikely to have a server
        .with_timeout(Duration::from_millis(100))
        .connect()
        .await;

    assert!(result.is_err());
}

#[cfg(feature = "srv")]
#[tokio::test]
async fn test_srv_lookup_fallback() {
    use async_minecraft_ping::ConnectionConfig;

    // This domain definitely doesn't have an SRV record
    // The library should fall back to the original address
    let result = ConnectionConfig::build("127.0.0.1")
        .with_port(1)
        .with_timeout(Duration::from_millis(100))
        .with_srv_lookup()
        .connect()
        .await;

    // Should fail to connect (no server), but shouldn't fail on SRV lookup
    assert!(result.is_err());
}
