//! This module defines various methods to read and
//! write packets in Minecraft's
//! [ServerListPing](https://wiki.vg/Server_List_Ping)
//! protocol.

use std::io::Cursor;
use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("error reading or writing data")]
    Io(#[from] std::io::Error),

    #[error("invalid packet length")]
    InvalidPacketLength,

    #[error("invalid varint data")]
    InvalidVarInt,

    #[error("invalid packet (expected ID {expected:?}, actual ID {actual:?})")]
    InvalidPacketId { expected: usize, actual: usize },

    #[error("invalid ServerListPing response body (invalid UTF-8)")]
    InvalidResponseBody,

    #[error("connection timed out")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

/// State represents the desired next state of the
/// exchange.
///
/// It's a bit silly now as there's only
/// one entry, but technically there is more than
/// one type that can be sent here.
#[derive(Clone, Copy)]
pub enum State {
    Status,
}

impl From<State> for usize {
    fn from(state: State) -> Self {
        match state {
            State::Status => 1,
        }
    }
}

/// RawPacket is the underlying wrapper of data that
/// gets read from and written to the socket.
///
/// Typically, the flow looks like this:
/// 1. Construct a specific packet (HandshakePacket
///    for example).
/// 2. Write that packet's contents to a byte buffer.
/// 3. Construct a RawPacket using that byte buffer.
/// 4. Write the RawPacket to the socket.
struct RawPacket {
    id: usize,
    data: Box<[u8]>,
}

impl RawPacket {
    fn new(id: usize, data: Box<[u8]>) -> Self {
        RawPacket { id, data }
    }
}

/// AsyncWireReadExt adds varint and varint-backed
/// string support to things that implement AsyncRead.
#[async_trait]
pub trait AsyncWireReadExt {
    async fn read_varint(&mut self) -> Result<usize, ProtocolError>;
    async fn read_string(&mut self) -> Result<String, ProtocolError>;
}

#[async_trait]
impl<R: AsyncRead + Unpin + Send + Sync> AsyncWireReadExt for R {
    async fn read_varint(&mut self) -> Result<usize, ProtocolError> {
        let mut read = 0;
        let mut result = 0;
        loop {
            let read_value = self.read_u8().await?;
            let value = read_value & 0b0111_1111;
            result |= (value as usize) << (7 * read);
            read += 1;
            if read > 5 {
                return Err(ProtocolError::InvalidVarInt);
            }
            if (read_value & 0b1000_0000) == 0 {
                return Ok(result);
            }
        }
    }

    async fn read_string(&mut self) -> Result<String, ProtocolError> {
        let length = self.read_varint().await?;

        let mut buffer = vec![0; length];
        self.read_exact(&mut buffer).await?;

        Ok(String::from_utf8(buffer).map_err(|_| ProtocolError::InvalidResponseBody)?)
    }
}

/// AsyncWireWriteExt adds varint and varint-backed
/// string support to things that implement AsyncWrite.
#[async_trait]
pub trait AsyncWireWriteExt {
    async fn write_varint(&mut self, int: usize) -> Result<(), ProtocolError>;
    async fn write_string(&mut self, string: &str) -> Result<(), ProtocolError>;
}

#[async_trait]
impl<W: AsyncWrite + Unpin + Send + Sync> AsyncWireWriteExt for W {
    async fn write_varint(&mut self, int: usize) -> Result<(), ProtocolError> {
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

    async fn write_string(&mut self, string: &str) -> Result<(), ProtocolError> {
        self.write_varint(string.len()).await?;
        self.write_all(string.as_bytes()).await?;

        Ok(())
    }
}

/// PacketId is used to allow AsyncWriteRawPacket
/// to generically get a packet's ID.
pub trait PacketId {
    fn get_packet_id(&self) -> usize;
}

/// ExpectedPacketId is used to allow AsyncReadRawPacket
/// to generically get a packet's expected ID.
pub trait ExpectedPacketId {
    fn get_expected_packet_id() -> usize;
}

/// AsyncReadFromBuffer is used to allow
/// AsyncReadRawPacket to generically read a
/// packet's specific data from a buffer.
#[async_trait]
pub trait AsyncReadFromBuffer: Sized {
    async fn read_from_buffer(buffer: Vec<u8>) -> Result<Self, ProtocolError>;
}

/// AsyncWriteToBuffer is used to allow
/// AsyncWriteRawPacket to generically write a
/// packet's specific data into a buffer.
#[async_trait]
pub trait AsyncWriteToBuffer {
    async fn write_to_buffer(&self) -> Result<Vec<u8>, ProtocolError>;
}

/// AsyncReadRawPacket is the core piece of
/// the read side of the protocol. It allows
/// the user to construct a specific packet
/// from something that implements AsyncRead.
#[async_trait]
pub trait AsyncReadRawPacket {
    async fn read_packet<T: ExpectedPacketId + AsyncReadFromBuffer + Send + Sync>(
        &mut self,
    ) -> Result<T, ProtocolError>;

    async fn read_packet_with_timeout<T: ExpectedPacketId + AsyncReadFromBuffer + Send + Sync>(
        &mut self,
        timeout: Duration,
    ) -> Result<T, ProtocolError>;
}

#[async_trait]
impl<R: AsyncRead + Unpin + Send + Sync> AsyncReadRawPacket for R {
    async fn read_packet<T: ExpectedPacketId + AsyncReadFromBuffer + Send + Sync>(
        &mut self,
    ) -> Result<T, ProtocolError> {
        let length = self.read_varint().await?;

        if length == 0 {
            return Err(ProtocolError::InvalidPacketLength);
        }

        let packet_id = self.read_varint().await?;

        let expected_packet_id = T::get_expected_packet_id();

        if packet_id != expected_packet_id {
            return Err(ProtocolError::InvalidPacketId {
                expected: expected_packet_id,
                actual: packet_id,
            });
        }

        let mut buffer = vec![0; length - 1];
        self.read_exact(&mut buffer).await?;

        T::read_from_buffer(buffer).await
    }

    async fn read_packet_with_timeout<T: ExpectedPacketId + AsyncReadFromBuffer + Send + Sync>(
        &mut self,
        timeout: Duration,
    ) -> Result<T, ProtocolError> {
        tokio::time::timeout(timeout, self.read_packet()).await?
    }
}

/// AsyncWriteRawPacket is the core piece of
/// the write side of the protocol. It allows
/// the user to write a specific packet to
/// something that implements AsyncWrite.
#[async_trait]
pub trait AsyncWriteRawPacket {
    async fn write_packet<T: PacketId + AsyncWriteToBuffer + Send + Sync>(
        &mut self,
        packet: T,
    ) -> Result<(), ProtocolError>;

    async fn write_packet_with_timeout<T: PacketId + AsyncWriteToBuffer + Send + Sync>(
        &mut self,
        packet: T,
        timeout: Duration,
    ) -> Result<(), ProtocolError>;
}

#[async_trait]
impl<W: AsyncWrite + Unpin + Send + Sync> AsyncWriteRawPacket for W {
    async fn write_packet<T: PacketId + AsyncWriteToBuffer + Send + Sync>(
        &mut self,
        packet: T,
    ) -> Result<(), ProtocolError> {
        let packet_buffer = packet.write_to_buffer().await?;

        let raw_packet = RawPacket::new(packet.get_packet_id(), packet_buffer.into_boxed_slice());

        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        buffer.write_varint(raw_packet.id).await?;
        buffer.write_all(&raw_packet.data).await?;

        let inner = buffer.into_inner();
        self.write_varint(inner.len()).await?;
        self.write_all(&inner).await?;
        Ok(())
    }

    async fn write_packet_with_timeout<T: PacketId + AsyncWriteToBuffer + Send + Sync>(
        &mut self,
        packet: T,
        timeout: Duration,
    ) -> Result<(), ProtocolError> {
        tokio::time::timeout(timeout, self.write_packet(packet)).await?
    }
}

/// HandshakePacket is the first of two packets
/// to be sent during a status check for
/// ServerListPing.
pub struct HandshakePacket {
    pub packet_id: usize,
    pub protocol_version: usize,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: State,
}

impl HandshakePacket {
    pub fn new(protocol_version: usize, server_address: String, server_port: u16) -> Self {
        Self {
            packet_id: 0,
            protocol_version,
            server_address,
            server_port,
            next_state: State::Status,
        }
    }
}

#[async_trait]
impl AsyncWriteToBuffer for HandshakePacket {
    async fn write_to_buffer(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buffer = Cursor::new(Vec::<u8>::new());

        buffer.write_varint(self.protocol_version).await?;
        buffer.write_string(&self.server_address).await?;
        buffer.write_u16(self.server_port).await?;
        buffer.write_varint(self.next_state.into()).await?;

        Ok(buffer.into_inner())
    }
}

impl PacketId for HandshakePacket {
    fn get_packet_id(&self) -> usize {
        self.packet_id
    }
}

/// RequestPacket is the second of two packets
/// to be sent during a status check for
/// ServerListPing.
pub struct RequestPacket {
    pub packet_id: usize,
}

impl RequestPacket {
    pub fn new() -> Self {
        Self { packet_id: 0 }
    }
}

#[async_trait]
impl AsyncWriteToBuffer for RequestPacket {
    async fn write_to_buffer(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}

impl PacketId for RequestPacket {
    fn get_packet_id(&self) -> usize {
        self.packet_id
    }
}

/// ResponsePacket is the response from the
/// server to a status check for
/// ServerListPing.
pub struct ResponsePacket {
    #[allow(dead_code)]
    pub packet_id: usize,
    pub body: String,
}

impl ExpectedPacketId for ResponsePacket {
    fn get_expected_packet_id() -> usize {
        0
    }
}

#[async_trait]
impl AsyncReadFromBuffer for ResponsePacket {
    async fn read_from_buffer(buffer: Vec<u8>) -> Result<Self, ProtocolError> {
        let mut reader = Cursor::new(buffer);

        let body = reader.read_string().await?;

        Ok(ResponsePacket { packet_id: 0, body })
    }
}

pub struct PingPacket {
    pub packet_id: usize,
    pub payload: u64,
}

impl PingPacket {
    pub fn new(payload: u64) -> Self {
        Self {
            packet_id: 1,
            payload,
        }
    }
}

#[async_trait]
impl AsyncWriteToBuffer for PingPacket {
    async fn write_to_buffer(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buffer = Cursor::new(Vec::<u8>::new());

        buffer.write_u64(self.payload).await?;

        Ok(buffer.into_inner())
    }
}

impl PacketId for PingPacket {
    fn get_packet_id(&self) -> usize {
        self.packet_id
    }
}

pub struct PongPacket {
    #[allow(dead_code)]
    pub packet_id: usize,
    pub payload: u64,
}

impl ExpectedPacketId for PongPacket {
    fn get_expected_packet_id() -> usize {
        1
    }
}

#[async_trait]
impl AsyncReadFromBuffer for PongPacket {
    async fn read_from_buffer(buffer: Vec<u8>) -> Result<Self, ProtocolError> {
        let mut reader = Cursor::new(buffer);

        let payload = reader.read_u64().await?;

        Ok(PongPacket {
            packet_id: 0,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_varint_roundtrip() {
        let test_cases = vec![
            0usize, 1, 127, 128, 255, 256, 16383, 16384, 2097151, 2097152, 268435455,
        ];

        for value in test_cases {
            let mut buffer = Cursor::new(Vec::new());
            buffer.write_varint(value).await.unwrap();

            let mut reader = Cursor::new(buffer.into_inner());
            let result = reader.read_varint().await.unwrap();

            assert_eq!(value, result, "Varint roundtrip failed for {}", value);
        }
    }

    #[tokio::test]
    async fn test_varint_encoding() {
        // Test specific known encodings
        let cases = vec![
            (0usize, vec![0x00]),
            (1, vec![0x01]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (25565, vec![0xdd, 0xc7, 0x01]),
            (2097151, vec![0xff, 0xff, 0x7f]),
        ];

        for (value, expected) in cases {
            let mut buffer = Cursor::new(Vec::new());
            buffer.write_varint(value).await.unwrap();
            assert_eq!(
                buffer.into_inner(),
                expected,
                "Varint encoding failed for {}",
                value
            );
        }
    }

    #[tokio::test]
    async fn test_string_roundtrip() {
        let test_cases = vec![
            "",
            "hello",
            "localhost",
            "mc.example.com",
            "こんにちは", // Unicode
        ];

        for s in test_cases {
            let mut buffer = Cursor::new(Vec::new());
            buffer.write_string(s).await.unwrap();

            let mut reader = Cursor::new(buffer.into_inner());
            let result = reader.read_string().await.unwrap();

            assert_eq!(s, result, "String roundtrip failed for {:?}", s);
        }
    }

    #[tokio::test]
    async fn test_handshake_packet_serialization() {
        let packet = HandshakePacket::new(578, "localhost".to_string(), 25565);
        let buffer = packet.write_to_buffer().await.unwrap();

        // Verify the buffer contains expected data
        let mut reader = Cursor::new(buffer);

        // Protocol version (578 as varint)
        let protocol = reader.read_varint().await.unwrap();
        assert_eq!(protocol, 578);

        // Server address
        let address = reader.read_string().await.unwrap();
        assert_eq!(address, "localhost");

        // Server port (big-endian u16)
        let port = reader.read_u16().await.unwrap();
        assert_eq!(port, 25565);

        // Next state (1 for status)
        let state = reader.read_varint().await.unwrap();
        assert_eq!(state, 1);
    }

    #[tokio::test]
    async fn test_request_packet_serialization() {
        let packet = RequestPacket::new();
        let buffer = packet.write_to_buffer().await.unwrap();

        // Request packet has no data
        assert!(buffer.is_empty());
    }

    #[tokio::test]
    async fn test_ping_packet_serialization() {
        let packet = PingPacket::new(12345678);
        let buffer = packet.write_to_buffer().await.unwrap();

        // Ping packet contains a u64 payload (8 bytes, big-endian)
        assert_eq!(buffer.len(), 8);

        let mut reader = Cursor::new(buffer);
        let payload = reader.read_u64().await.unwrap();
        assert_eq!(payload, 12345678);
    }

    #[tokio::test]
    async fn test_response_packet_deserialization() {
        // Create a buffer with a JSON string
        let json = r#"{"version":{"name":"1.20.4","protocol":765}}"#;
        let mut buffer = Cursor::new(Vec::new());
        buffer.write_string(json).await.unwrap();

        let packet = ResponsePacket::read_from_buffer(buffer.into_inner())
            .await
            .unwrap();
        assert_eq!(packet.body, json);
    }

    #[tokio::test]
    async fn test_pong_packet_deserialization() {
        let payload: u64 = 987654321;
        let buffer = payload.to_be_bytes().to_vec();

        let packet = PongPacket::read_from_buffer(buffer).await.unwrap();
        assert_eq!(packet.payload, payload);
    }

    #[tokio::test]
    async fn test_invalid_varint() {
        // A varint with more than 5 continuation bytes is invalid
        let invalid = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        let mut reader = Cursor::new(invalid);

        let result = reader.read_varint().await;
        assert!(matches!(result, Err(ProtocolError::InvalidVarInt)));
    }
}
