use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::models::exp;
use chrono::Utc;
use clickhouse::{Client, Row};
use futures::TryStreamExt;
use serde::Serialize;
use sqlx::types::Json;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct ServerPingQueue {
    pub pg: PgPool,
    pub redis: RedisPool,
    pub clickhouse: Client,
}

impl ServerPingQueue {
    pub fn new(pg: PgPool, redis: RedisPool, clickhouse: Client) -> Self {
        Self {
            pg,
            redis,
            clickhouse,
        }
    }

    pub async fn ping_minecraft_java_servers(
        &self,
    ) -> Result<(), ServerPingError> {
        let mut stream = sqlx::query!(
            r#"
            SELECT id, components AS "components: Json<exp::ProjectSerial>"
            FROM mods
            WHERE status = 'approved'
            "#
        )
        .fetch(&self.pg);

        let mut ping_results = Vec::new();

        while let Some(row) = stream.try_next().await? {
            let project_id: u64 = row.id as u64;
            let components: exp::ProjectSerial = row.components.0;

            if let Some(java_server) = components.minecraft_java_server {
                let java_server: exp::minecraft::JavaServerProject =
                    exp::component::Component::from_db(java_server);
                let address = &java_server.address;
                let port = java_server.port;

                let ping_result =
                    self.ping_minecraft_server(address, port).await;
                let recorded = Utc::now().timestamp_millis();

                ping_results.push(ServerPingRecord {
                    recorded,
                    project_id,
                    address: address.clone(),
                    port,
                    online: ping_result.is_some(),
                    latency_ms: ping_result.map(|r| r.as_millis() as u32),
                });
            }
        }

        if !ping_results.is_empty() {
            let mut insert = self
                .clickhouse
                .insert::<ServerPingRecord>("minecraft_java_server_pings")
                .await?;

            for result in &ping_results {
                insert.write(result).await?;
            }

            insert.end().await?;
        }

        Ok(())
    }

    async fn ping_minecraft_server(
        &self,
        address: &str,
        port: u16,
    ) -> Option<std::time::Duration> {
        let start = Instant::now();

        match TcpStream::connect((address, port)).await {
            Ok(mut stream) => {
                let handshake = create_handshake(address, port);
                let status_request = vec![0x01, 0x00];

                if stream.write_all(&handshake).await.is_err() {
                    return None;
                }

                if stream.write_all(&status_request).await.is_err() {
                    return None;
                }

                if stream.flush().await.is_err() {
                    return None;
                }

                let mut response_len_bytes = [0u8; 5];
                if stream.read_exact(&mut response_len_bytes).await.is_err() {
                    return None;
                }

                let response_len = varint_decode(&response_len_bytes[1..])?;

                let mut response = vec![0u8; response_len as usize + 1];
                response[0] = response_len_bytes[0];

                if stream.read_exact(&mut response[1..]).await.is_err() {
                    return None;
                }

                Some(start.elapsed())
            }
            Err(_) => None,
        }
    }
}

fn create_handshake(address: &str, port: u16) -> Vec<u8> {
    let mut packet = Vec::new();

    packet.extend_from_slice(&varint_encode(0x00));

    packet.extend_from_slice(&varint_encode(765));
    packet.extend_from_slice(&varint_encode(address.len() as i32));
    packet.extend_from_slice(address.as_bytes());

    packet.extend_from_slice(&port.to_be_bytes());

    packet.extend_from_slice(&varint_encode(1));

    let mut handshake = Vec::new();
    handshake.extend_from_slice(&varint_encode(packet.len() as i32));
    handshake.extend_from_slice(&packet);

    handshake
}

fn varint_encode(mut value: i32) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
    bytes
}

fn varint_decode(bytes: &[u8]) -> Option<i32> {
    let mut result = 0i32;
    let mut shift = 0;

    for &byte in bytes {
        result |= ((byte & 0x7F) as i32) << shift;
        if byte & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
        if shift >= 32 {
            return None;
        }
    }

    None
}

#[derive(Debug, Row, Serialize, Clone)]
struct ServerPingRecord {
    recorded: i64,
    project_id: u64,
    address: String,
    port: u16,
    online: bool,
    latency_ms: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum ServerPingError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Clickhouse error: {0}")]
    Clickhouse(#[from] clickhouse::error::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
