use crate::ErrorKind;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::time::Duration;
use tokio::net::ToSocketAddrs;
use tokio::select;
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Box<RawValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<ServerPlayers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<ServerVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<Url>,
    #[serde(default)]
    pub enforces_secure_chat: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ping: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerPlayers {
    pub max: i32,
    pub online: i32,
    #[serde(default)]
    pub sample: Vec<ServerGameProfile>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerGameProfile {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerVersion {
    pub name: String,
    pub protocol: i32,
}

pub async fn get_server_status(
    address: &impl ToSocketAddrs,
    original_address: (&str, u16),
    protocol_version: Option<i32>,
) -> Result<ServerStatus> {
    select! {
        res = modern::status(address, original_address, protocol_version) => res,
        _ = tokio::time::sleep(Duration::from_secs(30)) => Err(ErrorKind::OtherError(
            format!("Ping of {}:{} timed out", original_address.0, original_address.1)
        ).into())
    }
}

mod modern {
    use super::ServerStatus;
    use crate::ErrorKind;
    use chrono::Utc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpStream, ToSocketAddrs};

    pub async fn status(
        address: &impl ToSocketAddrs,
        original_address: (&str, u16),
        protocol_version: Option<i32>,
    ) -> crate::Result<ServerStatus> {
        let mut stream = TcpStream::connect(address).await?;
        handshake(&mut stream, original_address, protocol_version).await?;
        let mut result = status_body(&mut stream).await?;
        result.ping = ping(&mut stream).await.ok();
        Ok(result)
    }

    async fn handshake(
        stream: &mut TcpStream,
        original_address: (&str, u16),
        protocol_version: Option<i32>,
    ) -> crate::Result<()> {
        let (host, port) = original_address;
        let protocol_version = protocol_version.unwrap_or(-1);

        const PACKET_ID: i32 = 0;
        const NEXT_STATE: i32 = 1;

        let packet_size = varint::get_byte_size(PACKET_ID)
            + varint::get_byte_size(protocol_version)
            + varint::get_byte_size(host.len() as i32)
            + host.len()
            + size_of::<u16>()
            + varint::get_byte_size(NEXT_STATE);

        let mut packet_buffer = Vec::with_capacity(
            varint::get_byte_size(packet_size as i32) + packet_size,
        );

        varint::write(&mut packet_buffer, packet_size as i32);
        varint::write(&mut packet_buffer, PACKET_ID);
        varint::write(&mut packet_buffer, protocol_version);
        varint::write(&mut packet_buffer, host.len() as i32);
        packet_buffer.extend_from_slice(host.as_bytes());
        packet_buffer.extend_from_slice(&port.to_be_bytes());
        varint::write(&mut packet_buffer, NEXT_STATE);

        stream.write_all(&packet_buffer).await?;
        stream.flush().await?;

        Ok(())
    }

    async fn status_body(
        stream: &mut TcpStream,
    ) -> crate::Result<ServerStatus> {
        stream.write_all(&[0x01, 0x00]).await?;
        stream.flush().await?;

        let packet_length = varint::read(stream).await?;
        if packet_length < 0 {
            return Err(ErrorKind::InputError(
                "Invalid status response packet length".to_string(),
            )
            .into());
        }

        let mut packet_stream = stream.take(packet_length as u64);
        let packet_id = varint::read(&mut packet_stream).await?;
        if packet_id != 0x00 {
            return Err(ErrorKind::InputError(
                "Unexpected status response".to_string(),
            )
            .into());
        }
        let response_length = varint::read(&mut packet_stream).await?;
        let mut json_response = vec![0_u8; response_length as usize];
        packet_stream.read_exact(&mut json_response).await?;

        if packet_stream.limit() > 0 {
            tokio::io::copy(&mut packet_stream, &mut tokio::io::sink()).await?;
        }

        Ok(serde_json::from_slice(&json_response)?)
    }

    async fn ping(stream: &mut TcpStream) -> crate::Result<i64> {
        let start_time = Utc::now();
        let ping_magic = start_time.timestamp_millis();

        stream.write_all(&[0x09, 0x01]).await?;
        stream.write_i64(ping_magic).await?;
        stream.flush().await?;

        let mut response_prefix = [0_u8; 2];
        stream.read_exact(&mut response_prefix).await?;
        let response_magic = stream.read_i64().await?;
        if response_prefix != [0x09, 0x01] || response_magic != ping_magic {
            return Err(ErrorKind::InputError(
                "Unexpected ping response".to_string(),
            )
            .into());
        }

        let response_time = Utc::now();
        Ok((response_time - start_time).num_milliseconds())
    }

    mod varint {
        use std::io;
        use tokio::io::{AsyncRead, AsyncReadExt};

        const MAX_VARINT_SIZE: usize = 5;
        const DATA_BITS_MASK: u32 = 0x7f;
        const CONT_BIT_MASK_U8: u8 = 0x80;
        const CONT_BIT_MASK_U32: u32 = CONT_BIT_MASK_U8 as u32;
        const DATA_BITS_PER_BYTE: usize = 7;

        pub fn get_byte_size(x: i32) -> usize {
            let x = x as u32;
            for size in 1..MAX_VARINT_SIZE {
                if (x & (u32::MAX << (size * DATA_BITS_PER_BYTE))) == 0 {
                    return size;
                }
            }
            MAX_VARINT_SIZE
        }

        pub fn write(out: &mut Vec<u8>, value: i32) {
            let mut value = value as u32;
            while value >= CONT_BIT_MASK_U32 {
                out.push(((value & DATA_BITS_MASK) | CONT_BIT_MASK_U32) as u8);
                value >>= DATA_BITS_PER_BYTE;
            }
            out.push(value as u8);
        }

        pub async fn read<R: AsyncRead + Unpin>(
            reader: &mut R,
        ) -> io::Result<i32> {
            let mut result = 0;
            let mut shift = 0;

            loop {
                let b = reader.read_u8().await?;
                result |=
                    (b as u32 & DATA_BITS_MASK) << (shift * DATA_BITS_PER_BYTE);
                shift += 1;
                if shift > MAX_VARINT_SIZE {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "VarInt too big",
                    ));
                }
                if b & CONT_BIT_MASK_U8 == 0 {
                    return Ok(result as i32);
                }
            }
        }
    }
}
