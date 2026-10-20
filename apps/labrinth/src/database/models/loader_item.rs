use super::ids::LoaderId;
use eyre::{Result, WrapErr};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

const LOADER_ID: &str = "loader_id:v4";
const LOADERS_LIST_NAMESPACE: &str = "loaders:v4";

#[derive(Clone, Serialize, Deserialize)]
pub struct LoaderMetadata {
    pub platform: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Loader {
    pub id: LoaderId,
    pub loader: String,
    pub icon: String,
    pub supported_project_types: Vec<String>,
    pub supported_games: Vec<String>, // slugs
    pub metadata: LoaderMetadata,
}

impl Loader {
    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<LoaderId>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        {
            let mut redis = redis
                .connect()
                .await
                .wrap_err("connecting to redis for cached loader id")?;
            let key = redis.key().metadata(LOADER_ID, name);
            let cached_id: Option<i32> = redis
                .get_deserialized(&key)
                .await
                .wrap_err("fetching cached loader id")?;
            if let Some(cached_id) = cached_id {
                return Ok(Some(LoaderId(cached_id)));
            }
        }

        let result = sqlx::query!(
            "
            SELECT id FROM loaders
            WHERE loader = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await
        .wrap_err("fetching loader id")?
        .map(|r| LoaderId(r.id));

        if let Some(result) = result {
            let mut redis = redis
                .connect()
                .await
                .wrap_err("connecting to redis to cache loader id")?;
            let key = redis.key().metadata(LOADER_ID, name);
            redis
                .set_serialized(&key, &result.0, None)
                .await
                .wrap_err("caching loader id")?;
        }

        Ok(result)
    }

    pub async fn list<'a, E>(exec: E, redis: &RedisPool) -> Result<Vec<Loader>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        {
            let mut redis = redis
                .connect()
                .await
                .wrap_err("connecting to redis for cached loaders")?;
            let key = redis.key().metadata(LOADERS_LIST_NAMESPACE, "all");
            let cached_loaders: Option<Vec<Loader>> = redis
                .get_deserialized(&key)
                .await
                .wrap_err("fetching cached loaders")?;
            if let Some(cached_loaders) = cached_loaders {
                return Ok(cached_loaders);
            }
        }

        let result = sqlx::query!(
            "
            SELECT l.id id, l.loader loader, l.icon icon,
            (l.metadata->>'platform')::boolean AS platform,
            ARRAY_AGG(DISTINCT pt.name) filter (where pt.name is not null) project_types,
            ARRAY_AGG(DISTINCT g.slug) filter (where g.slug is not null) games
            FROM loaders l
            LEFT OUTER JOIN loaders_project_types lpt ON joining_loader_id = l.id
            LEFT OUTER JOIN project_types pt ON lpt.joining_project_type_id = pt.id
            LEFT OUTER JOIN loaders_project_types_games lptg ON lptg.loader_id = lpt.joining_loader_id AND lptg.project_type_id = lpt.joining_project_type_id
            LEFT OUTER JOIN games g ON lptg.game_id = g.id
            GROUP BY l.id;
            ",
        )
        .fetch(exec)
        .map_ok(|x| Loader {
            id: LoaderId(x.id),
            loader: x.loader,
            icon: x.icon,
            supported_project_types: x
                .project_types
                .unwrap_or_default()
                .iter()
                .map(|x| x.to_string())
                .collect(),
            supported_games: x
                .games
                .unwrap_or_default(),
            metadata: LoaderMetadata {
                platform: x.platform,
            },
        })
        .try_collect::<Vec<_>>()
        .await
        .wrap_err("fetching loaders")?;

        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis to cache loaders")?;
        let key = redis.key().metadata(LOADERS_LIST_NAMESPACE, "all");

        redis
            .set_serialized(&key, &result, None)
            .await
            .wrap_err("caching loaders")?;

        Ok(result)
    }
}
