use std::collections::HashMap;

use crate::database::redis::RedisPool;

use super::DatabaseError;
use super::ids::*;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

const TAGS_NAMESPACE: &str = "tags";

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

#[derive(Serialize, Deserialize)]
pub struct LinkPlatform {
    pub id: LinkPlatformId,
    pub name: String,
    pub donation: bool,
}

impl Category {
    // Gets hashmap of category ids matching a name
    // Multiple categories can have the same name, but different project types, so we need to return a hashmap
    // ProjectTypeId -> CategoryId
    pub async fn get_ids<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<HashMap<ProjectTypeId, CategoryId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id, project_type FROM categories
            WHERE category = $1
            ",
            name,
        )
        .fetch_all(exec)
        .await?;

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
    ) -> Result<Option<CategoryId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
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
        .await?;

        Ok(result.map(|r| CategoryId(r.id)))
    }

    pub async fn list<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Category>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;

        let res: Option<Vec<Category>> = redis
            .get_deserialized_from_json(TAGS_NAMESPACE, "category")
            .await?;

        if let Some(res) = res {
            return Ok(res);
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
        .await?;

        redis
            .set_serialized_to_json(TAGS_NAMESPACE, "category", &result, None)
            .await?;

        Ok(result)
    }
}

impl LinkPlatform {
    pub async fn get_id<'a, E>(
        id: &str,
        exec: E,
    ) -> Result<Option<LinkPlatformId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM link_platforms
            WHERE name = $1
            ",
            id
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| LinkPlatformId(r.id)))
    }

    pub async fn list<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<LinkPlatform>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;

        let res: Option<Vec<LinkPlatform>> = redis
            .get_deserialized_from_json(TAGS_NAMESPACE, "link_platform")
            .await?;

        if let Some(res) = res {
            return Ok(res);
        }

        let result = sqlx::query!(
            "
            SELECT id, name, donation FROM link_platforms
            "
        )
        .fetch(exec)
        .map_ok(|c| LinkPlatform {
            id: LinkPlatformId(c.id),
            name: c.name,
            donation: c.donation,
        })
        .try_collect::<Vec<LinkPlatform>>()
        .await?;

        redis
            .set_serialized_to_json(
                TAGS_NAMESPACE,
                "link_platform",
                &result,
                None,
            )
            .await?;

        Ok(result)
    }
}

impl ReportType {
    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<ReportTypeId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM report_types
            WHERE name = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| ReportTypeId(r.id)))
    }

    pub async fn list<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<String>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;

        let res: Option<Vec<String>> = redis
            .get_deserialized_from_json(TAGS_NAMESPACE, "report_type")
            .await?;

        if let Some(res) = res {
            return Ok(res);
        }

        let result = sqlx::query!(
            "
            SELECT name FROM report_types
            "
        )
        .fetch(exec)
        .map_ok(|c| c.name)
        .try_collect::<Vec<String>>()
        .await?;

        redis
            .set_serialized_to_json(
                TAGS_NAMESPACE,
                "report_type",
                &result,
                None,
            )
            .await?;

        Ok(result)
    }
}

impl ProjectType {
    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<ProjectTypeId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM project_types
            WHERE name = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| ProjectTypeId(r.id)))
    }

    pub async fn list<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<String>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;

        let res: Option<Vec<String>> = redis
            .get_deserialized_from_json(TAGS_NAMESPACE, "project_type")
            .await?;

        if let Some(res) = res {
            return Ok(res);
        }

        let result = sqlx::query!(
            "
            SELECT name FROM project_types
            "
        )
        .fetch(exec)
        .map_ok(|c| c.name)
        .try_collect::<Vec<String>>()
        .await?;

        redis
            .set_serialized_to_json(
                TAGS_NAMESPACE,
                "project_type",
                &result,
                None,
            )
            .await?;

        Ok(result)
    }
}
