use crate::database;
use crate::database::models::DBCollection;
use crate::database::models::project_item::ProjectQueryResult;
use crate::database::models::version_item::VersionQueryResult;
use crate::database::redis::RedisPool;
use crate::database::{DBProject, DBVersion, models};
use crate::models::users::User;
use crate::routes::ApiError;
use itertools::Itertools;
use sqlx::PgPool;

pub trait ValidateAuthorized {
    fn validate_authorized(
        &self,
        user_option: Option<&User>,
    ) -> Result<(), ApiError>;
}

pub trait ValidateAllAuthorized {
    fn validate_all_authorized(
        self,
        user_option: Option<&User>,
    ) -> Result<(), ApiError>;
}

impl<'a, T, A> ValidateAllAuthorized for T
where
    T: IntoIterator<Item = &'a A>,
    A: ValidateAuthorized + 'a,
{
    fn validate_all_authorized(
        self,
        user_option: Option<&User>,
    ) -> Result<(), ApiError> {
        self.into_iter()
            .try_for_each(|c| c.validate_authorized(user_option))
    }
}

pub async fn is_visible_project(
    project_data: &DBProject,
    user_option: &Option<User>,
    pool: &PgPool,
    hide_unlisted: bool,
) -> Result<bool, ApiError> {
    filter_visible_project_ids(
        vec![project_data],
        user_option,
        pool,
        hide_unlisted,
    )
    .await
    .map(|x| !x.is_empty())
}

pub async fn is_team_member_project(
    project_data: &DBProject,
    user_option: &Option<User>,
    pool: &PgPool,
) -> Result<bool, ApiError> {
    filter_enlisted_projects_ids(vec![project_data], user_option, pool)
        .await
        .map(|x| !x.is_empty())
}

pub async fn filter_visible_projects(
    mut projects: Vec<ProjectQueryResult>,
    user_option: &Option<User>,
    pool: &PgPool,
    hide_unlisted: bool,
) -> Result<Vec<crate::models::projects::Project>, ApiError> {
    let filtered_project_ids = filter_visible_project_ids(
        projects.iter().map(|x| &x.inner).collect_vec(),
        user_option,
        pool,
        hide_unlisted,
    )
    .await?;
    projects.retain(|x| filtered_project_ids.contains(&x.inner.id));
    Ok(projects.into_iter().map(|x| x.into()).collect())
}

// Filters projects for which we can see, meaning one of the following is true:
// - it's not hidden
// - the user is enlisted on the project's team (filter_enlisted_projects)
// - the user is a mod
// This is essentially whether you can know of the project's existence
pub async fn filter_visible_project_ids(
    projects: Vec<&DBProject>,
    user_option: &Option<User>,
    pool: &PgPool,
    hide_unlisted: bool,
) -> Result<Vec<crate::database::models::DBProjectId>, ApiError> {
    let mut return_projects = Vec::new();
    let mut check_projects = Vec::new();

    // Return projects that are not hidden or we are a mod of
    for project in projects {
        if (if hide_unlisted {
            project.status.is_searchable()
        } else {
            !project.status.is_hidden()
        }) || user_option.as_ref().is_some_and(|x| x.role.is_mod())
        {
            return_projects.push(project.id);
        } else if user_option.is_some() {
            check_projects.push(project);
        }
    }

    // For hidden projects, return a filtered list of projects for which we are enlisted on the team
    if !check_projects.is_empty() {
        return_projects.extend(
            filter_enlisted_projects_ids(check_projects, user_option, pool)
                .await?,
        );
    }

    Ok(return_projects)
}

// Filters out projects for which we are a member of the team (or a mod)
// These are projects we have internal access to and can potentially see even if they are hidden
// This is useful for getting visibility of versions, or seeing analytics or sensitive team-restricted data of a project
pub async fn filter_enlisted_projects_ids(
    projects: Vec<&DBProject>,
    user_option: &Option<User>,
    pool: &PgPool,
) -> Result<Vec<crate::database::models::DBProjectId>, ApiError> {
    let mut return_projects = vec![];

    if let Some(user) = user_option {
        let user_id: models::ids::DBUserId = user.id.into();

        use futures::TryStreamExt;

        sqlx::query!(
            "
            SELECT m.id id, m.team_id team_id FROM team_members tm
            INNER JOIN mods m ON m.team_id = tm.team_id
            LEFT JOIN organizations o ON o.team_id = tm.team_id
            WHERE tm.team_id = ANY($1) AND tm.user_id = $3
            UNION
            SELECT m.id id, m.team_id team_id FROM team_members tm
            INNER JOIN organizations o ON o.team_id = tm.team_id
            INNER JOIN mods m ON m.organization_id = o.id
            WHERE o.id = ANY($2) AND tm.user_id = $3
            ",
            &projects.iter().map(|x| x.team_id.0).collect::<Vec<_>>(),
            &projects
                .iter()
                .filter_map(|x| x.organization_id.map(|x| x.0))
                .collect::<Vec<_>>(),
            user_id as database::models::ids::DBUserId,
        )
        .fetch(pool)
        .map_ok(|row| {
            for x in &projects {
                let bool =
                    Some(x.id.0) == row.id && Some(x.team_id.0) == row.team_id;
                if bool {
                    return_projects.push(x.id);
                }
            }
        })
        .try_collect::<Vec<()>>()
        .await?;
    }
    Ok(return_projects)
}

pub async fn is_visible_version(
    version_data: &DBVersion,
    user_option: &Option<User>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<bool, ApiError> {
    filter_visible_version_ids(vec![version_data], user_option, pool, redis)
        .await
        .map(|x| !x.is_empty())
}

pub async fn is_team_member_version(
    version_data: &DBVersion,
    user_option: &Option<User>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<bool, ApiError> {
    filter_enlisted_version_ids(vec![version_data], user_option, pool, redis)
        .await
        .map(|x| !x.is_empty())
}

pub async fn filter_visible_versions(
    mut versions: Vec<VersionQueryResult>,
    user_option: &Option<User>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Vec<crate::models::projects::Version>, ApiError> {
    let filtered_version_ids = filter_visible_version_ids(
        versions.iter().map(|x| &x.inner).collect_vec(),
        user_option,
        pool,
        redis,
    )
    .await?;
    versions.retain(|x| filtered_version_ids.contains(&x.inner.id));
    Ok(versions.into_iter().map(|x| x.into()).collect())
}

impl ValidateAuthorized for models::DBOAuthClient {
    fn validate_authorized(
        &self,
        user_option: Option<&User>,
    ) -> Result<(), ApiError> {
        if let Some(user) = user_option {
            return if user.role.is_mod() || user.id == self.created_by.into() {
                Ok(())
            } else {
                Err(ApiError::CustomAuthentication(
                    "You don't have sufficient permissions to interact with this OAuth application"
                        .to_string(),
                ))
            };
        }

        Ok(())
    }
}

pub async fn filter_visible_version_ids(
    versions: Vec<&DBVersion>,
    user_option: &Option<User>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Vec<crate::database::models::DBVersionId>, ApiError> {
    let mut return_versions = Vec::new();

    // First, filter out versions belonging to projects we can't see
    // (ie: a hidden project, but public version, should still be hidden)
    // Gets project ids of versions
    let project_ids = versions.iter().map(|x| x.project_id).collect::<Vec<_>>();

    // Get visible projects- ones we are allowed to see public versions for.
    let visible_project_ids = filter_visible_project_ids(
        DBProject::get_many_ids(&project_ids, pool, redis)
            .await?
            .iter()
            .map(|x| &x.inner)
            .collect(),
        user_option,
        pool,
        false,
    )
    .await?;

    // Then, get enlisted versions (Versions that are a part of a project we are a member of)
    let enlisted_version_ids =
        filter_enlisted_version_ids(versions.clone(), user_option, pool, redis)
            .await?;

    // Return versions that are not hidden, we are a mod of, or we are enlisted on the team of
    for version in versions {
        // We can see the version if:
        // - it's not hidden and we can see the project
        // - we are a mod
        // - we are enlisted on the team of the mod
        if (!version.status.is_hidden()
            && visible_project_ids.contains(&version.project_id))
            || user_option.as_ref().is_some_and(|x| x.role.is_mod())
            || enlisted_version_ids.contains(&version.id)
        {
            return_versions.push(version.id);
        }
    }

    Ok(return_versions)
}

pub async fn filter_enlisted_version_ids(
    versions: Vec<&DBVersion>,
    user_option: &Option<User>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Vec<crate::database::models::DBVersionId>, ApiError> {
    let mut return_versions = Vec::new();

    // Get project ids of versions
    let project_ids = versions.iter().map(|x| x.project_id).collect::<Vec<_>>();

    // Get enlisted projects- ones we are allowed to see hidden versions for.
    let authorized_project_ids = filter_enlisted_projects_ids(
        DBProject::get_many_ids(&project_ids, pool, redis)
            .await?
            .iter()
            .map(|x| &x.inner)
            .collect(),
        user_option,
        pool,
    )
    .await?;

    for version in versions {
        if user_option.as_ref().is_some_and(|x| x.role.is_mod())
            || (user_option.is_some()
                && authorized_project_ids.contains(&version.project_id))
        {
            return_versions.push(version.id);
        }
    }

    Ok(return_versions)
}

pub async fn is_visible_collection(
    collection_data: &DBCollection,
    user_option: &Option<User>,
) -> Result<bool, ApiError> {
    let mut authorized = !collection_data.status.is_hidden()
        && !collection_data.projects.is_empty();
    if let Some(user) = &user_option {
        if !authorized
            && (user.role.is_mod() || user.id == collection_data.user_id.into())
        {
            authorized = true;
        }
    }
    Ok(authorized)
}

pub async fn filter_visible_collections(
    collections: Vec<DBCollection>,
    user_option: &Option<User>,
) -> Result<Vec<crate::models::collections::Collection>, ApiError> {
    let mut return_collections = Vec::new();
    let mut check_collections = Vec::new();

    for collection in collections {
        if (!collection.status.is_hidden() && !collection.projects.is_empty())
            || user_option.as_ref().is_some_and(|x| x.role.is_mod())
        {
            return_collections.push(collection.into());
        } else if user_option.is_some() {
            check_collections.push(collection);
        }
    }

    for collection in check_collections {
        // Collections are simple- if we are the owner or a mod, we can see it
        if let Some(user) = user_option {
            if user.role.is_mod() || user.id == collection.user_id.into() {
                return_collections.push(collection.into());
            }
        }
    }

    Ok(return_collections)
}
