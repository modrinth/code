use std::collections::HashMap;

use const_format::formatcp;
use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info};

use crate::{
    database::{DBProject, redis::RedisPool},
    models::ids::ProjectId,
    routes::analytics::MINECRAFT_SERVER_PLAYS,
    util::error::Context,
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
    db: &PgPool,
    redis_pool: &RedisPool,
    clickhouse: &clickhouse::Client,
) -> Result<()> {
    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct Row {
        project_id: u64,
        plays_2w: u64,
        plays_4w: u64,
    }

    // for each project..
    // - count last 2 weeks and 4 weeks of play rows
    // for each play row...
    // - build a per-actor key per project id
    //   - user_id` when `user_id != 0`
    //   - otherwise `minecraft_uuid
    // - keep only rows where the rolling 24h count per actor is <= 3
    let rows = clickhouse
        .query(formatcp!(
            "
            SELECT
                project_id,
                countIf(
                    recorded BETWEEN now64(4) - INTERVAL 2 WEEK AND now64(4)
                ) AS plays_2w,
                countIf(
                    recorded BETWEEN now64(4) - INTERVAL 4 WEEK AND now64(4)
                ) AS plays_4w
            FROM (
                SELECT
                    project_id,
                    recorded,
                    count() OVER (
                        PARTITION BY
                            project_id,
                            if(
                                user_id != 0,
                                concat('u:', toString(user_id)),
                                concat('m:', toString(minecraft_uuid))
                            )
                        ORDER BY toUnixTimestamp64Milli(recorded)
                        RANGE BETWEEN 86400000 PRECEDING AND CURRENT ROW
                    ) AS plays_per_actor_24h
                FROM {MINECRAFT_SERVER_PLAYS}
            )
            WHERE plays_per_actor_24h <= 3
            GROUP BY project_id
            "
        ))
        .fetch_all::<Row>()
        .await
        .wrap_err("failed to create cursor for total server plays")?;

    info!(
        "Caching Minecraft server analytics for {} projects",
        rows.len()
    );

    let project_slugs = sqlx::query!(
        "
        SELECT id, slug FROM mods
        WHERE id = ANY($1)
        ",
        &rows
            .iter()
            .map(|row| row.project_id.cast_signed())
            .collect::<Vec<_>>(),
    )
    .fetch_all(db)
    .await
    .wrap_internal_err("failed to get slugs for projects to cache analytics")?
    .into_iter()
    .filter_map(|row| {
        row.slug
            .map(|slug| (ProjectId(row.id.cast_unsigned()), slug))
    })
    .collect::<HashMap<_, _>>();

    let mut redis = redis_pool
        .connect()
        .await
        .wrap_err("failed to connect to redis")?;

    for row in rows {
        let project_id = ProjectId(row.project_id);
        let analytics = MinecraftServerAnalytics {
            verified_plays_2w: row.plays_2w,
            verified_plays_4w: row.plays_4w,
        };

        debug!("Caching analytics for {project_id}: {analytics:?}");
        redis
            .set_serialized_to_json(
                MINECRAFT_SERVER_ANALYTICS,
                project_id.to_string(),
                analytics,
                None,
            )
            .await
            .wrap_err_with(|| {
                eyre!("failed to set analytics for project '{project_id}'")
            })?;

        DBProject::clear_cache(
            project_id.into(),
            project_slugs.get(&project_id).cloned(),
            None,
            redis_pool,
        )
        .await
        .wrap_err_with(|| {
            eyre!("failed to clear cache for project '{project_id}'")
        })?;
    }

    info!("Cached Minecraft server analytics");
    Ok(())
}
