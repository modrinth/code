use crate::database::redis::RedisPool;
use crate::models::exp;
use crate::{database::PgPool, util::error::Context};
use async_minecraft_ping::{ServerDescription, StatusResponse};
use chrono::Utc;
use clickhouse::{Client, Row};
use futures::TryStreamExt;
use serde::Serialize;
use sqlx::types::Json;
use std::time::{Duration, Instant};
use tracing::info;

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

    pub async fn ping_minecraft_java_servers(&self) -> eyre::Result<()> {
        let mut server_projects = sqlx::query!(
            r#"
            SELECT id, components AS "components: Json<exp::ProjectSerial>"
            FROM mods
            WHERE status = 'approved' AND components ? 'minecraft_java_server'
            "#
        )
        .fetch(&self.pg);

        let mut ping_results = Vec::new();

        while let Some(row) = server_projects.try_next().await? {
            let project_id: u64 = row.id as u64;
            let components: exp::ProjectSerial = row.components.0;

            let Some(java_server) = components.minecraft_java_server else {
                continue;
            };

            let java_server: exp::minecraft::JavaServerProject =
                exp::component::Component::from_serial(java_server);
            let address = &java_server.address;
            let port = java_server.port;

            let recorded = Utc::now().timestamp_millis();
            let ping_record = match self.ping_server(address, port).await {
                Ok((status, latency)) => ServerPingRecord {
                    recorded,
                    project_id,
                    address: address.clone(),
                    port,
                    latency_ms: Some(latency.as_millis() as u32),
                    description: match status.description {
                        ServerDescription::Plain(text)
                        | ServerDescription::Object { text } => Some(text),
                    },
                    version_name: Some(status.version.name),
                    version_protocol: Some(status.version.protocol),
                    players_online: Some(status.players.online),
                    players_max: Some(status.players.max),
                },
                Err(err) => {
                    info!("Failed to ping {address}:{port}: {err:?}");
                    ServerPingRecord {
                        recorded,
                        project_id,
                        address: address.clone(),
                        port,
                        latency_ms: None,
                        description: None,
                        version_name: None,
                        version_protocol: None,
                        players_online: None,
                        players_max: None,
                    }
                }
            };

            ping_results.push(ping_record);
        }

        if !ping_results.is_empty() {
            let mut insert = self
                .clickhouse
                .insert::<ServerPingRecord>("minecraft_java_server_pings")
                .await
                .wrap_err("failed to begin inserting ping records")?;

            for result in &ping_results {
                insert
                    .write(result)
                    .await
                    .wrap_err("failed to write ping record")?;
            }

            insert
                .end()
                .await
                .wrap_err("failed to end inserting ping records")?;
        }

        Ok(())
    }

    async fn ping_server(
        &self,
        address: &str,
        port: u16,
    ) -> eyre::Result<(StatusResponse, Duration)> {
        let start = Instant::now();

        let task = async move {
            let conn = async_minecraft_ping::ConnectionConfig::build(address)
                .with_port(port)
                .connect()
                .await
                .wrap_err("failed to connect to server")?;

            let status = conn
                .status()
                .await
                .wrap_err("failed to get server status")?
                .status;
            Ok((status, start.elapsed()))
        };

        let timeout = dotenvy::var("SERVER_PING_TIMEOUT")
            .unwrap()
            .parse::<u64>()
            .wrap_err("failed to parse SERVER_PING_TIMEOUT")?;

        tokio::time::timeout(Duration::from_millis(timeout), task)
            .await
            .wrap_err("server ping timed out")
            .flatten()
    }
}

#[derive(Debug, Row, Serialize, Clone)]
struct ServerPingRecord {
    recorded: i64,
    project_id: u64,
    address: String,
    port: u16,
    latency_ms: Option<u32>,
    description: Option<String>,
    version_name: Option<String>,
    version_protocol: Option<u32>,
    players_online: Option<u32>,
    players_max: Option<u32>,
}

#[cfg(test)]
mod tests {
    use crate::test::{
        api_v3::ApiV3,
        environment::{TestEnvironment, with_test_environment},
    };

    use super::*;

    #[actix_rt::test]
    async fn test_ping_server_success() {
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let queue = ServerPingQueue::new(
                env.db.pool,
                env.db.redis_pool,
                crate::clickhouse::init_client().await.unwrap(),
            );

            let _status =
                queue.ping_server("mc.hypixel.net", 25565).await.unwrap();
        })
        .await;
    }

    #[actix_rt::test]
    async fn test_ping_server_invalid_address() {
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let queue = ServerPingQueue::new(
                env.db.pool,
                env.db.redis_pool,
                crate::clickhouse::init_client().await.unwrap(),
            );

            _ = queue
                .ping_server("invalid.invalid", 25565)
                .await
                .unwrap_err();
        })
        .await;
    }
}
