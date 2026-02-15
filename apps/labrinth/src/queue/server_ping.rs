use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::models::exp;
use chrono::Utc;
use clickhouse::{Client, Row};
use futures::TryStreamExt;
use serde::Serialize;
use sqlx::types::Json;
use std::time::Instant;
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

                let ping_result = self.ping_server(address, port).await;
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

    async fn ping_server(
        &self,
        address: &str,
        port: u16,
    ) -> Option<std::time::Duration> {
        let start = Instant::now();

        match TcpStream::connect((address, port)).await {
            Ok(_stream) => Some(start.elapsed()),
            Err(_) => None,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_ping_server_success() {
        let mock_pg = sqlx::PgPool::connect_lazy("postgresql://localhost/test")
            .map(PgPool::from)
            .unwrap();
        let mock_redis = RedisPool::new("test_server_ping");
        let mock_clickhouse = Client::default();

        let queue = ServerPingQueue::new(mock_pg, mock_redis, mock_clickhouse);

        let result = queue.ping_server("example.com", 80).await;

        assert!(
            result.is_some(),
            "Connection to example.com:80 should succeed"
        );
        assert!(
            result.unwrap().as_millis() > 0,
            "Latency should be positive"
        );
    }

    #[actix_rt::test]
    async fn test_ping_server_invalid_address() {
        let mock_pg = sqlx::PgPool::connect_lazy("postgresql://localhost/test")
            .map(PgPool::from)
            .unwrap();
        let mock_redis = RedisPool::new("test_server_ping");
        let mock_clickhouse = Client::default();

        let queue = ServerPingQueue::new(mock_pg, mock_redis, mock_clickhouse);

        let result = queue
            .ping_server("this-domain-does-not-exist.invalid", 80)
            .await;

        assert!(
            result.is_none(),
            "Connection to invalid address should fail"
        );
    }

    #[actix_rt::test]
    async fn test_ping_server_timeout() {
        let mock_pg = sqlx::PgPool::connect_lazy("postgresql://localhost/test")
            .map(PgPool::from)
            .unwrap();
        let mock_redis = RedisPool::new("test_server_ping");
        let mock_clickhouse = Client::default();

        let queue = ServerPingQueue::new(mock_pg, mock_redis, mock_clickhouse);

        let result = queue.ping_server("192.0.2.1", 9999).await;

        assert!(result.is_none(), "Connection timeout should return None");
    }

    #[test]
    fn test_server_ping_record_serialization() {
        let record = ServerPingRecord {
            recorded: Utc::now().timestamp_millis(),
            project_id: 12345,
            address: "example.com".to_string(),
            port: 80,
            online: true,
            latency_ms: Some(42),
        };

        let json = serde_json::to_string(&record);
        assert!(json.is_ok(), "ServerPingRecord should serialize to JSON");
    }

    #[test]
    fn test_server_ping_queue_new() {
        let mock_pg = sqlx::PgPool::connect_lazy("postgresql://localhost/test")
            .map(PgPool::from)
            .unwrap();
        let mock_redis = RedisPool::new("test_server_ping");
        let mock_clickhouse = Client::default();

        let _queue = ServerPingQueue::new(mock_pg, mock_redis, mock_clickhouse);

        let _ = _queue.pg;
        let _ = _queue.redis;
        let _ = _queue.clickhouse;
    }
}
