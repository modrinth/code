use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::ToSocketAddrs;
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
    original_address: (&str, u16),
    address: &impl ToSocketAddrs,
) -> Result<ServerStatus> {
    let result = modern::status(original_address, address).await;
    if result.is_ok() {
        return result;
    }
    let legacy = legacy::status(original_address, address).await;
    if legacy.is_ok() {
        return legacy;
    }
    result // If the legacy ping fails, return the original failure
}

mod modern {
    use super::ServerStatus;
    use crate::{Error, ErrorKind};
    use chrono::Utc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpStream, ToSocketAddrs};

    pub async fn status(
        original_address: (&str, u16),
        address: &impl ToSocketAddrs,
    ) -> crate::Result<ServerStatus> {
        let mut stream = TcpStream::connect(address).await?;
        handshake(&mut stream, original_address).await?;
        let mut result = status_body(&mut stream).await?;
        result.ping = ping(&mut stream).await.ok();
        Ok(result)
    }

    async fn handshake(
        stream: &mut TcpStream,
        original_address: (&str, u16),
    ) -> crate::Result<()> {
        let (host, port) = original_address;

        const PACKET_ID: i32 = 0;
        const PROTOCOL_VERSION: i32 = -1;
        const NEXT_STATE: i32 = 1;

        let packet_size = varint::get_byte_size(PACKET_ID)
            + varint::get_byte_size(PROTOCOL_VERSION)
            + varint::get_byte_size(host.len() as i32)
            + host.len()
            + size_of::<u16>()
            + varint::get_byte_size(NEXT_STATE);

        let mut packet_buffer = Vec::with_capacity(
            varint::get_byte_size(packet_size as i32) + packet_size,
        );

        varint::write(&mut packet_buffer, packet_size as i32);
        varint::write(&mut packet_buffer, PACKET_ID);
        varint::write(&mut packet_buffer, PROTOCOL_VERSION);
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
            return Err(Error::from(ErrorKind::InputError(
                "Invalid status response packet length".to_string(),
            )));
        }

        let mut packet_stream = stream.take(packet_length as u64);
        let packet_id = varint::read(&mut packet_stream).await?;
        if packet_id != 0x00 {
            return Err(Error::from(ErrorKind::InputError(
                "Unexpected status response".to_string(),
            )));
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
            return Err(Error::from(ErrorKind::InputError(
                "Unexpected ping response".to_string(),
            )));
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
            while (value & CONT_BIT_MASK_U32) != 0 {
                out.push((value & DATA_BITS_MASK) as u8 | CONT_BIT_MASK_U8);
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

mod legacy {
    use super::ServerStatus;
    use crate::worlds::{ServerPlayers, ServerVersion};
    use crate::{Error, ErrorKind};
    use serde_json::value::to_raw_value;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpStream, ToSocketAddrs};

    pub async fn status(
        original_address: (&str, u16),
        address: &impl ToSocketAddrs,
    ) -> crate::Result<ServerStatus> {
        let mut packet = vec![0xfe, 0x01, 0xfa];
        write_legacy(&mut packet, "MC|PingHost");

        let (host, port) = original_address;
        let len_index = packet.len();
        packet.push(0x4a);
        write_legacy(&mut packet, host);
        packet.extend_from_slice(&(port as u32).to_be_bytes());
        packet.splice(
            len_index..len_index,
            ((packet.len() - len_index) as u16).to_be_bytes(),
        );

        let mut stream = TcpStream::connect(address).await?;
        stream.write_all(&packet).await?;
        stream.flush().await?;

        let packet_id = stream.read_u8().await?;
        if packet_id != 0xff {
            return Err(Error::from(ErrorKind::InputError(
                "Unexpected legacy status response".to_string(),
            )));
        }

        let data_length = stream.read_u16().await?;
        let mut data = vec![0u8; data_length as usize * 2];
        stream.read_exact(&mut data).await?;

        drop(stream);

        let data = String::from_utf16_lossy(
            &data
                .chunks_exact(2)
                .into_iter()
                .map(|a| u16::from_be_bytes([a[0], a[1]]))
                .collect::<Vec<u16>>(),
        );
        let mut parts = data.split('\0');
        if parts.next() != Some("§1") {
            return Err(Error::from(ErrorKind::InputError(
                "Legacy response status too old".to_string(),
            )));
        }

        Ok(ServerStatus {
            version: Some(ServerVersion {
                protocol: parts
                    .next()
                    .and_then(|x| x.parse().ok())
                    .unwrap_or(0),
                name: parts.next().unwrap_or("").to_owned(),
            }),
            description: parts.next().and_then(|x| to_raw_value(x).ok()),
            players: Some(ServerPlayers {
                online: parts.next().and_then(|x| x.parse().ok()).unwrap_or(-1),
                max: parts.next().and_then(|x| x.parse().ok()).unwrap_or(-1),
                sample: vec![],
            }),
            favicon: None,
            enforces_secure_chat: false,
            ping: None,
        })
    }

    fn write_legacy(out: &mut Vec<u8>, text: &str) {
        let encoded = text.encode_utf16().collect::<Vec<_>>();
        out.extend_from_slice(&(encoded.len() as u16).to_be_bytes());
        encoded
            .into_iter()
            .map(|x| x.to_be_bytes())
            .flatten()
            .for_each(|x| out.push(x));
    }
}
