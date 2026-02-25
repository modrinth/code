use const_format::formatcp;
use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

use crate::{
    database::redis::RedisPool, models::ids::ProjectId,
    routes::analytics::MINECRAFT_SERVER_PLAYS, util::error::Context,
};

pub const MINECRAFT_SERVER_ANALYTICS: &str = "minecraft_server_analytics";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftServerAnalytics {
    pub verified_plays_2w: u64,
    pub verified_plays_4w: u64,
}

/// Queries server project analytics (e.g. number of verified plays in last
/// 2 weeks for server projects) and caches them in Redis.
pub async fn cache_analytics(
    redis: &RedisPool,
    clickhouse: &clickhouse::Client,
) -> Result<()> {
    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct Row {
        project_id: u64,
        plays_2w: u64,
        plays_4w: u64,
    }

    let mut rows = clickhouse
        .query(formatcp!(
            "
            SELECT
          		project_id,
          		countIf(
         			recorded BETWEEN toUInt64(toUnixTimestamp(now() - INTERVAL 2 WEEK))
        			             AND toUInt64(toUnixTimestamp(now()))
          		) AS plays_2w,
          		countIf(
         			recorded BETWEEN toUInt64(toUnixTimestamp(now() - INTERVAL 4 WEEK))
        			             AND toUInt64(toUnixTimestamp(now()))
          		) AS plays_4w
           	FROM {MINECRAFT_SERVER_PLAYS}
           	GROUP BY project_id
            "
        ))
        .fetch::<Row>()
        .wrap_err("failed to create cursor for total server plays")?;

    let mut redis = redis
        .connect()
        .await
        .wrap_err("failed to connect to redis")?;

    while let Some(row) = rows
        .next()
        .await
        .wrap_err("failed to query total server plays")?
    {
        let project_id = ProjectId(row.project_id);
        redis
            .set_serialized_to_json(
                MINECRAFT_SERVER_ANALYTICS,
                project_id.to_string(),
                MinecraftServerAnalytics {
                    verified_plays_2w: row.plays_2w,
                    verified_plays_4w: row.plays_4w,
                },
                None,
            )
            .await
            .wrap_err_with(|| {
                eyre!("failed to set analytics for project '{project_id}'")
            })?;
    }

    Ok(())
}
