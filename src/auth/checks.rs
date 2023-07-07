use crate::database;
use crate::database::models::project_item::QueryProject;
use crate::database::models::version_item::QueryVersion;
use crate::database::{models, Project, Version};
use crate::models::users::User;
use crate::routes::ApiError;
use actix_web::web;
use sqlx::PgPool;

pub async fn is_authorized(
    project_data: &Project,
    user_option: &Option<User>,
    pool: &web::Data<PgPool>,
) -> Result<bool, ApiError> {
    let mut authorized = !project_data.status.is_hidden();

    if let Some(user) = &user_option {
        if !authorized {
            if user.role.is_mod() {
                authorized = true;
            } else {
                let user_id: models::ids::UserId = user.id.into();

                let project_exists = sqlx::query!(
                    "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
                    project_data.team_id as database::models::ids::TeamId,
                    user_id as database::models::ids::UserId,
                )
                .fetch_one(&***pool)
                .await?
                .exists;

                authorized = project_exists.unwrap_or(false);
            }
        }
    }

    Ok(authorized)
}

pub async fn filter_authorized_projects(
    projects: Vec<QueryProject>,
    user_option: &Option<User>,
    pool: &web::Data<PgPool>,
) -> Result<Vec<crate::models::projects::Project>, ApiError> {
    let mut return_projects = Vec::new();
    let mut check_projects = Vec::new();

    for project in projects {
        if !project.inner.status.is_hidden()
            || user_option
                .as_ref()
                .map(|x| x.role.is_mod())
                .unwrap_or(false)
        {
            return_projects.push(project.into());
        } else if user_option.is_some() {
            check_projects.push(project);
        }
    }

    if !check_projects.is_empty() {
        if let Some(user) = user_option {
            let user_id: models::ids::UserId = user.id.into();

            use futures::TryStreamExt;

            sqlx::query!(
                "
                SELECT m.id id, m.team_id team_id FROM team_members tm
                INNER JOIN mods m ON m.team_id = tm.team_id
                WHERE tm.team_id = ANY($1) AND tm.user_id = $2
                ",
                &check_projects
                    .iter()
                    .map(|x| x.inner.team_id.0)
                    .collect::<Vec<_>>(),
                user_id as database::models::ids::UserId,
            )
            .fetch_many(&***pool)
            .try_for_each(|e| {
                if let Some(row) = e.right() {
                    check_projects.retain(|x| {
                        let bool = x.inner.id.0 == row.id && x.inner.team_id.0 == row.team_id;

                        if bool {
                            return_projects.push(x.clone().into());
                        }

                        !bool
                    });
                }

                futures::future::ready(Ok(()))
            })
            .await?;
        }
    }

    Ok(return_projects)
}

pub async fn is_authorized_version(
    version_data: &Version,
    user_option: &Option<User>,
    pool: &web::Data<PgPool>,
) -> Result<bool, ApiError> {
    let mut authorized = !version_data.status.is_hidden();

    if let Some(user) = &user_option {
        if !authorized {
            if user.role.is_mod() {
                authorized = true;
            } else {
                let user_id: models::ids::UserId = user.id.into();

                let version_exists = sqlx::query!(
                    "SELECT EXISTS(SELECT 1 FROM mods m INNER JOIN team_members tm ON tm.team_id = m.team_id AND user_id = $2 WHERE m.id = $1)",
                    version_data.project_id as database::models::ids::ProjectId,
                    user_id as database::models::ids::UserId,
                )
                    .fetch_one(&***pool)
                    .await?
                    .exists;

                authorized = version_exists.unwrap_or(false);
            }
        }
    }

    Ok(authorized)
}

pub async fn filter_authorized_versions(
    versions: Vec<QueryVersion>,
    user_option: &Option<User>,
    pool: &web::Data<PgPool>,
) -> Result<Vec<crate::models::projects::Version>, ApiError> {
    let mut return_versions = Vec::new();
    let mut check_versions = Vec::new();

    for version in versions {
        if !version.inner.status.is_hidden()
            || user_option
                .as_ref()
                .map(|x| x.role.is_mod())
                .unwrap_or(false)
        {
            return_versions.push(version.into());
        } else if user_option.is_some() {
            check_versions.push(version);
        }
    }

    if !check_versions.is_empty() {
        if let Some(user) = user_option {
            let user_id: models::ids::UserId = user.id.into();

            use futures::TryStreamExt;

            sqlx::query!(
                "
                SELECT m.id FROM mods m
                INNER JOIN team_members tm ON tm.team_id = m.team_id AND user_id = $2
                WHERE m.id = ANY($1)
                ",
                &check_versions
                    .iter()
                    .map(|x| x.inner.project_id.0)
                    .collect::<Vec<_>>(),
                user_id as database::models::ids::UserId,
            )
            .fetch_many(&***pool)
            .try_for_each(|e| {
                if let Some(row) = e.right() {
                    check_versions.retain(|x| {
                        let bool = x.inner.project_id.0 == row.id;

                        if bool {
                            return_versions.push(x.clone().into());
                        }

                        !bool
                    });
                }

                futures::future::ready(Ok(()))
            })
            .await?;
        }
    }

    Ok(return_versions)
}
