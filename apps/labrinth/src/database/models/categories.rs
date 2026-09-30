use std::collections::HashMap;

use eyre::{Result, WrapErr};
use futures::TryStreamExt;
use xredis::RedisPool;

use super::ids::*;
use serde::{Deserialize, Serialize};

const TAGS_NAMESPACE: &str = "tags:v4";

pub struct ProjectType {
    pub id: ProjectTypeId,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Category {
    pub id: CategoryId,
    pub category: String,
    pub project_type: String,
    pub icon: String,
    pub header: String,
}

pub struct ReportType {
    pub id: ReportTypeId,
    pub report_type: String,
}

impl Category {
    // Gets hashmap of category ids matching a name
    // Multiple categories can have the same name, but different project types, so we need to return a hashmap
    // ProjectTypeId -> CategoryId
    pub async fn get_ids<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<HashMap<ProjectTypeId, CategoryId>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id, project_type FROM categories
            WHERE category = $1
            ",
            name,
        )
        .fetch_all(exec)
        .await
        .wrap_err("fetching category ids")?;

        let mut map = HashMap::new();
        for r in result {
            map.insert(ProjectTypeId(r.project_type), CategoryId(r.id));
        }

        Ok(map)
    }

    pub async fn get_id_project<'a, E>(
        name: &str,
        project_type: ProjectTypeId,
        exec: E,
    ) -> Result<Option<CategoryId>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM categories
            WHERE category = $1 AND project_type = $2
            ",
            name,
            project_type as ProjectTypeId
        )
        .fetch_optional(exec)
        .await
        .wrap_err("fetching category id")?;

        Ok(result.map(|r| CategoryId(r.id)))
    }

    pub async fn list<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Category>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        {
            let mut redis = redis
                .connect()
                .await
                .wrap_err("connecting to redis for cached categories")?;
            let key = redis.key().metadata(TAGS_NAMESPACE, "category");

            let res: Option<Vec<Category>> = redis
                .get_deserialized(&key)
                .await
                .wrap_err("fetching cached categories")?;

            if let Some(res) = res {
                return Ok(res);
            }
        }

        let result = sqlx::query!(
            "
            SELECT c.id id, c.category category, c.icon icon, c.header category_header, pt.name project_type
            FROM categories c
            INNER JOIN project_types pt ON c.project_type = pt.id
            ORDER BY c.ordering, c.category
            "
        )
        .fetch(exec)
        .map_ok(|c| Category {
            id: CategoryId(c.id),
            category: c.category,
            project_type: c.project_type,
            icon: c.icon,
            header: c.category_header
        })
        .try_collect::<Vec<Category>>()
        .await
        .wrap_err("fetching categories")?;

        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis to cache categories")?;
        let key = redis.key().metadata(TAGS_NAMESPACE, "category");

        redis
            .set_serialized(&key, &result, None)
            .await
            .wrap_err("caching categories")?;

        Ok(result)
    }
}

impl ReportType {
    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<ReportTypeId>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM report_types
            WHERE name = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await
        .wrap_err("fetching report type id")?;

        Ok(result.map(|r| ReportTypeId(r.id)))
    }

    pub async fn list<'a, E>(exec: E, redis: &RedisPool) -> Result<Vec<String>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        {
            let mut redis = redis
                .connect()
                .await
                .wrap_err("connecting to redis for cached report types")?;
            let key = redis.key().metadata(TAGS_NAMESPACE, "report_type");

            let res: Option<Vec<String>> =
                redis
                    .get_deserialized(&key)
                    .await
                    .wrap_err("fetching cached report types")?;

            if let Some(res) = res {
                return Ok(res);
            }
        }

        let result = sqlx::query!(
            "
            SELECT name FROM report_types
            "
        )
        .fetch(exec)
        .map_ok(|c| c.name)
        .try_collect::<Vec<String>>()
        .await
        .wrap_err("fetching report types")?;

        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis to cache report types")?;
        let key = redis.key().metadata(TAGS_NAMESPACE, "report_type");

        redis
            .set_serialized(&key, &result, None)
            .await
            .wrap_err("caching report types")?;

        Ok(result)
    }
}

impl ProjectType {
    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<ProjectTypeId>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM project_types
            WHERE name = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await
        .wrap_err("fetching project type id")?;

        Ok(result.map(|r| ProjectTypeId(r.id)))
    }

    pub async fn list<'a, E>(exec: E, redis: &RedisPool) -> Result<Vec<String>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        {
            let mut redis = redis
                .connect()
                .await
                .wrap_err("connecting to redis for cached project types")?;
            let key = redis.key().metadata(TAGS_NAMESPACE, "project_type");

            let res: Option<Vec<String>> =
                redis
                    .get_deserialized(&key)
                    .await
                    .wrap_err("fetching cached project types")?;

            if let Some(res) = res {
                return Ok(res);
            }
        }

        let result = sqlx::query!(
            "
            SELECT name FROM project_types
            "
        )
        .fetch(exec)
        .map_ok(|c| c.name)
        .try_collect::<Vec<String>>()
        .await
        .wrap_err("fetching project types")?;

        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis to cache project types")?;
        let key = redis.key().metadata(TAGS_NAMESPACE, "project_type");

        redis
            .set_serialized(&key, &result, None)
            .await
            .wrap_err("caching project types")?;

        Ok(result)
    }
}
