use crate::database::redis::RedisPool;
use ariadne::ids::base62_impl::parse_base62;
use dashmap::DashMap;
use futures::TryStreamExt;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use super::{DBTeamMember, ids::*};
use serde::{Deserialize, Serialize};

const ORGANIZATIONS_NAMESPACE: &str = "organizations";
const ORGANIZATIONS_TITLES_NAMESPACE: &str = "organizations_titles";

#[derive(Deserialize, Serialize, Clone, Debug)]
/// An organization of users who together control one or more projects and organizations.
pub struct DBOrganization {
    /// The id of the organization
    pub id: DBOrganizationId,

    /// The slug of the organization
    pub slug: String,

    /// The title of the organization
    pub name: String,

    /// The associated team of the organization
    pub team_id: DBTeamId,

    /// The description of the organization
    pub description: String,

    /// The display icon for the organization
    pub icon_url: Option<String>,
    pub raw_icon_url: Option<String>,
    pub color: Option<u32>,
}

impl DBOrganization {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), super::DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO organizations (id, slug, name, team_id, description, icon_url, raw_icon_url, color)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ",
            self.id.0,
            self.slug,
            self.name,
            self.team_id as DBTeamId,
            self.description,
            self.icon_url,
            self.raw_icon_url,
            self.color.map(|x| x as i32),
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get<'a, E>(
        string: &str,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Self::get_many(&[string], exec, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_id<'a, 'b, E>(
        id: DBOrganizationId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Self::get_many_ids(&[id], exec, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many_ids<'a, 'b, E>(
        organization_ids: &[DBOrganizationId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let ids = organization_ids
            .iter()
            .map(|x| crate::models::ids::OrganizationId::from(*x))
            .collect::<Vec<_>>();
        Self::get_many(&ids, exec, redis).await
    }

    pub async fn get_many<
        'a,
        E,
        T: Display + Hash + Eq + PartialEq + Clone + Debug,
    >(
        organization_strings: &[T],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let val = redis
            .get_cached_keys_with_slug(
                ORGANIZATIONS_NAMESPACE,
                ORGANIZATIONS_TITLES_NAMESPACE,
                false,
                organization_strings,
                |ids| async move {
                    let org_ids: Vec<i64> = ids
                        .iter()
                        .flat_map(|x| parse_base62(&x.to_string()).ok())
                        .map(|x| x as i64)
                        .collect();
                    let slugs = ids
                        .into_iter()
                        .map(|x| x.to_string().to_lowercase())
                        .collect::<Vec<_>>();

                    let organizations = sqlx::query!(
                        "
                        SELECT o.id, o.slug, o.name, o.team_id, o.description, o.icon_url, o.raw_icon_url, o.color
                        FROM organizations o
                        WHERE o.id = ANY($1) OR LOWER(o.slug) = ANY($2)
                        GROUP BY o.id;
                        ",
                        &org_ids,
                        &slugs,
                    )
                    .fetch(exec)
                    .try_fold(DashMap::new(), |acc, m| {
                        let org = DBOrganization {
                            id: DBOrganizationId(m.id),
                            slug: m.slug.clone(),
                            name: m.name,
                            team_id: DBTeamId(m.team_id),
                            description: m.description,
                            icon_url: m.icon_url,
                            raw_icon_url: m.raw_icon_url,
                            color: m.color.map(|x| x as u32),
                        };

                        acc.insert(m.id, (Some(m.slug), org));
                        async move { Ok(acc) }
                    })
                    .await?;

                    Ok(organizations)
                },
            )
            .await?;

        Ok(val)
    }

    // Gets organization associated with a project ID, if it exists and there is one
    pub async fn get_associated_organization_project_id<'a, 'b, E>(
        project_id: DBProjectId,
        exec: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT o.id, o.slug, o.name, o.team_id, o.description, o.icon_url, o.raw_icon_url, o.color
            FROM organizations o
            LEFT JOIN mods m ON m.organization_id = o.id
            WHERE m.id = $1
            GROUP BY o.id;
            ",
            project_id as DBProjectId,
        )
        .fetch_optional(exec)
        .await?;

        if let Some(result) = result {
            Ok(Some(DBOrganization {
                id: DBOrganizationId(result.id),
                slug: result.slug,
                name: result.name,
                team_id: DBTeamId(result.team_id),
                description: result.description,
                icon_url: result.icon_url,
                raw_icon_url: result.raw_icon_url,
                color: result.color.map(|x| x as u32),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn remove(
        id: DBOrganizationId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, super::DatabaseError> {
        let organization = Self::get_id(id, &mut **transaction, redis).await?;

        if let Some(organization) = organization {
            sqlx::query!(
                "
                DELETE FROM organizations
                WHERE id = $1
                ",
                id as DBOrganizationId,
            )
            .execute(&mut **transaction)
            .await?;

            DBTeamMember::clear_cache(organization.team_id, redis).await?;

            sqlx::query!(
                "
                DELETE FROM team_members
                WHERE team_id = $1
                ",
                organization.team_id as DBTeamId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM teams
                WHERE id = $1
                ",
                organization.team_id as DBTeamId,
            )
            .execute(&mut **transaction)
            .await?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub async fn clear_cache(
        id: DBOrganizationId,
        slug: Option<String>,
        redis: &RedisPool,
    ) -> Result<(), super::DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .delete_many([
                (ORGANIZATIONS_NAMESPACE, Some(id.0.to_string())),
                (
                    ORGANIZATIONS_TITLES_NAMESPACE,
                    slug.map(|x| x.to_lowercase()),
                ),
            ])
            .await?;
        Ok(())
    }
}
