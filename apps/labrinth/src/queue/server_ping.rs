use crate::database::redis::RedisPool;
use crate::models::exp;
use crate::{database::PgPool, util::error::Context};
use chrono::Utc;
use clickhouse::{Client, Row};
use futures::TryStreamExt;
use serde::Serialize;
use sqlx::types::Json;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
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
                exp::component::Component::from_db(java_server);
            let address = &java_server.address;
            let port = java_server.port;

            let recorded = Utc::now().timestamp_millis();
            let ping_record = match self.ping_server(address, port).await {
                Ok(record) => ServerPingRecord {
                    recorded,
                    project_id,
                    address: address.clone(),
                    port,
                    online: true,
                    latency_ms: Some(record.latency.as_millis() as u32),
                },
                Err(err) => {
                    info!("Failed to ping {address}:{port}: {err:?}");
                    ServerPingRecord {
                        recorded,
                        project_id,
                        address: address.clone(),
                        port,
                        online: false,
                        latency_ms: None,
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
    ) -> eyre::Result<PingRecord> {
        let start = Instant::now();

        let _stream = TcpStream::connect((address, port))
            .await
            .wrap_err("failed to connect to address and port")?;
        Ok(PingRecord {
            latency: start.elapsed(),
        })
    }
}

#[derive(Debug)]
struct PingRecord {
    latency: Duration,
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

            queue.ping_server("example.com", 80).await.unwrap();
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

            _ = queue.ping_server("invalid.invalid", 80).await.unwrap_err();
        })
        .await;
    }
}
