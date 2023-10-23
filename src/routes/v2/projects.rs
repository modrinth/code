use crate::auth::{filter_authorized_projects, get_user_from_headers, is_authorized};
use crate::database;
use crate::database::models::image_item;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::project_item::{GalleryItem, ModCategory};
use crate::database::models::thread_item::ThreadMessageBuilder;
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models;
use crate::models::ids::base62_impl::parse_base62;
use crate::models::images::ImageContext;
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::projects::{
    DonationLink, MonetizationStatus, Project, ProjectId, ProjectStatus, SearchRequest, SideType,
};
use crate::models::teams::ProjectPermissions;
use crate::models::threads::MessageBody;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::search::{search_for_project, SearchConfig, SearchError};
use crate::util::img;
use crate::util::routes::read_from_payload;
use crate::util::validate::validation_errors_to_string;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use meilisearch_sdk::indexes::IndexesResults;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use validator::Validate;

use database::models as db_models;
use db_models::ids as db_ids;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(project_search);
    cfg.service(projects_get);
    cfg.service(projects_edit);
    cfg.service(random_projects_get);

    cfg.service(
        web::scope("project")
            .service(project_get)
            .service(project_get_check)
            .service(project_delete)
            .service(project_edit)
            .service(project_icon_edit)
            .service(delete_project_icon)
            .service(add_gallery_item)
            .service(edit_gallery_item)
            .service(delete_gallery_item)
            .service(project_follow)
            .service(project_unfollow)
            .service(project_schedule)
            .service(super::teams::team_members_get_project)
            .service(
                web::scope("{project_id}")
                    .service(super::versions::version_list)
                    .service(super::versions::version_project_get)
                    .service(dependency_list),
            ),
    );
}

#[get("search")]
pub async fn project_search(
    web::Query(info): web::Query<SearchRequest>,
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, SearchError> {
    let results = search_for_project(&info, &config).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[derive(Deserialize, Validate)]
pub struct RandomProjects {
    #[validate(range(min = 1, max = 100))]
    pub count: u32,
}

#[get("projects_random")]
pub async fn random_projects_get(
    web::Query(count): web::Query<RandomProjects>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    count
        .validate()
        .map_err(|err| ApiError::Validation(validation_errors_to_string(err, None)))?;

    let project_ids = sqlx::query!(
        "
            SELECT id FROM mods TABLESAMPLE SYSTEM_ROWS($1) WHERE status = ANY($2)
            ",
        count.count as i32,
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| x.is_searchable())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch_many(&**pool)
    .try_filter_map(|e| async { Ok(e.right().map(|m| db_ids::ProjectId(m.id))) })
    .try_collect::<Vec<_>>()
    .await?;

    let projects_data = db_models::Project::get_many_ids(&project_ids, &**pool, &redis)
        .await?
        .into_iter()
        .map(Project::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(projects_data))
}

#[derive(Serialize, Deserialize)]
pub struct ProjectIds {
    pub ids: String,
}

#[get("projects")]
pub async fn projects_get(
    req: HttpRequest,
    web::Query(ids): web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids = serde_json::from_str::<Vec<&str>>(&ids.ids)?;
    let projects_data = db_models::Project::get_many(&ids, &**pool, &redis).await?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let projects = filter_authorized_projects(projects_data, &user_option, &pool).await?;

    Ok(HttpResponse::Ok().json(projects))
}

#[get("{id}")]
pub async fn project_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let project_data = db_models::Project::get(&string, &**pool, &redis).await?;
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    if let Some(data) = project_data {
        if is_authorized(&data.inner, &user_option, &pool).await? {
            return Ok(HttpResponse::Ok().json(Project::from(data)));
        }
    }
    Ok(HttpResponse::NotFound().body(""))
}

//checks the validity of a project id or slug
#[get("{id}/check")]
pub async fn project_get_check(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let slug = info.into_inner().0;

    let project_data = db_models::Project::get(&slug, &**pool, &redis).await?;

    if let Some(project) = project_data {
        Ok(HttpResponse::Ok().json(json! ({
            "id": models::ids::ProjectId::from(project.inner.id)
        })))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Serialize)]
struct DependencyInfo {
    pub projects: Vec<Project>,
    pub versions: Vec<models::projects::Version>,
}

#[get("dependencies")]
pub async fn dependency_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let result = db_models::Project::get(&string, &**pool, &redis).await?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    if let Some(project) = result {
        if !is_authorized(&project.inner, &user_option, &pool).await? {
            return Ok(HttpResponse::NotFound().body(""));
        }

        let dependencies =
            database::Project::get_dependencies(project.inner.id, &**pool, &redis).await?;

        let project_ids = dependencies
            .iter()
            .filter_map(|x| {
                if x.0.is_none() {
                    if let Some(mod_dependency_id) = x.2 {
                        Some(mod_dependency_id)
                    } else {
                        x.1
                    }
                } else {
                    x.1
                }
            })
            .collect::<Vec<_>>();

        let dep_version_ids = dependencies
            .iter()
            .filter_map(|x| x.0)
            .collect::<Vec<db_models::VersionId>>();
        let (projects_result, versions_result) = futures::future::try_join(
            database::Project::get_many_ids(&project_ids, &**pool, &redis),
            database::Version::get_many(&dep_version_ids, &**pool, &redis),
        )
        .await?;

        let mut projects = projects_result
            .into_iter()
            .map(models::projects::Project::from)
            .collect::<Vec<_>>();
        let mut versions = versions_result
            .into_iter()
            .map(models::projects::Version::from)
            .collect::<Vec<_>>();

        projects.sort_by(|a, b| b.published.cmp(&a.published));
        projects.dedup_by(|a, b| a.id == b.id);

        versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));
        versions.dedup_by(|a, b| a.id == b.id);

        Ok(HttpResponse::Ok().json(DependencyInfo { projects, versions }))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct EditProject {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub title: Option<String>,
    #[validate(length(min = 3, max = 256))]
    pub description: Option<String>,
    #[validate(length(max = 65536))]
    pub body: Option<String>,
    #[validate(length(max = 3))]
    pub categories: Option<Vec<String>>,
    #[validate(length(max = 256))]
    pub additional_categories: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub issues_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub source_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub wiki_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub license_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub discord_url: Option<Option<String>>,
    #[validate]
    pub donation_urls: Option<Vec<DonationLink>>,
    pub license_id: Option<String>,
    pub client_side: Option<SideType>,
    pub server_side: Option<SideType>,
    #[validate(
        length(min = 3, max = 64),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    pub slug: Option<String>,
    pub status: Option<ProjectStatus>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub requested_status: Option<Option<ProjectStatus>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 2000))]
    pub moderation_message: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 65536))]
    pub moderation_message_body: Option<Option<String>>,
    pub monetization_status: Option<MonetizationStatus>,
}

#[patch("{id}")]
pub async fn project_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    config: web::Data<SearchConfig>,
    new_project: web::Json<EditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;

    new_project
        .validate()
        .map_err(|err| ApiError::Validation(validation_errors_to_string(err, None)))?;

    let string = info.into_inner().0;
    let result = db_models::Project::get(&string, &**pool, &redis).await?;

    if let Some(project_item) = result {
        let id = project_item.inner.id;

        let (team_member, organization_team_member) =
            db_models::TeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await?;

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        );

        if let Some(perms) = permissions {
            let mut transaction = pool.begin().await?;

            if let Some(title) = &new_project.title {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the title of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET title = $1
                    WHERE (id = $2)
                    ",
                    title.trim(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(description) = &new_project.description {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the description of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET description = $1
                    WHERE (id = $2)
                    ",
                    description,
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(status) = &new_project.status {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the status of this project!"
                            .to_string(),
                    ));
                }

                if !(user.role.is_mod()
                    || !project_item.inner.status.is_approved()
                        && status == &ProjectStatus::Processing
                    || project_item.inner.status.is_approved() && status.can_be_requested())
                {
                    return Err(ApiError::CustomAuthentication(
                        "You don't have permission to set this status!".to_string(),
                    ));
                }

                if status == &ProjectStatus::Processing {
                    if project_item.versions.is_empty() {
                        return Err(ApiError::InvalidInput(String::from(
                            "Project submitted for review with no initial versions",
                        )));
                    }

                    sqlx::query!(
                        "
                        UPDATE mods
                        SET moderation_message = NULL, moderation_message_body = NULL, queued = NOW()
                        WHERE (id = $1)
                        ",
                        id as db_ids::ProjectId,
                    )
                    .execute(&mut *transaction)
                    .await?;

                    sqlx::query!(
                        "
                        UPDATE threads
                        SET show_in_mod_inbox = FALSE
                        WHERE id = $1
                        ",
                        project_item.thread_id as db_ids::ThreadId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }

                if status.is_approved() && !project_item.inner.status.is_approved() {
                    sqlx::query!(
                        "
                        UPDATE mods
                        SET approved = NOW()
                        WHERE id = $1 AND approved IS NULL
                        ",
                        id as db_ids::ProjectId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }

                if status.is_searchable() && !project_item.inner.webhook_sent {
                    if let Ok(webhook_url) = dotenvy::var("PUBLIC_DISCORD_WEBHOOK") {
                        crate::util::webhook::send_discord_webhook(
                            project_item.inner.id.into(),
                            &pool,
                            &redis,
                            webhook_url,
                            None,
                        )
                        .await
                        .ok();

                        sqlx::query!(
                            "
                            UPDATE mods
                            SET webhook_sent = TRUE
                            WHERE id = $1
                            ",
                            id as db_ids::ProjectId,
                        )
                        .execute(&mut *transaction)
                        .await?;
                    }
                }

                if user.role.is_mod() {
                    if let Ok(webhook_url) = dotenvy::var("MODERATION_DISCORD_WEBHOOK") {
                        crate::util::webhook::send_discord_webhook(
                            project_item.inner.id.into(),
                            &pool,
                            &redis,
                            webhook_url,
                            Some(
                                format!(
                                    "**[{}]({}/user/{})** changed project status from **{}** to **{}**",
                                    user.username,
                                    dotenvy::var("SITE_URL")?,
                                    user.username,
                                    &project_item.inner.status.as_friendly_str(),
                                    status.as_friendly_str(),
                                )
                                .to_string(),
                            ),
                        )
                        .await
                        .ok();
                    }
                }

                if team_member.map(|x| !x.accepted).unwrap_or(true) {
                    let notified_members = sqlx::query!(
                        "
                        SELECT tm.user_id id
                        FROM team_members tm
                        WHERE tm.team_id = $1 AND tm.accepted
                        ",
                        project_item.inner.team_id as db_ids::TeamId
                    )
                    .fetch_many(&mut *transaction)
                    .try_filter_map(|e| async { Ok(e.right().map(|c| db_models::UserId(c.id))) })
                    .try_collect::<Vec<_>>()
                    .await?;

                    NotificationBuilder {
                        body: NotificationBody::StatusChange {
                            project_id: project_item.inner.id.into(),
                            old_status: project_item.inner.status,
                            new_status: *status,
                        },
                    }
                    .insert_many(notified_members, &mut transaction, &redis)
                    .await?;
                }

                ThreadMessageBuilder {
                    author_id: Some(user.id.into()),
                    body: MessageBody::StatusChange {
                        new_status: *status,
                        old_status: project_item.inner.status,
                    },
                    thread_id: project_item.thread_id,
                }
                .insert(&mut transaction)
                .await?;

                sqlx::query!(
                    "
                    UPDATE mods
                    SET status = $1
                    WHERE (id = $2)
                    ",
                    status.as_str(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;

                if project_item.inner.status.is_searchable() && !status.is_searchable() {
                    delete_from_index(id.into(), config).await?;
                }
            }

            if let Some(requested_status) = &new_project.requested_status {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the requested status of this project!"
                            .to_string(),
                    ));
                }

                if !requested_status
                    .map(|x| x.can_be_requested())
                    .unwrap_or(true)
                {
                    return Err(ApiError::InvalidInput(String::from(
                        "Specified status cannot be requested!",
                    )));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET requested_status = $1
                    WHERE (id = $2)
                    ",
                    requested_status.map(|x| x.as_str()),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if perms.contains(ProjectPermissions::EDIT_DETAILS) {
                if new_project.categories.is_some() {
                    sqlx::query!(
                        "
                        DELETE FROM mods_categories
                        WHERE joining_mod_id = $1 AND is_additional = FALSE
                        ",
                        id as db_ids::ProjectId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }

                if new_project.additional_categories.is_some() {
                    sqlx::query!(
                        "
                        DELETE FROM mods_categories
                        WHERE joining_mod_id = $1 AND is_additional = TRUE
                        ",
                        id as db_ids::ProjectId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }

            if let Some(categories) = &new_project.categories {
                edit_project_categories(
                    categories,
                    &perms,
                    id as db_ids::ProjectId,
                    false,
                    &mut transaction,
                )
                .await?;
            }

            if let Some(categories) = &new_project.additional_categories {
                edit_project_categories(
                    categories,
                    &perms,
                    id as db_ids::ProjectId,
                    true,
                    &mut transaction,
                )
                .await?;
            }

            if let Some(issues_url) = &new_project.issues_url {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the issues URL of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET issues_url = $1
                    WHERE (id = $2)
                    ",
                    issues_url.as_deref(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(source_url) = &new_project.source_url {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the source URL of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET source_url = $1
                    WHERE (id = $2)
                    ",
                    source_url.as_deref(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(wiki_url) = &new_project.wiki_url {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the wiki URL of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET wiki_url = $1
                    WHERE (id = $2)
                    ",
                    wiki_url.as_deref(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(license_url) = &new_project.license_url {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the license URL of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET license_url = $1
                    WHERE (id = $2)
                    ",
                    license_url.as_deref(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(discord_url) = &new_project.discord_url {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the discord URL of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET discord_url = $1
                    WHERE (id = $2)
                    ",
                    discord_url.as_deref(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(slug) = &new_project.slug {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the slug of this project!"
                            .to_string(),
                    ));
                }

                let slug_project_id_option: Option<u64> = parse_base62(slug).ok();
                if let Some(slug_project_id) = slug_project_id_option {
                    let results = sqlx::query!(
                        "
                        SELECT EXISTS(SELECT 1 FROM mods WHERE id=$1)
                        ",
                        slug_project_id as i64
                    )
                    .fetch_one(&mut *transaction)
                    .await?;

                    if results.exists.unwrap_or(true) {
                        return Err(ApiError::InvalidInput(
                            "Slug collides with other project's id!".to_string(),
                        ));
                    }
                }

                // Make sure the new slug is different from the old one
                // We are able to unwrap here because the slug is always set
                if !slug.eq(&project_item.inner.slug.clone().unwrap_or_default()) {
                    let results = sqlx::query!(
                        "
                      SELECT EXISTS(SELECT 1 FROM mods WHERE slug = LOWER($1))
                      ",
                        slug
                    )
                    .fetch_one(&mut *transaction)
                    .await?;

                    if results.exists.unwrap_or(true) {
                        return Err(ApiError::InvalidInput(
                            "Slug collides with other project's id!".to_string(),
                        ));
                    }
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET slug = LOWER($1)
                    WHERE (id = $2)
                    ",
                    Some(slug),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(new_side) = &new_project.client_side {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the side type of this mod!"
                            .to_string(),
                    ));
                }

                let side_type_id =
                    db_models::categories::SideType::get_id(new_side.as_str(), &mut *transaction)
                        .await?
                        .expect("No database entry found for side type");

                sqlx::query!(
                    "
                    UPDATE mods
                    SET client_side = $1
                    WHERE (id = $2)
                    ",
                    side_type_id as db_models::SideTypeId,
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(new_side) = &new_project.server_side {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the side type of this project!"
                            .to_string(),
                    ));
                }

                let side_type_id =
                    db_models::categories::SideType::get_id(new_side.as_str(), &mut *transaction)
                        .await?
                        .expect("No database entry found for side type");

                sqlx::query!(
                    "
                    UPDATE mods
                    SET server_side = $1
                    WHERE (id = $2)
                    ",
                    side_type_id as db_models::SideTypeId,
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(license) = &new_project.license_id {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the license of this project!"
                            .to_string(),
                    ));
                }

                let mut license = license.clone();

                if license.to_lowercase() == "arr" {
                    license = models::projects::DEFAULT_LICENSE_ID.to_string();
                }

                spdx::Expression::parse(&license).map_err(|err| {
                    ApiError::InvalidInput(format!("Invalid SPDX license identifier: {err}"))
                })?;

                sqlx::query!(
                    "
                    UPDATE mods
                    SET license = $1
                    WHERE (id = $2)
                    ",
                    license,
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }
            if let Some(donations) = &new_project.donation_urls {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the donation links of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    DELETE FROM mods_donations
                    WHERE joining_mod_id = $1
                    ",
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;

                for donation in donations {
                    let platform_id = db_models::categories::DonationPlatform::get_id(
                        &donation.id,
                        &mut *transaction,
                    )
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInput(format!(
                            "Platform {} does not exist.",
                            donation.id.clone()
                        ))
                    })?;

                    sqlx::query!(
                        "
                        INSERT INTO mods_donations (joining_mod_id, joining_platform_id, url)
                        VALUES ($1, $2, $3)
                        ",
                        id as db_ids::ProjectId,
                        platform_id as db_ids::DonationPlatformId,
                        donation.url
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }

            if let Some(moderation_message) = &new_project.moderation_message {
                if !user.role.is_mod()
                    && (!project_item.inner.status.is_approved() || moderation_message.is_some())
                {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the moderation message of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET moderation_message = $1
                    WHERE (id = $2)
                    ",
                    moderation_message.as_deref(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(moderation_message_body) = &new_project.moderation_message_body {
                if !user.role.is_mod()
                    && (!project_item.inner.status.is_approved()
                        || moderation_message_body.is_some())
                {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the moderation message body of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET moderation_message_body = $1
                    WHERE (id = $2)
                    ",
                    moderation_message_body.as_deref(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(body) = &new_project.body {
                if !perms.contains(ProjectPermissions::EDIT_BODY) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the body of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET body = $1
                    WHERE (id = $2)
                    ",
                    body,
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(monetization_status) = &new_project.monetization_status {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the monetization status of this project!"
                            .to_string(),
                    ));
                }

                if (*monetization_status == MonetizationStatus::ForceDemonetized
                    || project_item.inner.monetization_status
                        == MonetizationStatus::ForceDemonetized)
                    && !user.role.is_mod()
                {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the monetization status of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET monetization_status = $1
                    WHERE (id = $2)
                    ",
                    monetization_status.as_str(),
                    id as db_ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            // check new description and body for links to associated images
            // if they no longer exist in the description or body, delete them
            let checkable_strings: Vec<&str> = vec![&new_project.description, &new_project.body]
                .into_iter()
                .filter_map(|x| x.as_ref().map(|y| y.as_str()))
                .collect();

            let context = ImageContext::Project {
                project_id: Some(id.into()),
            };

            img::delete_unused_images(context, checkable_strings, &mut transaction, &redis).await?;
            db_models::Project::clear_cache(
                project_item.inner.id,
                project_item.inner.slug,
                None,
                &redis,
            )
            .await?;

            transaction.commit().await?;
            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthentication(
                "You do not have permission to edit this project!".to_string(),
            ))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(derive_new::new)]
pub struct CategoryChanges<'a> {
    pub categories: &'a Option<Vec<String>>,
    pub add_categories: &'a Option<Vec<String>>,
    pub remove_categories: &'a Option<Vec<String>>,
}

#[derive(Deserialize, Validate)]
pub struct BulkEditProject {
    #[validate(length(max = 3))]
    pub categories: Option<Vec<String>>,
    #[validate(length(max = 3))]
    pub add_categories: Option<Vec<String>>,
    pub remove_categories: Option<Vec<String>>,

    #[validate(length(max = 256))]
    pub additional_categories: Option<Vec<String>>,
    #[validate(length(max = 3))]
    pub add_additional_categories: Option<Vec<String>>,
    pub remove_additional_categories: Option<Vec<String>>,

    #[validate]
    pub donation_urls: Option<Vec<DonationLink>>,
    #[validate]
    pub add_donation_urls: Option<Vec<DonationLink>>,
    #[validate]
    pub remove_donation_urls: Option<Vec<DonationLink>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub issues_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub source_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub wiki_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub discord_url: Option<Option<String>>,
}

#[patch("projects")]
pub async fn projects_edit(
    req: HttpRequest,
    web::Query(ids): web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    bulk_edit_project: web::Json<BulkEditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;

    bulk_edit_project
        .validate()
        .map_err(|err| ApiError::Validation(validation_errors_to_string(err, None)))?;

    let project_ids: Vec<db_ids::ProjectId> = serde_json::from_str::<Vec<ProjectId>>(&ids.ids)?
        .into_iter()
        .map(|x| x.into())
        .collect();

    let projects_data = db_models::Project::get_many_ids(&project_ids, &**pool, &redis).await?;

    if let Some(id) = project_ids
        .iter()
        .find(|x| !projects_data.iter().any(|y| x == &&y.inner.id))
    {
        return Err(ApiError::InvalidInput(format!(
            "Project {} not found",
            ProjectId(id.0 as u64)
        )));
    }

    let team_ids = projects_data
        .iter()
        .map(|x| x.inner.team_id)
        .collect::<Vec<db_models::TeamId>>();
    let team_members =
        db_models::TeamMember::get_from_team_full_many(&team_ids, &**pool, &redis).await?;

    let organization_ids = projects_data
        .iter()
        .filter_map(|x| x.inner.organization_id)
        .collect::<Vec<db_models::OrganizationId>>();
    let organizations =
        db_models::Organization::get_many_ids(&organization_ids, &**pool, &redis).await?;

    let organization_team_ids = organizations
        .iter()
        .map(|x| x.team_id)
        .collect::<Vec<db_models::TeamId>>();
    let organization_team_members =
        db_models::TeamMember::get_from_team_full_many(&organization_team_ids, &**pool, &redis)
            .await?;

    let categories = db_models::categories::Category::list(&**pool, &redis).await?;
    let donation_platforms = db_models::categories::DonationPlatform::list(&**pool, &redis).await?;

    let mut transaction = pool.begin().await?;

    for project in projects_data {
        if !user.role.is_mod() {
            let team_member = team_members
                .iter()
                .find(|x| x.team_id == project.inner.team_id && x.user_id == user.id.into());

            let organization = project
                .inner
                .organization_id
                .and_then(|oid| organizations.iter().find(|x| x.id == oid));

            let organization_team_member = if let Some(organization) = organization {
                organization_team_members
                    .iter()
                    .find(|x| x.team_id == organization.team_id && x.user_id == user.id.into())
            } else {
                None
            };

            let permissions = ProjectPermissions::get_permissions_by_role(
                &user.role,
                &team_member.cloned(),
                &organization_team_member.cloned(),
            )
            .unwrap_or_default();

            if team_member.is_some() {
                if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(format!(
                        "You do not have the permissions to bulk edit project {}!",
                        project.inner.title
                    )));
                }
            } else if project.inner.status.is_hidden() {
                return Err(ApiError::InvalidInput(format!(
                    "Project {} not found",
                    ProjectId(project.inner.id.0 as u64)
                )));
            } else {
                return Err(ApiError::CustomAuthentication(format!(
                    "You are not a member of project {}!",
                    project.inner.title
                )));
            };
        }

        bulk_edit_project_categories(
            &categories,
            &project.categories,
            project.inner.id as db_ids::ProjectId,
            CategoryChanges::new(
                &bulk_edit_project.categories,
                &bulk_edit_project.add_categories,
                &bulk_edit_project.remove_categories,
            ),
            3,
            false,
            &mut transaction,
        )
        .await?;

        bulk_edit_project_categories(
            &categories,
            &project.additional_categories,
            project.inner.id as db_ids::ProjectId,
            CategoryChanges::new(
                &bulk_edit_project.additional_categories,
                &bulk_edit_project.add_additional_categories,
                &bulk_edit_project.remove_additional_categories,
            ),
            256,
            true,
            &mut transaction,
        )
        .await?;

        let project_donations: Vec<DonationLink> = project
            .donation_urls
            .into_iter()
            .map(|d| DonationLink {
                id: d.platform_short,
                platform: d.platform_name,
                url: d.url,
            })
            .collect();
        let mut set_donation_links =
            if let Some(donation_links) = bulk_edit_project.donation_urls.clone() {
                donation_links
            } else {
                project_donations.clone()
            };

        if let Some(delete_donations) = &bulk_edit_project.remove_donation_urls {
            for donation in delete_donations {
                if let Some(pos) = set_donation_links
                    .iter()
                    .position(|x| donation.url == x.url && donation.id == x.id)
                {
                    set_donation_links.remove(pos);
                }
            }
        }

        if let Some(add_donations) = &bulk_edit_project.add_donation_urls {
            set_donation_links.append(&mut add_donations.clone());
        }

        if set_donation_links != project_donations {
            sqlx::query!(
                "
                DELETE FROM mods_donations
                WHERE joining_mod_id = $1
                ",
                project.inner.id as db_ids::ProjectId,
            )
            .execute(&mut *transaction)
            .await?;

            for donation in set_donation_links {
                let platform_id = donation_platforms
                    .iter()
                    .find(|x| x.short == donation.id)
                    .ok_or_else(|| {
                        ApiError::InvalidInput(format!(
                            "Platform {} does not exist.",
                            donation.id.clone()
                        ))
                    })?
                    .id;

                sqlx::query!(
                    "
                    INSERT INTO mods_donations (joining_mod_id, joining_platform_id, url)
                    VALUES ($1, $2, $3)
                    ",
                    project.inner.id as db_ids::ProjectId,
                    platform_id as db_ids::DonationPlatformId,
                    donation.url
                )
                .execute(&mut *transaction)
                .await?;
            }
        }

        if let Some(issues_url) = &bulk_edit_project.issues_url {
            sqlx::query!(
                "
                UPDATE mods
                SET issues_url = $1
                WHERE (id = $2)
                ",
                issues_url.as_deref(),
                project.inner.id as db_ids::ProjectId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        if let Some(source_url) = &bulk_edit_project.source_url {
            sqlx::query!(
                "
                UPDATE mods
                SET source_url = $1
                WHERE (id = $2)
                ",
                source_url.as_deref(),
                project.inner.id as db_ids::ProjectId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        if let Some(wiki_url) = &bulk_edit_project.wiki_url {
            sqlx::query!(
                "
                UPDATE mods
                SET wiki_url = $1
                WHERE (id = $2)
                ",
                wiki_url.as_deref(),
                project.inner.id as db_ids::ProjectId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        if let Some(discord_url) = &bulk_edit_project.discord_url {
            sqlx::query!(
                "
                UPDATE mods
                SET discord_url = $1
                WHERE (id = $2)
                ",
                discord_url.as_deref(),
                project.inner.id as db_ids::ProjectId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        db_models::Project::clear_cache(project.inner.id, project.inner.slug, None, &redis).await?;
    }

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

pub async fn bulk_edit_project_categories(
    all_db_categories: &[db_models::categories::Category],
    project_categories: &Vec<String>,
    project_id: db_ids::ProjectId,
    bulk_changes: CategoryChanges<'_>,
    max_num_categories: usize,
    is_additional: bool,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), ApiError> {
    let mut set_categories = if let Some(categories) = bulk_changes.categories.clone() {
        categories
    } else {
        project_categories.clone()
    };

    if let Some(delete_categories) = &bulk_changes.remove_categories {
        for category in delete_categories {
            if let Some(pos) = set_categories.iter().position(|x| x == category) {
                set_categories.remove(pos);
            }
        }
    }

    if let Some(add_categories) = &bulk_changes.add_categories {
        for category in add_categories {
            if set_categories.len() < max_num_categories {
                set_categories.push(category.clone());
            } else {
                break;
            }
        }
    }

    if &set_categories != project_categories {
        sqlx::query!(
            "
            DELETE FROM mods_categories
            WHERE joining_mod_id = $1 AND is_additional = $2
            ",
            project_id as db_ids::ProjectId,
            is_additional
        )
        .execute(&mut **transaction)
        .await?;

        let mut mod_categories = Vec::new();
        for category in set_categories {
            let category_id = all_db_categories
                .iter()
                .find(|x| x.category == category)
                .ok_or_else(|| {
                    ApiError::InvalidInput(format!("Category {} does not exist.", category.clone()))
                })?
                .id;
            mod_categories.push(ModCategory::new(project_id, category_id, is_additional));
        }
        ModCategory::insert_many(mod_categories, &mut *transaction).await?;
    }

    Ok(())
}

pub async fn edit_project_categories(
    categories: &Vec<String>,
    perms: &ProjectPermissions,
    project_id: db_ids::ProjectId,
    additional: bool,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), ApiError> {
    if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
        let additional_str = if additional { "additional " } else { "" };
        return Err(ApiError::CustomAuthentication(format!(
            "You do not have the permissions to edit the {additional_str}categories of this project!"
        )));
    }

    let mut mod_categories = Vec::new();
    for category in categories {
        let category_id = db_models::categories::Category::get_id(category, &mut **transaction)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(format!("Category {} does not exist.", category.clone()))
            })?;
        mod_categories.push(ModCategory::new(project_id, category_id, additional));
    }
    ModCategory::insert_many(mod_categories, &mut *transaction).await?;

    Ok(())
}

#[derive(Deserialize)]
pub struct SchedulingData {
    pub time: DateTime<Utc>,
    pub requested_status: ProjectStatus,
}

#[post("{id}/schedule")]
pub async fn project_schedule(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    scheduling_data: web::Json<SchedulingData>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;

    if scheduling_data.time < Utc::now() {
        return Err(ApiError::InvalidInput(
            "You cannot schedule a project to be released in the past!".to_string(),
        ));
    }

    if !scheduling_data.requested_status.can_be_requested() {
        return Err(ApiError::InvalidInput(
            "Specified requested status cannot be requested!".to_string(),
        ));
    }

    let string = info.into_inner().0;
    let result = db_models::Project::get(&string, &**pool, &redis).await?;

    if let Some(project_item) = result {
        let (team_member, organization_team_member) =
            db_models::TeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await?;

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member.clone(),
            &organization_team_member.clone(),
        )
        .unwrap_or_default();

        if !user.role.is_mod() && !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to edit this project's scheduling data!".to_string(),
            ));
        }

        if !project_item.inner.status.is_approved() {
            return Err(ApiError::InvalidInput(
                "This project has not been approved yet. Submit to the queue with the private status to schedule it in the future!".to_string(),
            ));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET status = $1, approved = $2
            WHERE (id = $3)
            ",
            ProjectStatus::Scheduled.as_str(),
            scheduling_data.time,
            project_item.inner.id as db_ids::ProjectId,
        )
        .execute(&**pool)
        .await?;

        db_models::Project::clear_cache(
            project_item.inner.id,
            project_item.inner.slug,
            None,
            &redis,
        )
        .await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[patch("{id}/icon")]
#[allow(clippy::too_many_arguments)]
pub async fn project_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    if let Some(content_type) = crate::util::ext::get_image_content_type(&ext.ext) {
        let cdn_url = dotenvy::var("CDN_URL")?;
        let user = get_user_from_headers(
            &req,
            &**pool,
            &redis,
            &session_queue,
            Some(&[Scopes::PROJECT_WRITE]),
        )
        .await?
        .1;
        let string = info.into_inner().0;

        let project_item = db_models::Project::get(&string, &**pool, &redis)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput("The specified project does not exist!".to_string())
            })?;

        if !user.role.is_mod() {
            let (team_member, organization_team_member) =
                db_models::TeamMember::get_for_project_permissions(
                    &project_item.inner,
                    user.id.into(),
                    &**pool,
                )
                .await?;

            // Hide the project
            if team_member.is_none() && organization_team_member.is_none() {
                return Err(ApiError::CustomAuthentication(
                    "The specified project does not exist!".to_string(),
                ));
            }

            let permissions = ProjectPermissions::get_permissions_by_role(
                &user.role,
                &team_member,
                &organization_team_member,
            )
            .unwrap_or_default();

            if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to edit this project's icon.".to_string(),
                ));
            }
        }

        if let Some(icon) = project_item.inner.icon_url {
            let name = icon.split(&format!("{cdn_url}/")).nth(1);

            if let Some(icon_path) = name {
                file_host.delete_file_version("", icon_path).await?;
            }
        }

        let bytes =
            read_from_payload(&mut payload, 262144, "Icons must be smaller than 256KiB").await?;

        let color = crate::util::img::get_color_from_img(&bytes)?;

        let hash = sha1::Sha1::from(&bytes).hexdigest();
        let project_id: ProjectId = project_item.inner.id.into();
        let upload_data = file_host
            .upload_file(
                content_type,
                &format!("data/{}/{}.{}", project_id, hash, ext.ext),
                bytes.freeze(),
            )
            .await?;

        let mut transaction = pool.begin().await?;

        sqlx::query!(
            "
            UPDATE mods
            SET icon_url = $1, color = $2
            WHERE (id = $3)
            ",
            format!("{}/{}", cdn_url, upload_data.file_name),
            color.map(|x| x as i32),
            project_item.inner.id as db_ids::ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        db_models::Project::clear_cache(
            project_item.inner.id,
            project_item.inner.slug,
            None,
            &redis,
        )
        .await?;

        transaction.commit().await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::InvalidInput(format!(
            "Invalid format for project icon: {}",
            ext.ext
        )))
    }
}

#[delete("{id}/icon")]
pub async fn delete_project_icon(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;
    let string = info.into_inner().0;

    let project_item = db_models::Project::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("The specified project does not exist!".to_string())
        })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::TeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::CustomAuthentication(
                "The specified project does not exist!".to_string(),
            ));
        }
        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit this project's icon.".to_string(),
            ));
        }
    }

    let cdn_url = dotenvy::var("CDN_URL")?;
    if let Some(icon) = project_item.inner.icon_url {
        let name = icon.split(&format!("{cdn_url}/")).nth(1);

        if let Some(icon_path) = name {
            file_host.delete_file_version("", icon_path).await?;
        }
    }

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        UPDATE mods
        SET icon_url = NULL, color = NULL
        WHERE (id = $1)
        ",
        project_item.inner.id as db_ids::ProjectId,
    )
    .execute(&mut *transaction)
    .await?;

    db_models::Project::clear_cache(project_item.inner.id, project_item.inner.slug, None, &redis)
        .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Serialize, Deserialize, Validate)]
pub struct GalleryCreateQuery {
    pub featured: bool,
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<String>,
    pub ordering: Option<i64>,
}

#[post("{id}/gallery")]
#[allow(clippy::too_many_arguments)]
pub async fn add_gallery_item(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    web::Query(item): web::Query<GalleryCreateQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    if let Some(content_type) = crate::util::ext::get_image_content_type(&ext.ext) {
        item.validate()
            .map_err(|err| ApiError::Validation(validation_errors_to_string(err, None)))?;

        let cdn_url = dotenvy::var("CDN_URL")?;
        let user = get_user_from_headers(
            &req,
            &**pool,
            &redis,
            &session_queue,
            Some(&[Scopes::PROJECT_WRITE]),
        )
        .await?
        .1;
        let string = info.into_inner().0;

        let project_item = db_models::Project::get(&string, &**pool, &redis)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput("The specified project does not exist!".to_string())
            })?;

        if project_item.gallery_items.len() > 64 {
            return Err(ApiError::CustomAuthentication(
                "You have reached the maximum of gallery images to upload.".to_string(),
            ));
        }

        if !user.role.is_admin() {
            let (team_member, organization_team_member) =
                db_models::TeamMember::get_for_project_permissions(
                    &project_item.inner,
                    user.id.into(),
                    &**pool,
                )
                .await?;

            // Hide the project
            if team_member.is_none() && organization_team_member.is_none() {
                return Err(ApiError::CustomAuthentication(
                    "The specified project does not exist!".to_string(),
                ));
            }

            let permissions = ProjectPermissions::get_permissions_by_role(
                &user.role,
                &team_member,
                &organization_team_member,
            )
            .unwrap_or_default();

            if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to edit this project's gallery.".to_string(),
                ));
            }
        }

        let bytes = read_from_payload(
            &mut payload,
            5 * (1 << 20),
            "Gallery image exceeds the maximum of 5MiB.",
        )
        .await?;
        let hash = sha1::Sha1::from(&bytes).hexdigest();

        let id: ProjectId = project_item.inner.id.into();
        let url = format!("data/{}/images/{}.{}", id, hash, &*ext.ext);

        let file_url = format!("{cdn_url}/{url}");
        if project_item
            .gallery_items
            .iter()
            .any(|x| x.image_url == file_url)
        {
            return Err(ApiError::InvalidInput(
                "You may not upload duplicate gallery images!".to_string(),
            ));
        }

        file_host
            .upload_file(content_type, &url, bytes.freeze())
            .await?;

        let mut transaction = pool.begin().await?;

        if item.featured {
            sqlx::query!(
                "
                UPDATE mods_gallery
                SET featured = $2
                WHERE mod_id = $1
                ",
                project_item.inner.id as db_ids::ProjectId,
                false,
            )
            .execute(&mut *transaction)
            .await?;
        }

        let gallery_item = vec![db_models::project_item::GalleryItem {
            image_url: file_url,
            featured: item.featured,
            title: item.title,
            description: item.description,
            created: Utc::now(),
            ordering: item.ordering.unwrap_or(0),
        }];
        GalleryItem::insert_many(gallery_item, project_item.inner.id, &mut transaction).await?;

        db_models::Project::clear_cache(
            project_item.inner.id,
            project_item.inner.slug,
            None,
            &redis,
        )
        .await?;

        transaction.commit().await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::InvalidInput(format!(
            "Invalid format for gallery image: {}",
            ext.ext
        )))
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct GalleryEditQuery {
    /// The url of the gallery item to edit
    pub url: String,
    pub featured: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 255))]
    pub title: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<Option<String>>,
    pub ordering: Option<i64>,
}

#[patch("{id}/gallery")]
pub async fn edit_gallery_item(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryEditQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;
    let string = info.into_inner().0;

    item.validate()
        .map_err(|err| ApiError::Validation(validation_errors_to_string(err, None)))?;

    let project_item = db_models::Project::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("The specified project does not exist!".to_string())
        })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::TeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::CustomAuthentication(
                "The specified project does not exist!".to_string(),
            ));
        }
        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit this project's gallery.".to_string(),
            ));
        }
    }
    let mut transaction = pool.begin().await?;

    let id = sqlx::query!(
        "
        SELECT id FROM mods_gallery
        WHERE image_url = $1
        ",
        item.url
    )
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(format!(
            "Gallery item at URL {} is not part of the project's gallery.",
            item.url
        ))
    })?
    .id;

    let mut transaction = pool.begin().await?;

    if let Some(featured) = item.featured {
        if featured {
            sqlx::query!(
                "
                UPDATE mods_gallery
                SET featured = $2
                WHERE mod_id = $1
                ",
                project_item.inner.id as db_ids::ProjectId,
                false,
            )
            .execute(&mut *transaction)
            .await?;
        }

        sqlx::query!(
            "
            UPDATE mods_gallery
            SET featured = $2
            WHERE id = $1
            ",
            id,
            featured
        )
        .execute(&mut *transaction)
        .await?;
    }
    if let Some(title) = item.title {
        sqlx::query!(
            "
            UPDATE mods_gallery
            SET title = $2
            WHERE id = $1
            ",
            id,
            title
        )
        .execute(&mut *transaction)
        .await?;
    }
    if let Some(description) = item.description {
        sqlx::query!(
            "
            UPDATE mods_gallery
            SET description = $2
            WHERE id = $1
            ",
            id,
            description
        )
        .execute(&mut *transaction)
        .await?;
    }
    if let Some(ordering) = item.ordering {
        sqlx::query!(
            "
            UPDATE mods_gallery
            SET ordering = $2
            WHERE id = $1
            ",
            id,
            ordering
        )
        .execute(&mut *transaction)
        .await?;
    }

    db_models::Project::clear_cache(project_item.inner.id, project_item.inner.slug, None, &redis)
        .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Serialize, Deserialize)]
pub struct GalleryDeleteQuery {
    pub url: String,
}

#[delete("{id}/gallery")]
pub async fn delete_gallery_item(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryDeleteQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;
    let string = info.into_inner().0;

    let project_item = db_models::Project::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("The specified project does not exist!".to_string())
        })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::TeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::CustomAuthentication(
                "The specified project does not exist!".to_string(),
            ));
        }

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit this project's gallery.".to_string(),
            ));
        }
    }
    let mut transaction = pool.begin().await?;

    let id = sqlx::query!(
        "
        SELECT id FROM mods_gallery
        WHERE image_url = $1
        ",
        item.url
    )
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(format!(
            "Gallery item at URL {} is not part of the project's gallery.",
            item.url
        ))
    })?
    .id;

    let cdn_url = dotenvy::var("CDN_URL")?;
    let name = item.url.split(&format!("{cdn_url}/")).nth(1);

    if let Some(icon_path) = name {
        file_host.delete_file_version("", icon_path).await?;
    }

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        DELETE FROM mods_gallery
        WHERE id = $1
        ",
        id
    )
    .execute(&mut *transaction)
    .await?;

    db_models::Project::clear_cache(project_item.inner.id, project_item.inner.slug, None, &redis)
        .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[delete("{id}")]
pub async fn project_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    config: web::Data<SearchConfig>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_DELETE]),
    )
    .await?
    .1;
    let string = info.into_inner().0;

    let project = db_models::Project::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("The specified project does not exist!".to_string())
        })?;

    if !user.role.is_admin() {
        let (team_member, organization_team_member) =
            db_models::TeamMember::get_for_project_permissions(
                &project.inner,
                user.id.into(),
                &**pool,
            )
            .await?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::CustomAuthentication(
                "The specified project does not exist!".to_string(),
            ));
        }

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::DELETE_PROJECT) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to delete this project!".to_string(),
            ));
        }
    }

    let mut transaction = pool.begin().await?;
    let context = ImageContext::Project {
        project_id: Some(project.inner.id.into()),
    };
    let uploaded_images = db_models::Image::get_many_contexted(context, &mut transaction).await?;
    for image in uploaded_images {
        image_item::Image::remove(image.id, &mut transaction, &redis).await?;
    }

    sqlx::query!(
        "
        DELETE FROM collections_mods
        WHERE mod_id = $1
        ",
        project.inner.id as db_ids::ProjectId,
    )
    .execute(&mut *transaction)
    .await?;

    let result = db_models::Project::remove(project.inner.id, &mut transaction, &redis).await?;

    transaction.commit().await?;

    delete_from_index(project.inner.id.into(), config).await?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[post("{id}/follow")]
pub async fn project_follow(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::USER_WRITE]),
    )
    .await?
    .1;
    let string = info.into_inner().0;

    let result = db_models::Project::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("The specified project does not exist!".to_string())
        })?;

    let user_id: db_ids::UserId = user.id.into();
    let project_id: db_ids::ProjectId = result.inner.id;

    if !is_authorized(&result.inner, &Some(user), &pool).await? {
        return Ok(HttpResponse::NotFound().body(""));
    }

    let following = sqlx::query!(
        "
        SELECT EXISTS(SELECT 1 FROM mod_follows mf WHERE mf.follower_id = $1 AND mf.mod_id = $2)
        ",
        user_id as db_ids::UserId,
        project_id as db_ids::ProjectId
    )
    .fetch_one(&**pool)
    .await?
    .exists
    .unwrap_or(false);

    if !following {
        let mut transaction = pool.begin().await?;

        sqlx::query!(
            "
            UPDATE mods
            SET follows = follows + 1
            WHERE id = $1
            ",
            project_id as db_ids::ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            INSERT INTO mod_follows (follower_id, mod_id)
            VALUES ($1, $2)
            ",
            user_id as db_ids::UserId,
            project_id as db_ids::ProjectId
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::InvalidInput(
            "You are already following this project!".to_string(),
        ))
    }
}

#[delete("{id}/follow")]
pub async fn project_unfollow(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::USER_WRITE]),
    )
    .await?
    .1;
    let string = info.into_inner().0;

    let result = db_models::Project::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("The specified project does not exist!".to_string())
        })?;

    let user_id: db_ids::UserId = user.id.into();
    let project_id = result.inner.id;

    let following = sqlx::query!(
        "
        SELECT EXISTS(SELECT 1 FROM mod_follows mf WHERE mf.follower_id = $1 AND mf.mod_id = $2)
        ",
        user_id as db_ids::UserId,
        project_id as db_ids::ProjectId
    )
    .fetch_one(&**pool)
    .await?
    .exists
    .unwrap_or(false);

    if following {
        let mut transaction = pool.begin().await?;

        sqlx::query!(
            "
            UPDATE mods
            SET follows = follows - 1
            WHERE id = $1
            ",
            project_id as db_ids::ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mod_follows
            WHERE follower_id = $1 AND mod_id = $2
            ",
            user_id as db_ids::UserId,
            project_id as db_ids::ProjectId
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::InvalidInput(
            "You are not following this project!".to_string(),
        ))
    }
}

pub async fn delete_from_index(
    id: ProjectId,
    config: web::Data<SearchConfig>,
) -> Result<(), meilisearch_sdk::errors::Error> {
    let client = meilisearch_sdk::client::Client::new(&*config.address, &*config.key);

    let indexes: IndexesResults = client.get_indexes().await?;

    for index in indexes.results {
        index.delete_document(id.to_string()).await?;
    }

    Ok(())
}
