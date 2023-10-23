use crate::{
    database::redis::RedisPool,
    models::ids::base62_impl::{parse_base62, to_base62},
};

use super::{ids::*, TeamMember};
use serde::{Deserialize, Serialize};

const ORGANIZATIONS_NAMESPACE: &str = "organizations";
const ORGANIZATIONS_TITLES_NAMESPACE: &str = "organizations_titles";

#[derive(Deserialize, Serialize, Clone, Debug)]
/// An organization of users who together control one or more projects and organizations.
pub struct Organization {
    /// The id of the organization
    pub id: OrganizationId,

    /// The title (and slug) of the organization
    pub title: String,

    /// The associated team of the organization
    pub team_id: TeamId,

    /// The description of the organization
    pub description: String,

    /// The display icon for the organization
    pub icon_url: Option<String>,
    pub color: Option<u32>,
}

impl Organization {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), super::DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO organizations (id, title, team_id, description, icon_url, color)
            VALUES ($1, $2, $3, $4, $5, $6)
            ",
            self.id.0,
            self.title,
            self.team_id as TeamId,
            self.description,
            self.icon_url,
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
        id: OrganizationId,
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
        organization_ids: &[OrganizationId],
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

    pub async fn get_many<'a, E, T: ToString>(
        organization_strings: &[T],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        if organization_strings.is_empty() {
            return Ok(Vec::new());
        }

        let mut found_organizations = Vec::new();
        let mut remaining_strings = organization_strings
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();

        let mut organization_ids = organization_strings
            .iter()
            .flat_map(|x| parse_base62(&x.to_string()).map(|x| x as i64))
            .collect::<Vec<_>>();

        organization_ids.append(
            &mut redis
                .multi_get::<i64, _>(
                    ORGANIZATIONS_TITLES_NAMESPACE,
                    organization_strings
                        .iter()
                        .map(|x| x.to_string().to_lowercase()),
                )
                .await?
                .into_iter()
                .flatten()
                .collect(),
        );

        if !organization_ids.is_empty() {
            let organizations = redis
                .multi_get::<String, _>(ORGANIZATIONS_NAMESPACE, organization_ids)
                .await?;

            for organization in organizations {
                if let Some(organization) =
                    organization.and_then(|x| serde_json::from_str::<Organization>(&x).ok())
                {
                    remaining_strings.retain(|x| {
                        &to_base62(organization.id.0 as u64) != x
                            && organization.title.to_lowercase() != x.to_lowercase()
                    });
                    found_organizations.push(organization);
                    continue;
                }
            }
        }

        if !remaining_strings.is_empty() {
            let organization_ids_parsed: Vec<i64> = remaining_strings
                .iter()
                .flat_map(|x| parse_base62(&x.to_string()).ok())
                .map(|x| x as i64)
                .collect();

            let organizations: Vec<Organization> = sqlx::query!(
                "
                SELECT o.id, o.title, o.team_id, o.description, o.icon_url, o.color
                FROM organizations o
                WHERE o.id = ANY($1) OR o.title = ANY($2)
                GROUP BY o.id;
                ",
                &organization_ids_parsed,
                &remaining_strings
                    .into_iter()
                    .map(|x| x.to_string().to_lowercase())
                    .collect::<Vec<_>>(),
            )
            .fetch_many(exec)
            .try_filter_map(|e| async {
                Ok(e.right().map(|m| Organization {
                    id: OrganizationId(m.id),
                    title: m.title,
                    team_id: TeamId(m.team_id),
                    description: m.description,
                    icon_url: m.icon_url,
                    color: m.color.map(|x| x as u32),
                }))
            })
            .try_collect::<Vec<Organization>>()
            .await?;

            for organization in organizations {
                redis
                    .set_serialized_to_json(
                        ORGANIZATIONS_NAMESPACE,
                        organization.id.0,
                        &organization,
                        None,
                    )
                    .await?;
                redis
                    .set(
                        ORGANIZATIONS_TITLES_NAMESPACE,
                        organization.title.to_lowercase(),
                        organization.id.0,
                        None,
                    )
                    .await?;

                found_organizations.push(organization);
            }
        }

        Ok(found_organizations)
    }

    // Gets organization associated with a project ID, if it exists and there is one
    pub async fn get_associated_organization_project_id<'a, 'b, E>(
        project_id: ProjectId,
        exec: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT o.id, o.title, o.team_id, o.description, o.icon_url, o.color
            FROM organizations o
            LEFT JOIN mods m ON m.organization_id = o.id
            WHERE m.id = $1
            GROUP BY o.id;
            ",
            project_id as ProjectId,
        )
        .fetch_optional(exec)
        .await?;

        if let Some(result) = result {
            Ok(Some(Organization {
                id: OrganizationId(result.id),
                title: result.title,
                team_id: TeamId(result.team_id),
                description: result.description,
                icon_url: result.icon_url,
                color: result.color.map(|x| x as u32),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn remove(
        id: OrganizationId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, super::DatabaseError> {
        use futures::TryStreamExt;

        let organization = Self::get_id(id, &mut **transaction, redis).await?;

        if let Some(organization) = organization {
            let projects: Vec<ProjectId> = sqlx::query!(
                "
                SELECT m.id
                FROM mods m
                WHERE m.organization_id = $1
                ",
                id as OrganizationId,
            )
            .fetch_many(&mut **transaction)
            .try_filter_map(|e| async { Ok(e.right().map(|m| ProjectId(m.id))) })
            .try_collect::<Vec<ProjectId>>()
            .await?;

            for project_id in projects {
                let _result =
                    super::project_item::Project::remove(project_id, transaction, redis).await?;
            }

            Organization::clear_cache(id, Some(organization.title), redis).await?;

            sqlx::query!(
                "
                DELETE FROM organizations
                WHERE id = $1
                ",
                id as OrganizationId,
            )
            .execute(&mut **transaction)
            .await?;

            TeamMember::clear_cache(organization.team_id, redis).await?;

            sqlx::query!(
                "
                DELETE FROM team_members
                WHERE team_id = $1
                ",
                organization.team_id as TeamId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM teams
                WHERE id = $1
                ",
                organization.team_id as TeamId,
            )
            .execute(&mut **transaction)
            .await?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub async fn clear_cache(
        id: OrganizationId,
        title: Option<String>,
        redis: &RedisPool,
    ) -> Result<(), super::DatabaseError> {
        redis
            .delete_many([
                (ORGANIZATIONS_NAMESPACE, Some(id.0.to_string())),
                (
                    ORGANIZATIONS_TITLES_NAMESPACE,
                    title.map(|x| x.to_lowercase()),
                ),
            ])
            .await?;
        Ok(())
    }
}
