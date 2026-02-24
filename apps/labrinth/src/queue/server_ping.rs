use crate::database::DBProject;
use crate::database::models::DBProjectId;
use crate::database::redis::RedisPool;
use crate::env::ENV;
use crate::models::exp;
use crate::models::ids::ProjectId;
use crate::{database::PgPool, util::error::Context};
use async_minecraft_ping::ServerDescription;
use chrono::{TimeDelta, Utc};
use clickhouse::{Client, Row};
use serde::Serialize;
use sqlx::types::Json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{Instrument, debug, info, info_span, warn};

pub struct ServerPingQueue {
    pub db: PgPool,
    pub redis: RedisPool,
    pub clickhouse: Client,
}

pub const REDIS_NAMESPACE: &str = "minecraft_java_server_ping";
pub const CLICKHOUSE_TABLE: &str = "minecraft_java_server_pings";

impl ServerPingQueue {
    pub fn new(db: PgPool, redis: RedisPool, clickhouse: Client) -> Self {
        Self {
            db,
            redis,
            clickhouse,
        }
    }

    pub async fn ping_minecraft_java_servers(&self) -> eyre::Result<()> {
        let server_projects = self.find_servers_to_ping().await?;
        info!("Found {} servers to ping", server_projects.len());

        let active_pings =
            Arc::new(Semaphore::new(ENV.SERVER_PING_MAX_CONCURRENT));

        let pings = server_projects
            .into_iter()
            .filter_map(|(project_id, java_server)| {
                let address = java_server.address.to_string();
                if address.trim().is_empty() {
                    debug!("Skipping ping for server project {project_id} with empty address");
                    return None;
                }

                let port = java_server.port;
                let span = info_span!("ping", %project_id, %address, %port);

                let active_pings = active_pings.clone();
                let task = async move {
                    let _permit = active_pings.acquire().await.expect("semaphore should not be closed now");

                    let mut retries = ENV.SERVER_PING_RETRIES;
                    let result = loop {
                        match ping_server(&address, port).await {
                            Ok(ping) => {
                                debug!(?ping, "Received successful ping");
                                break Ok(ping);
                            }
                            Err(err) if retries == 0 => {
                                debug!("Failed to ping server in {:?}ms, no retries left: {err:#}", ENV.SERVER_PING_TIMEOUT_MS);
                                break Err(err);
                            }
                            Err(err) => {
                                debug!(%retries, "Failed to ping server in {:?}ms, retrying: {err:#}", ENV.SERVER_PING_TIMEOUT_MS);
                                retries -= 1;
                                continue;
                            }
                        };
                    };

                    (project_id, exp::minecraft::JavaServerPing {
                        when: Utc::now(),
                        address: address.to_string(),
                        port,
                        data: result.ok(),
                    })
                };
                Some(tokio::spawn(task.instrument(span)))
            })
            .collect::<JoinSet<_>>()
            .join_all()
            .await
            .into_iter()
            .filter_map(|result| result.ok())
            .collect::<Vec<_>>();

        if !pings.is_empty() {
            let mut ch = self
                .clickhouse
                .insert::<ServerPingRecord>(CLICKHOUSE_TABLE)
                .await
                .wrap_err("failed to begin inserting ping records")?;

            let mut redis = self
                .redis
                .connect()
                .await
                .wrap_err("failed to connect to redis")?;

            for (project_id, ping) in &pings {
                let data = ping.data.as_ref();

                let row = ServerPingRecord {
                    recorded: ping.when.timestamp_nanos_opt().unwrap()
                        / 100_000,
                    project_id: project_id.0,
                    address: ping.address.clone(),
                    port: ping.port,
                    latency_ms: data.map(|d| d.latency.as_millis() as u32),
                    description: data.map(|d| d.description.clone()),
                    version_name: data.map(|d| d.version_name.clone()),
                    version_protocol: data.map(|d| d.version_protocol),
                    players_online: data.map(|d| d.players_online),
                    players_max: data.map(|d| d.players_max),
                };

                ch.write(&row)
                    .await
                    .wrap_err("failed to write ping record")?;

                redis
                    .set_serialized_to_json(
                        REDIS_NAMESPACE,
                        project_id,
                        ping,
                        None,
                    )
                    .await
                    .wrap_err("failed to set redis key")?;

                DBProject::clear_cache(
                    (*project_id).into(),
                    None,
                    None,
                    &self.redis,
                )
                .await
                .inspect_err(|err| {
                    warn!("failed to clear project cache: {err:#}")
                })
                .ok();
            }

            ch.end()
                .await
                .wrap_err("failed to end inserting ping records")?;
        }

        let num_success =
            pings.iter().filter(|(_, ping)| ping.data.is_some()).count();
        let num_total = pings.len();

        info!(
            "Inserted ping results for {} servers - {num_success}/{num_total} successful",
            pings.len()
        );
        Ok(())
    }

    async fn find_servers_to_ping(
        &self,
    ) -> eyre::Result<Vec<(ProjectId, exp::minecraft::JavaServerProject)>> {
        // first select all java servers
        let all_server_projects = sqlx::query!(
            r#"
            SELECT id, components AS "components: Json<exp::ProjectSerial>"
            FROM mods
            WHERE components ? 'minecraft_java_server'
            "#
        )
        .fetch_all(&self.db)
        .await
        .wrap_err("failed to fetch servers to ping")?;

        if all_server_projects.is_empty() {
            // we must early-exit, otherwise we'll run `redis.get_many()`,
            // which runs `MGET` with no args; this gives:
            // "ResponseError: wrong number of arguments for 'mget' command"
            return Ok(Vec::new());
        }

        let mut redis = self
            .redis
            .connect()
            .await
            .wrap_err("failed to connect to redis")?;

        // get the last ping info for all of them
        // querying redis here, which is a cache, not the source of truth (clickhouse),
        // but it should be fine since we don't usually flush redis
        // and if we do miss an entry that we shouldn't, we just ping it again
        let all_project_ids = all_server_projects
            .iter()
            .map(|row| ProjectId::from(DBProjectId(row.id)).to_string())
            .collect::<Vec<_>>();

        let all_server_last_pings = redis
            .get_many_deserialized_from_json::<exp::minecraft::JavaServerPing>(
                REDIS_NAMESPACE,
                &all_project_ids,
            )
            .await
            .wrap_err("failed to fetch server project last pings")?;

        let now = Utc::now();
        let projects_to_ping = all_server_projects
            .into_iter()
            .zip(all_server_last_pings)
            // only include projects which:
            // - have not had a ping in redis yet
            // - OR their last ping was a failure
            // - OR their last successful ping was more than `SERVER_PING_MIN_INTERVAL_SEC` seconds ago
            .filter(|(_, ping)| {
                let Some(ping) = ping else { return true };
                if ping.data.is_none() {
                    return true;
                };
                ping.when.signed_duration_since(now)
                    > TimeDelta::seconds(
                        ENV.SERVER_PING_MIN_INTERVAL_SEC as i64,
                    )
            })
            .filter_map(|(row, _)| {
                let server = row.components.0.minecraft_java_server?;
                Some((ProjectId::from(DBProjectId(row.id)), server))
            })
            .collect::<Vec<_>>();

        Ok(projects_to_ping)
    }
}

async fn ping_server(
    address: &str,
    port: u16,
) -> eyre::Result<exp::minecraft::JavaServerPingData> {
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

        Ok(exp::minecraft::JavaServerPingData {
            latency: start.elapsed(),
            version_name: status.version.name,
            version_protocol: status.version.protocol,
            description: match status.description {
                ServerDescription::Plain(text)
                | ServerDescription::Object { text } => text,
            },
            players_online: status.players.online,
            players_max: status.players.max,
        })
    };

    let timeout = Duration::from_millis(ENV.SERVER_PING_TIMEOUT_MS);
    tokio::time::timeout(timeout, task)
        .await
        .wrap_err("server ping timed out")
        .flatten()
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
    use super::*;

    #[actix_rt::test]
    async fn test_ping_server_success() {
        let _status = ping_server("mc.hypixel.net", 25565).await.unwrap();
    }

    #[actix_rt::test]
    async fn test_ping_server_invalid_address() {
        _ = ping_server("invalid.invalid", 25565).await.unwrap_err();
    }
}
