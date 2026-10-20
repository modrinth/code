use super::ids::GameId;
use eyre::{Result, WrapErr};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

const GAMES_LIST_NAMESPACE: &str = "games:v4";
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Game {
    pub id: GameId,
    pub slug: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
}

impl Game {
    pub async fn get_slug<'a, E>(
        slug: &str,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<Game>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Self::list(exec, redis)
            .await
            .wrap_err("listing games")?
            .into_iter()
            .find(|x| x.slug == slug))
    }

    pub async fn list<'a, E>(exec: E, redis: &RedisPool) -> Result<Vec<Game>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        {
            let mut redis = redis
                .connect()
                .await
                .wrap_err("connecting to redis for cached games")?;
            let key = redis.key().metadata(GAMES_LIST_NAMESPACE, "games");
            let cached_games: Option<Vec<Game>> = redis
                .get_deserialized(&key)
                .await
                .wrap_err("fetching cached games")?;
            if let Some(cached_games) = cached_games {
                return Ok(cached_games);
            }
        }

        let result = sqlx::query!(
            "
            SELECT id, slug, name, icon_url, banner_url FROM games
            ",
        )
        .fetch(exec)
        .map_ok(|x| Game {
            id: GameId(x.id),
            slug: x.slug,
            name: x.name,
            icon_url: x.icon_url,
            banner_url: x.banner_url,
        })
        .try_collect::<Vec<Game>>()
        .await
        .wrap_err("fetching games")?;

        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis to cache games")?;
        let key = redis.key().metadata(GAMES_LIST_NAMESPACE, "games");

        redis
            .set_serialized(&key, &result, None)
            .await
            .wrap_err("caching games")?;

        Ok(result)
    }
}
