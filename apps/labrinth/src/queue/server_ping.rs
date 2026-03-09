use crate::database::DBProject;
use crate::database::models::DBProjectId;
use crate::database::redis::RedisPool;
use crate::env::ENV;
use crate::models::exp;
use crate::models::ids::ProjectId;
use crate::models::projects::ProjectStatus;
use crate::{database::PgPool, util::error::Context};
use chrono::{TimeDelta, Utc};
use clickhouse::{Client, Row};
use serde::Serialize;
use sqlx::types::Json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{Instrument, info, info_span, trace, warn};

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
            .map(|(project_id, java_server)| {
                let span = info_span!("ping", %project_id, address = %java_server.address);

                let active_pings = active_pings.clone();
                let address = java_server.address;
                let task = async move {
                    let _permit = active_pings.acquire().await.expect("semaphore should not be closed now");

                    let mut retries = ENV.SERVER_PING_RETRIES;
                    let result = loop {
                        match ping_server(&address, None).await {
                            Ok(ping) => {
                                info!(?ping, "Received successful ping");
                                break Ok(ping);
                            }
                            Err(err) if retries == 0 => {
                                info!("Failed to ping server in {:?}ms, no retries left: {err:#}", ENV.SERVER_PING_TIMEOUT_MS);
                                break Err(err);
                            }
                            Err(err) => {
                                trace!(%retries, "Failed to ping server in {:?}ms, retrying: {err:#}", ENV.SERVER_PING_TIMEOUT_MS);
                                retries -= 1;
                                continue;
                            }
                        };
                    };

                    (project_id, exp::minecraft::JavaServerPing {
                        when: Utc::now(),
                        address,
                        data: result.ok(),
                    })
                };
                tokio::spawn(task.instrument(span))
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
            WHERE
                status = ANY($1)
                AND components ? 'minecraft_java_server'
            "#,
            &ProjectStatus::iterator()
                .filter(|s| s.is_approved())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
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
            // only include projects which have and address AND:
            // - have not had a ping in redis yet
            // - OR their last ping was a failure
            // - OR their last successful ping was more than `SERVER_PING_MIN_INTERVAL_SEC` seconds ago
            .filter(|(row, ping)| {
                if row
                    .components
                    .0
                    .minecraft_java_server
                    .as_ref()
                    .is_none_or(|p| p.address.trim().is_empty())
                {
                    return false;
                }

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

pub async fn ping_server(
    address: &str,
    timeout: Option<Duration>,
) -> eyre::Result<exp::minecraft::JavaServerPingData> {
    let start = Instant::now();
    let default_duration = Duration::from_millis(ENV.SERVER_PING_TIMEOUT_MS);
    let timeout = timeout
        .map(|duration| duration.min(default_duration))
        .unwrap_or(default_duration);

    let (address, port) = match address.rsplit_once(':') {
        Some((addr, port)) => {
            let port = port.parse::<u16>().wrap_err("invalid port number")?;
            (addr, port)
        }
        None => (address, 25565),
    };

    let task = async move {
        let conn = async_minecraft_ping::ConnectionConfig::build(address)
            .with_port(port)
            .with_srv_lookup()
            .connect()
            .await
            .wrap_err("failed to connect to server")?;

        let status = conn
            .status()
            .await
            .wrap_err("failed to get server status")?
            .status;

        eyre::Ok(exp::minecraft::JavaServerPingData {
            latency: start.elapsed(),
            version_name: status.version.name,
            version_protocol: status.version.protocol,
            description: status.description,
            players_online: status.players.online,
            players_max: status.players.max,
        })
    };

    tokio::time::timeout(timeout, task)
        .await
        .map_err(eyre::Error::new)
        .flatten()
}

#[derive(Debug, Row, Serialize, Clone)]
struct ServerPingRecord {
    recorded: i64,
    project_id: u64,
    address: String,
    latency_ms: Option<u32>,
    description: Option<serde_json::Value>,
    version_name: Option<String>,
    version_protocol: Option<i32>,
    players_online: Option<i32>,
    players_max: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_ping_server_success() {
        let _status = ping_server("mc.hypixel.net", None).await.unwrap();
    }

    #[actix_rt::test]
    async fn test_follow_srv_record() {
        _ = ping_server("hypixel.net", None).await.unwrap();
    }

    #[actix_rt::test]
    async fn test_ping_server_invalid_address() {
        _ = ping_server("invalid.invalid", None).await.unwrap_err();
    }

    #[actix_rt::test]
    async fn test_ping_zero_timeout() {
        _ = ping_server("mc.hypixel.net", Some(Duration::ZERO))
            .await
            .unwrap_err();
    }
}
