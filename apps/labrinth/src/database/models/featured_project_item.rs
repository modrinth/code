use crate::database::redis::RedisPool;
use crate::models::ids::{ModId, UserId};
use crate::database::PgPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBFeaturedProject {
	pub user_id: UserId,
	pub mod_id: ModId,
	pub created: DateTime<Utc>,
}

impl DBFeaturedProject {
	pub async fn get_user_featured(
		user_id: UserId,
		pool: &PgPool,
		redis: &RedisPool,
	) -> crate::database::Result<Vec<ModId>> {
		const FEATURED_PROJECTS_QUERY: &str = r#"
			SELECT mod_id FROM featured_projects
			WHERE user_id = $1
			ORDER BY created DESC
		"#;

		let key = format!("featured_projects:{}", user_id);

		let result = redis
			.get_json::<Vec<ModId>>(&key)
			.await
			.unwrap_or_default();

		if !result.is_empty() {
			return Ok(result);
		}

		let row = sqlx::query(FEATURED_PROJECTS_QUERY)
			.bind(user_id.0)
			.fetch_all(pool)
			.await?;

		let featured_projects: Vec<ModId> = row
			.iter()
			.map(|r| ModId(r.get::<i64, _>("mod_id")))
			.collect();

		let _ = redis.set_json_with_expiry(&key, &featured_projects, 3600).await;

		Ok(featured_projects)
	}

	pub async fn toggle_featured(
		user_id: UserId,
		mod_id: ModId,
		pool: &PgPool,
		redis: &RedisPool,
	) -> crate::database::Result<bool> {
		const CHECK_QUERY: &str = r#"
			SELECT 1 FROM featured_projects
			WHERE user_id = $1 AND mod_id = $2
		"#;

		const INSERT_QUERY: &str = r#"
			INSERT INTO featured_projects (user_id, mod_id)
			VALUES ($1, $2)
			ON CONFLICT DO NOTHING
		"#;

		const DELETE_QUERY: &str = r#"
			DELETE FROM featured_projects
			WHERE user_id = $1 AND mod_id = $2
		"#;

		let exists = sqlx::query(CHECK_QUERY)
			.bind(user_id.0)
			.bind(mod_id.0)
			.fetch_optional(pool)
			.await?
			.is_some();

		if exists {
			sqlx::query(DELETE_QUERY)
				.bind(user_id.0)
				.bind(mod_id.0)
				.execute(pool)
				.await?;
		} else {
			sqlx::query(INSERT_QUERY)
				.bind(user_id.0)
				.bind(mod_id.0)
				.execute(pool)
				.await?;
		}

		let key = format!("featured_projects:{}", user_id);
		let _ = redis.delete_key(&key).await;

		Ok(!exists)
	}

	pub async fn is_featured(
		user_id: UserId,
		mod_id: ModId,
		pool: &PgPool,
		redis: &RedisPool,
	) -> crate::database::Result<bool> {
		const QUERY: &str = r#"
			SELECT 1 FROM featured_projects
			WHERE user_id = $1 AND mod_id = $2
		"#;

		let key = format!("featured_projects:{}:{}", user_id, mod_id);

		if let Ok(Some(cached)) = redis.get::<bool>(&key).await {
			return Ok(cached);
		}

		let result = sqlx::query(QUERY)
			.bind(user_id.0)
			.bind(mod_id.0)
			.fetch_optional(pool)
			.await?
			.is_some();

		let _ = redis.set_with_expiry(&key, &result.to_string(), 3600).await;

		Ok(result)
	}
}
