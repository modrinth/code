use std::collections::HashMap;
use std::sync::Arc;

use crate::auth::checks::{filter_visible_versions, is_visible_project};
use crate::auth::{filter_visible_projects, get_user_from_headers};
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::project_item::{DBGalleryItem, DBModCategory};
use crate::database::models::thread_item::ThreadMessageBuilder;
use crate::database::models::{DBTeamMember, ids as db_ids, image_item};
use crate::database::redis::RedisPool;
use crate::database::{self, models as db_models};
use crate::file_hosting::FileHost;
use crate::models;
use crate::models::ids::ProjectId;
use crate::models::images::ImageContext;
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::projects::{
    MonetizationStatus, Project, ProjectStatus, SearchRequest,
};
use crate::models::teams::ProjectPermissions;
use crate::models::threads::MessageBody;
use crate::queue::moderation::AutomatedModerationQueue;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::search::indexing::remove_documents;
use crate::search::{SearchConfig, SearchError, search_for_project};
use crate::util::img;
use crate::util::img::{delete_old_images, upload_image_optimized};
use crate::util::routes::read_from_payload;
use crate::util::validate::validation_errors_to_string;
use actix_web::{HttpRequest, HttpResponse, web};
use ariadne::ids::base62_impl::parse_base62;
use chrono::Utc;
use futures::TryStreamExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("search", web::get().to(project_search));
    cfg.route("projects", web::get().to(projects_get));
    cfg.route("projects", web::patch().to(projects_edit));
    cfg.route("projects_random", web::get().to(random_projects_get));

    cfg.service(
        web::scope("project")
            .route("{id}", web::get().to(project_get))
            .route("{id}/check", web::get().to(project_get_check))
            .route("{id}", web::delete().to(project_delete))
            .route("{id}", web::patch().to(project_edit))
            .route("{id}/icon", web::patch().to(project_icon_edit))
            .route("{id}/icon", web::delete().to(delete_project_icon))
            .route("{id}/gallery", web::post().to(add_gallery_item))
            .route("{id}/gallery", web::patch().to(edit_gallery_item))
            .route("{id}/gallery", web::delete().to(delete_gallery_item))
            .route("{id}/follow", web::post().to(project_follow))
            .route("{id}/follow", web::delete().to(project_unfollow))
            .route("{id}/organization", web::get().to(project_get_organization))
            .service(
                web::scope("{project_id}")
                    .route(
                        "members",
                        web::get().to(super::teams::team_members_get_project),
                    )
                    .route(
                        "version",
                        web::get().to(super::versions::version_list),
                    )
                    .route(
                        "version/{slug}",
                        web::get().to(super::versions::version_project_get),
                    )
                    .route("dependencies", web::get().to(dependency_list)),
            ),
    );
}

#[derive(Deserialize, Validate)]
pub struct RandomProjects {
    #[validate(range(min = 1, max = 100))]
    pub count: u32,
}

pub async fn random_projects_get(
    web::Query(count): web::Query<RandomProjects>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    count.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

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
    .fetch(&**pool)
    .map_ok(|m| db_ids::DBProjectId(m.id))
    .try_collect::<Vec<_>>()
    .await?;

    let projects_data =
        db_models::DBProject::get_many_ids(&project_ids, &**pool, &redis)
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

pub async fn projects_get(
    req: HttpRequest,
    web::Query(ids): web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids = serde_json::from_str::<Vec<&str>>(&ids.ids)?;
    let projects_data =
        db_models::DBProject::get_many(&ids, &**pool, &redis).await?;

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

    let projects =
        filter_visible_projects(projects_data, &user_option, &pool, false)
            .await?;

    Ok(HttpResponse::Ok().json(projects))
}

pub async fn project_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let project_data =
        db_models::DBProject::get(&string, &**pool, &redis).await?;
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
        if is_visible_project(&data.inner, &user_option, &pool, false).await? {
            return Ok(HttpResponse::Ok().json(Project::from(data)));
        }
    }
    Err(ApiError::NotFound)
}

#[derive(Serialize, Deserialize, Validate)]
pub struct EditProject {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub name: Option<String>,
    #[validate(length(min = 3, max = 256))]
    pub summary: Option<String>,
    #[validate(length(max = 65536))]
    pub description: Option<String>,
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
    pub license_url: Option<Option<String>>,
    #[validate(custom(
        function = "crate::util::validate::validate_url_hashmap_optional_values"
    ))]
    // <name, url> (leave url empty to delete)
    pub link_urls: Option<HashMap<String, Option<String>>>,
    pub license_id: Option<String>,
    #[validate(
        length(min = 3, max = 64),
        regex(path = *crate::util::validate::RE_URL_SAFE)
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

#[allow(clippy::too_many_arguments)]
pub async fn project_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    search_config: web::Data<SearchConfig>,
    new_project: web::Json<EditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    moderation_queue: web::Data<AutomatedModerationQueue>,
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

    new_project.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let string = info.into_inner().0;
    let result = db_models::DBProject::get(&string, &**pool, &redis).await?;
    if let Some(project_item) = result {
        let id = project_item.inner.id;

        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
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

            if let Some(name) = &new_project.name {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the name of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET name = $1
                    WHERE (id = $2)
                    ",
                    name.trim(),
                    id as db_ids::DBProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(summary) = &new_project.summary {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the summary of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET summary = $1
                    WHERE (id = $2)
                    ",
                    summary,
                    id as db_ids::DBProjectId,
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
                    || project_item.inner.status.is_approved()
                        && status.can_be_requested())
                {
                    return Err(ApiError::CustomAuthentication(
                        "You don't have permission to set this status!"
                            .to_string(),
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
                        id as db_ids::DBProjectId,
                    )
                    .execute(&mut *transaction)
                    .await?;

                    moderation_queue
                        .projects
                        .insert(project_item.inner.id.into());
                }

                if status.is_approved()
                    && !project_item.inner.status.is_approved()
                {
                    sqlx::query!(
                        "
                        UPDATE mods
                        SET approved = NOW()
                        WHERE id = $1 AND approved IS NULL
                        ",
                        id as db_ids::DBProjectId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
                if status.is_searchable() && !project_item.inner.webhook_sent {
                    if let Ok(webhook_url) =
                        dotenvy::var("PUBLIC_DISCORD_WEBHOOK")
                    {
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
                            id as db_ids::DBProjectId,
                        )
                        .execute(&mut *transaction)
                        .await?;
                    }
                }

                if user.role.is_mod() {
                    if let Ok(webhook_url) =
                        dotenvy::var("MODERATION_SLACK_WEBHOOK")
                    {
                        crate::util::webhook::send_slack_webhook(
                            project_item.inner.id.into(),
                            &pool,
                            &redis,
                            webhook_url,
                            Some(
                                format!(
                                    "*<{}/user/{}|{}>* changed project status from *{}* to *{}*",
                                    dotenvy::var("SITE_URL")?,
                                    user.username,
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
                        project_item.inner.team_id as db_ids::DBTeamId
                    )
                    .fetch(&mut *transaction)
                    .map_ok(|c| db_models::DBUserId(c.id))
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
                    hide_identity: user.role.is_mod(),
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
                    id as db_ids::DBProjectId,
                )
                .execute(&mut *transaction)
                .await?;

                if project_item.inner.status.is_searchable()
                    && !status.is_searchable()
                {
                    remove_documents(
                        &project_item
                            .versions
                            .into_iter()
                            .map(|x| x.into())
                            .collect::<Vec<_>>(),
                        &search_config,
                    )
                    .await?;
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
                    id as db_ids::DBProjectId,
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
                        id as db_ids::DBProjectId,
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
                        id as db_ids::DBProjectId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }

            if let Some(categories) = &new_project.categories {
                edit_project_categories(
                    categories,
                    &perms,
                    id as db_ids::DBProjectId,
                    false,
                    &mut transaction,
                )
                .await?;
            }

            if let Some(categories) = &new_project.additional_categories {
                edit_project_categories(
                    categories,
                    &perms,
                    id as db_ids::DBProjectId,
                    true,
                    &mut transaction,
                )
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
                    id as db_ids::DBProjectId,
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

                let slug_project_id_option: Option<u64> =
                    parse_base62(slug).ok();
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
                            "Slug collides with other project's id!"
                                .to_string(),
                        ));
                    }
                }

                // Make sure the new slug is different from the old one
                // We are able to unwrap here because the slug is always set
                if !slug.eq(&project_item
                    .inner
                    .slug
                    .clone()
                    .unwrap_or_default())
                {
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
                            "Slug collides with other project's id!"
                                .to_string(),
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
                    id as db_ids::DBProjectId,
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
                    ApiError::InvalidInput(format!(
                        "Invalid SPDX license identifier: {err}"
                    ))
                })?;

                sqlx::query!(
                    "
                    UPDATE mods
                    SET license = $1
                    WHERE (id = $2)
                    ",
                    license,
                    id as db_ids::DBProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }
            if let Some(links) = &new_project.link_urls {
                if !links.is_empty() {
                    if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                        return Err(ApiError::CustomAuthentication(
                            "You do not have the permissions to edit the links of this project!"
                                .to_string(),
                        ));
                    }

                    let ids_to_delete =
                        links.keys().cloned().collect::<Vec<String>>();
                    // Deletes all links from hashmap- either will be deleted or be replaced
                    sqlx::query!(
                        "
                        DELETE FROM mods_links
                        WHERE joining_mod_id = $1 AND joining_platform_id IN (
                            SELECT id FROM link_platforms WHERE name = ANY($2)
                        )
                        ",
                        id as db_ids::DBProjectId,
                        &ids_to_delete
                    )
                    .execute(&mut *transaction)
                    .await?;

                    for (platform, url) in links {
                        if let Some(url) = url {
                            let platform_id =
                                db_models::categories::LinkPlatform::get_id(
                                    platform,
                                    &mut *transaction,
                                )
                                .await?
                                .ok_or_else(
                                    || {
                                        ApiError::InvalidInput(format!(
                                            "Platform {} does not exist.",
                                            platform.clone()
                                        ))
                                    },
                                )?;
                            sqlx::query!(
                                "
                                INSERT INTO mods_links (joining_mod_id, joining_platform_id, url)
                                VALUES ($1, $2, $3)
                                ",
                                id as db_ids::DBProjectId,
                                platform_id as db_ids::LinkPlatformId,
                                url
                            )
                            .execute(&mut *transaction)
                            .await?;
                        }
                    }
                }
            }
            if let Some(moderation_message) = &new_project.moderation_message {
                if !user.role.is_mod()
                    && (!project_item.inner.status.is_approved()
                        || moderation_message.is_some())
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
                    id as db_ids::DBProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(moderation_message_body) =
                &new_project.moderation_message_body
            {
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
                    id as db_ids::DBProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(description) = &new_project.description {
                if !perms.contains(ProjectPermissions::EDIT_BODY) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the description (body) of this project!"
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
                    id as db_ids::DBProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(monetization_status) = &new_project.monetization_status
            {
                if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the monetization status of this project!"
                            .to_string(),
                    ));
                }

                if (*monetization_status
                    == MonetizationStatus::ForceDemonetized
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
                    id as db_ids::DBProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            // check new description and body for links to associated images
            // if they no longer exist in the description or body, delete them
            let checkable_strings: Vec<&str> =
                vec![&new_project.description, &new_project.summary]
                    .into_iter()
                    .filter_map(|x| x.as_ref().map(|y| y.as_str()))
                    .collect();

            let context = ImageContext::Project {
                project_id: Some(id.into()),
            };

            img::delete_unused_images(
                context,
                checkable_strings,
                &mut transaction,
                &redis,
            )
            .await?;

            transaction.commit().await?;
            db_models::DBProject::clear_cache(
                project_item.inner.id,
                project_item.inner.slug,
                None,
                &redis,
            )
            .await?;

            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthentication(
                "You do not have permission to edit this project!".to_string(),
            ))
        }
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn edit_project_categories(
    categories: &Vec<String>,
    perms: &ProjectPermissions,
    project_id: db_ids::DBProjectId,
    is_additional: bool,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), ApiError> {
    if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
        let additional_str = if is_additional { "additional " } else { "" };
        return Err(ApiError::CustomAuthentication(format!(
            "You do not have the permissions to edit the {additional_str}categories of this project!"
        )));
    }

    let mut mod_categories = Vec::new();
    for category in categories {
        let category_ids = db_models::categories::Category::get_ids(
            category,
            &mut **transaction,
        )
        .await?;
        // TODO: We should filter out categories that don't match the project type of any of the versions
        // ie: if mod and modpack both share a name this should only have modpack if it only has a modpack as a version

        let mcategories = category_ids
            .values()
            .map(|&category_id| DBModCategory {
                project_id,
                category_id,
                is_additional,
            })
            .collect::<Vec<_>>();
        mod_categories.extend(mcategories);
    }
    DBModCategory::insert_many(mod_categories, &mut *transaction).await?;

    Ok(())
}

// TODO: Re-add this if we want to match v3 Projects structure to v3 Search Result structure, otherwise, delete
// #[derive(Serialize, Deserialize)]
// pub struct ReturnSearchResults {
//     pub hits: Vec<Project>,
//     pub page: usize,
//     pub hits_per_page: usize,
//     pub total_hits: usize,
// }

pub async fn project_search(
    web::Query(info): web::Query<SearchRequest>,
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, SearchError> {
    let results = search_for_project(&info, &config).await?;

    // TODO: add this back
    // let results = ReturnSearchResults {
    //     hits: results
    //         .hits
    //         .into_iter()
    //         .filter_map(Project::from_search)
    //         .collect::<Vec<_>>(),
    //     page: results.page,
    //     hits_per_page: results.hits_per_page,
    //     total_hits: results.total_hits,
    // };

    Ok(HttpResponse::Ok().json(results))
}

//checks the validity of a project id or slug
pub async fn project_get_check(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let slug = info.into_inner().0;

    let project_data =
        db_models::DBProject::get(&slug, &**pool, &redis).await?;

    if let Some(project) = project_data {
        Ok(HttpResponse::Ok().json(json! ({
            "id": models::ids::ProjectId::from(project.inner.id)
        })))
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DependencyInfo {
    pub projects: Vec<Project>,
    pub versions: Vec<models::projects::Version>,
}

pub async fn dependency_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let result = db_models::DBProject::get(&string, &**pool, &redis).await?;

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
        if !is_visible_project(&project.inner, &user_option, &pool, false)
            .await?
        {
            return Err(ApiError::NotFound);
        }

        let dependencies = database::DBProject::get_dependencies(
            project.inner.id,
            &**pool,
            &redis,
        )
        .await?;
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
            .unique()
            .collect::<Vec<_>>();

        let dep_version_ids = dependencies
            .iter()
            .filter_map(|x| x.0)
            .unique()
            .collect::<Vec<db_models::DBVersionId>>();
        let (projects_result, versions_result) = futures::future::try_join(
            database::DBProject::get_many_ids(&project_ids, &**pool, &redis),
            database::DBVersion::get_many(&dep_version_ids, &**pool, &redis),
        )
        .await?;

        let mut projects = filter_visible_projects(
            projects_result,
            &user_option,
            &pool,
            false,
        )
        .await?;
        let mut versions = filter_visible_versions(
            versions_result,
            &user_option,
            &pool,
            &redis,
        )
        .await?;

        projects.sort_by(|a, b| b.published.cmp(&a.published));
        projects.dedup_by(|a, b| a.id == b.id);

        versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));
        versions.dedup_by(|a, b| a.id == b.id);

        Ok(HttpResponse::Ok().json(DependencyInfo { projects, versions }))
    } else {
        Err(ApiError::NotFound)
    }
}

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

    #[validate(custom(
        function = " crate::util::validate::validate_url_hashmap_optional_values"
    ))]
    pub link_urls: Option<HashMap<String, Option<String>>>,
}

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

    bulk_edit_project.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let project_ids: Vec<db_ids::DBProjectId> =
        serde_json::from_str::<Vec<ProjectId>>(&ids.ids)?
            .into_iter()
            .map(|x| x.into())
            .collect();

    let projects_data =
        db_models::DBProject::get_many_ids(&project_ids, &**pool, &redis)
            .await?;

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
        .collect::<Vec<db_models::DBTeamId>>();
    let team_members = db_models::DBTeamMember::get_from_team_full_many(
        &team_ids, &**pool, &redis,
    )
    .await?;

    let organization_ids = projects_data
        .iter()
        .filter_map(|x| x.inner.organization_id)
        .collect::<Vec<db_models::DBOrganizationId>>();
    let organizations = db_models::DBOrganization::get_many_ids(
        &organization_ids,
        &**pool,
        &redis,
    )
    .await?;

    let organization_team_ids = organizations
        .iter()
        .map(|x| x.team_id)
        .collect::<Vec<db_models::DBTeamId>>();
    let organization_team_members =
        db_models::DBTeamMember::get_from_team_full_many(
            &organization_team_ids,
            &**pool,
            &redis,
        )
        .await?;

    let categories =
        db_models::categories::Category::list(&**pool, &redis).await?;
    let link_platforms =
        db_models::categories::LinkPlatform::list(&**pool, &redis).await?;

    let mut transaction = pool.begin().await?;

    for project in projects_data {
        if !user.role.is_mod() {
            let team_member = team_members.iter().find(|x| {
                x.team_id == project.inner.team_id
                    && x.user_id == user.id.into()
            });

            let organization = project
                .inner
                .organization_id
                .and_then(|oid| organizations.iter().find(|x| x.id == oid));

            let organization_team_member =
                if let Some(organization) = organization {
                    organization_team_members.iter().find(|x| {
                        x.team_id == organization.team_id
                            && x.user_id == user.id.into()
                    })
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
                        project.inner.name
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
                    project.inner.name
                )));
            };
        }

        bulk_edit_project_categories(
            &categories,
            &project.categories,
            project.inner.id as db_ids::DBProjectId,
            CategoryChanges {
                categories: &bulk_edit_project.categories,
                add_categories: &bulk_edit_project.add_categories,
                remove_categories: &bulk_edit_project.remove_categories,
            },
            3,
            false,
            &mut transaction,
        )
        .await?;

        bulk_edit_project_categories(
            &categories,
            &project.additional_categories,
            project.inner.id as db_ids::DBProjectId,
            CategoryChanges {
                categories: &bulk_edit_project.additional_categories,
                add_categories: &bulk_edit_project.add_additional_categories,
                remove_categories: &bulk_edit_project
                    .remove_additional_categories,
            },
            256,
            true,
            &mut transaction,
        )
        .await?;

        if let Some(links) = &bulk_edit_project.link_urls {
            let ids_to_delete = links.keys().cloned().collect::<Vec<String>>();
            // Deletes all links from hashmap- either will be deleted or be replaced
            sqlx::query!(
                "
                DELETE FROM mods_links
                WHERE joining_mod_id = $1 AND joining_platform_id IN (
                    SELECT id FROM link_platforms WHERE name = ANY($2)
                )
                ",
                project.inner.id as db_ids::DBProjectId,
                &ids_to_delete
            )
            .execute(&mut *transaction)
            .await?;

            for (platform, url) in links {
                if let Some(url) = url {
                    let platform_id = link_platforms
                        .iter()
                        .find(|x| &x.name == platform)
                        .ok_or_else(|| {
                            ApiError::InvalidInput(format!(
                                "Platform {} does not exist.",
                                platform.clone()
                            ))
                        })?
                        .id;
                    sqlx::query!(
                        "
                        INSERT INTO mods_links (joining_mod_id, joining_platform_id, url)
                        VALUES ($1, $2, $3)
                        ",
                        project.inner.id as db_ids::DBProjectId,
                        platform_id as db_ids::LinkPlatformId,
                        url
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }
        }

        db_models::DBProject::clear_cache(
            project.inner.id,
            project.inner.slug,
            None,
            &redis,
        )
        .await?;
    }

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

pub async fn bulk_edit_project_categories(
    all_db_categories: &[db_models::categories::Category],
    project_categories: &Vec<String>,
    project_id: db_ids::DBProjectId,
    bulk_changes: CategoryChanges<'_>,
    max_num_categories: usize,
    is_additional: bool,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), ApiError> {
    let mut set_categories =
        if let Some(categories) = bulk_changes.categories.clone() {
            categories
        } else {
            project_categories.clone()
        };

    if let Some(delete_categories) = &bulk_changes.remove_categories {
        for category in delete_categories {
            if let Some(pos) = set_categories.iter().position(|x| x == category)
            {
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
            project_id as db_ids::DBProjectId,
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
                    ApiError::InvalidInput(format!(
                        "Category {} does not exist.",
                        category.clone()
                    ))
                })?
                .id;
            mod_categories.push(DBModCategory {
                project_id,
                category_id,
                is_additional,
            });
        }
        DBModCategory::insert_many(mod_categories, &mut *transaction).await?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

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

    let project_item = db_models::DBProject::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
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
                "You don't have permission to edit this project's icon."
                    .to_string(),
            ));
        }
    }

    delete_old_images(
        project_item.inner.icon_url,
        project_item.inner.raw_icon_url,
        &***file_host,
    )
    .await?;

    let bytes = read_from_payload(
        &mut payload,
        262144,
        "Icons must be smaller than 256KiB",
    )
    .await?;

    let project_id: ProjectId = project_item.inner.id.into();
    let upload_result = upload_image_optimized(
        &format!("data/{project_id}"),
        bytes.freeze(),
        &ext.ext,
        Some(96),
        Some(1.0),
        &***file_host,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
            UPDATE mods
            SET icon_url = $1, raw_icon_url = $2, color = $3
            WHERE (id = $4)
            ",
        upload_result.url,
        upload_result.raw_url,
        upload_result.color.map(|x| x as i32),
        project_item.inner.id as db_ids::DBProjectId,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    db_models::DBProject::clear_cache(
        project_item.inner.id,
        project_item.inner.slug,
        None,
        &redis,
    )
    .await?;

    Ok(HttpResponse::NoContent().body(""))
}

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

    let project_item = db_models::DBProject::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
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
                "You don't have permission to edit this project's icon."
                    .to_string(),
            ));
        }
    }

    delete_old_images(
        project_item.inner.icon_url,
        project_item.inner.raw_icon_url,
        &***file_host,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        UPDATE mods
        SET icon_url = NULL, raw_icon_url = NULL, color = NULL
        WHERE (id = $1)
        ",
        project_item.inner.id as db_ids::DBProjectId,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    db_models::DBProject::clear_cache(
        project_item.inner.id,
        project_item.inner.slug,
        None,
        &redis,
    )
    .await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Serialize, Deserialize, Validate)]
pub struct GalleryCreateQuery {
    pub featured: bool,
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<String>,
    pub ordering: Option<i64>,
}

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
    item.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

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

    let project_item = db_models::DBProject::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

    if project_item.gallery_items.len() > 64 {
        return Err(ApiError::CustomAuthentication(
            "You have reached the maximum of gallery images to upload."
                .to_string(),
        ));
    }

    if !user.role.is_admin() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
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
                "You don't have permission to edit this project's gallery."
                    .to_string(),
            ));
        }
    }

    let bytes = read_from_payload(
        &mut payload,
        2 * (1 << 20),
        "Gallery image exceeds the maximum of 2MiB.",
    )
    .await?;

    let id: ProjectId = project_item.inner.id.into();
    let upload_result = upload_image_optimized(
        &format!("data/{id}/images"),
        bytes.freeze(),
        &ext.ext,
        Some(350),
        Some(1.0),
        &***file_host,
    )
    .await?;

    if project_item
        .gallery_items
        .iter()
        .any(|x| x.image_url == upload_result.url)
    {
        return Err(ApiError::InvalidInput(
            "You may not upload duplicate gallery images!".to_string(),
        ));
    }

    let mut transaction = pool.begin().await?;

    if item.featured {
        sqlx::query!(
            "
                UPDATE mods_gallery
                SET featured = $2
                WHERE mod_id = $1
                ",
            project_item.inner.id as db_ids::DBProjectId,
            false,
        )
        .execute(&mut *transaction)
        .await?;
    }

    let gallery_item = vec![db_models::project_item::DBGalleryItem {
        image_url: upload_result.url,
        raw_image_url: upload_result.raw_url,
        featured: item.featured,
        name: item.name,
        description: item.description,
        created: Utc::now(),
        ordering: item.ordering.unwrap_or(0),
    }];
    DBGalleryItem::insert_many(
        gallery_item,
        project_item.inner.id,
        &mut transaction,
    )
    .await?;

    transaction.commit().await?;
    db_models::DBProject::clear_cache(
        project_item.inner.id,
        project_item.inner.slug,
        None,
        &redis,
    )
    .await?;

    Ok(HttpResponse::NoContent().body(""))
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
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<Option<String>>,
    pub ordering: Option<i64>,
}

pub async fn edit_gallery_item(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryEditQuery>,
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

    item.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let result = sqlx::query!(
        "
        SELECT id, mod_id FROM mods_gallery
        WHERE image_url = $1
        ",
        item.url
    )
    .fetch_optional(&**pool)
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(format!(
            "Gallery item at URL {} is not part of the project's gallery.",
            item.url
        ))
    })?;

    let project_item = db_models::DBProject::get_id(
        database::models::DBProjectId(result.mod_id),
        &**pool,
        &redis,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "The specified project does not exist!".to_string(),
        )
    })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
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
                "You don't have permission to edit this project's gallery."
                    .to_string(),
            ));
        }
    }

    let mut transaction = pool.begin().await?;

    if let Some(featured) = item.featured {
        if featured {
            sqlx::query!(
                "
                UPDATE mods_gallery
                SET featured = $2
                WHERE mod_id = $1
                ",
                project_item.inner.id as db_ids::DBProjectId,
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
            result.id,
            featured
        )
        .execute(&mut *transaction)
        .await?;
    }
    if let Some(name) = item.name {
        sqlx::query!(
            "
            UPDATE mods_gallery
            SET name = $2
            WHERE id = $1
            ",
            result.id,
            name
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
            result.id,
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
            result.id,
            ordering
        )
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    db_models::DBProject::clear_cache(
        project_item.inner.id,
        project_item.inner.slug,
        None,
        &redis,
    )
    .await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Serialize, Deserialize)]
pub struct GalleryDeleteQuery {
    pub url: String,
}

pub async fn delete_gallery_item(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryDeleteQuery>,
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

    let item = sqlx::query!(
        "
        SELECT id, image_url, raw_image_url, mod_id FROM mods_gallery
        WHERE image_url = $1
        ",
        item.url
    )
    .fetch_optional(&**pool)
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(format!(
            "Gallery item at URL {} is not part of the project's gallery.",
            item.url
        ))
    })?;

    let project_item = db_models::DBProject::get_id(
        database::models::DBProjectId(item.mod_id),
        &**pool,
        &redis,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "The specified project does not exist!".to_string(),
        )
    })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
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
                "You don't have permission to edit this project's gallery."
                    .to_string(),
            ));
        }
    }

    delete_old_images(
        Some(item.image_url),
        Some(item.raw_image_url),
        &***file_host,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        DELETE FROM mods_gallery
        WHERE id = $1
        ",
        item.id
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    db_models::DBProject::clear_cache(
        project_item.inner.id,
        project_item.inner.slug,
        None,
        &redis,
    )
    .await?;

    Ok(HttpResponse::NoContent().body(""))
}

pub async fn project_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    search_config: web::Data<SearchConfig>,
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

    let project = db_models::DBProject::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

    if !user.role.is_admin() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
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
    let uploaded_images =
        db_models::DBImage::get_many_contexted(context, &mut transaction)
            .await?;
    for image in uploaded_images {
        image_item::DBImage::remove(image.id, &mut transaction, &redis).await?;
    }

    sqlx::query!(
        "
        DELETE FROM collections_mods
        WHERE mod_id = $1
        ",
        project.inner.id as db_ids::DBProjectId,
    )
    .execute(&mut *transaction)
    .await?;

    let result = db_models::DBProject::remove(
        project.inner.id,
        &mut transaction,
        &redis,
    )
    .await?;

    transaction.commit().await?;

    remove_documents(
        &project
            .versions
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<_>>(),
        &search_config,
    )
    .await?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}

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

    let result = db_models::DBProject::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

    let user_id: db_ids::DBUserId = user.id.into();
    let project_id: db_ids::DBProjectId = result.inner.id;

    if !is_visible_project(&result.inner, &Some(user), &pool, false).await? {
        return Err(ApiError::NotFound);
    }

    let following = sqlx::query!(
        "
        SELECT EXISTS(SELECT 1 FROM mod_follows mf WHERE mf.follower_id = $1 AND mf.mod_id = $2)
        ",
        user_id as db_ids::DBUserId,
        project_id as db_ids::DBProjectId
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
            project_id as db_ids::DBProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            INSERT INTO mod_follows (follower_id, mod_id)
            VALUES ($1, $2)
            ",
            user_id as db_ids::DBUserId,
            project_id as db_ids::DBProjectId
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

    let result = db_models::DBProject::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

    let user_id: db_ids::DBUserId = user.id.into();
    let project_id = result.inner.id;

    let following = sqlx::query!(
        "
        SELECT EXISTS(SELECT 1 FROM mod_follows mf WHERE mf.follower_id = $1 AND mf.mod_id = $2)
        ",
        user_id as db_ids::DBUserId,
        project_id as db_ids::DBProjectId
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
            project_id as db_ids::DBProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mod_follows
            WHERE follower_id = $1 AND mod_id = $2
            ",
            user_id as db_ids::DBUserId,
            project_id as db_ids::DBProjectId
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

pub async fn project_get_organization(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ, Scopes::ORGANIZATION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();
    let user_id = current_user.as_ref().map(|x| x.id.into());

    let string = info.into_inner().0;
    let result = db_models::DBProject::get(&string, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

    if !is_visible_project(&result.inner, &current_user, &pool, false).await? {
        Err(ApiError::InvalidInput(
            "The specified project does not exist!".to_string(),
        ))
    } else if let Some(organization_id) = result.inner.organization_id {
        let organization =
            db_models::DBOrganization::get_id(organization_id, &**pool, &redis)
                .await?
                .ok_or_else(|| {
                    ApiError::InvalidInput(
                        "The attached organization does not exist!".to_string(),
                    )
                })?;

        let members_data = DBTeamMember::get_from_team_full(
            organization.team_id,
            &**pool,
            &redis,
        )
        .await?;

        let users = crate::database::models::DBUser::get_many_ids(
            &members_data.iter().map(|x| x.user_id).collect::<Vec<_>>(),
            &**pool,
            &redis,
        )
        .await?;
        let logged_in = current_user
            .as_ref()
            .and_then(|user| {
                members_data
                    .iter()
                    .find(|x| x.user_id == user.id.into() && x.accepted)
            })
            .is_some();
        let team_members: Vec<_> = members_data
            .into_iter()
            .filter(|x| {
                logged_in
                    || x.accepted
                    || user_id
                        .map(|y: crate::database::models::DBUserId| {
                            y == x.user_id
                        })
                        .unwrap_or(false)
            })
            .flat_map(|data| {
                users.iter().find(|x| x.id == data.user_id).map(|user| {
                    crate::models::teams::TeamMember::from(
                        data,
                        user.clone(),
                        !logged_in,
                    )
                })
            })
            .collect();

        let organization = models::organizations::Organization::from(
            organization,
            team_members,
        );
        return Ok(HttpResponse::Ok().json(organization));
    } else {
        Err(ApiError::NotFound)
    }
}
