use crate::database;
use crate::file_hosting::FileHost;
use crate::models;
use crate::models::projects::{
    DonationLink, Project, ProjectId, ProjectStatus, SearchRequest, SideType,
};
use crate::models::teams::Permissions;
use crate::routes::ApiError;
use crate::search::{search_for_project, SearchConfig, SearchError};
use crate::util::auth::{get_user_from_headers, is_authorized};
use crate::util::routes::read_from_payload;
use crate::util::validate::validation_errors_to_string;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use time::OffsetDateTime;
use validator::Validate;

#[get("search")]
pub async fn project_search(
    web::Query(info): web::Query<SearchRequest>,
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, SearchError> {
    let results = search_for_project(&info, &**config).await?;
    Ok(HttpResponse::Ok().json(results))
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
) -> Result<HttpResponse, ApiError> {
    let project_ids =
        serde_json::from_str::<Vec<ProjectId>>(&*ids.ids)?
            .into_iter()
            .map(|x| x.into())
            .collect();

    let projects_data =
        database::models::Project::get_many_full(project_ids, &**pool).await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    let projects: Vec<_> = futures::stream::iter(projects_data)
        .filter_map(|data| async {
            if is_authorized(&data, &user_option, &pool).await.ok()? {
                Some(Project::from(data))
            } else {
                None
            }
        })
        .collect()
        .await;

    Ok(HttpResponse::Ok().json(projects))
}

#[get("{id}")]
pub async fn project_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let project_data =
        database::models::Project::get_full_from_slug_or_project_id(
            &string, &**pool,
        )
        .await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(data) = project_data {
        if is_authorized(&data, &user_option, &pool).await? {
            return Ok(HttpResponse::Ok().json(Project::from(data)));
        }
    }
    Ok(HttpResponse::NotFound().body(""))
}

#[derive(Serialize)]
struct DependencyInfo {
    pub projects: Vec<models::projects::Project>,
    pub versions: Vec<models::projects::Version>,
}

#[get("dependencies")]
pub async fn dependency_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let result = database::models::Project::get_full_from_slug_or_project_id(
        &string, &**pool,
    )
    .await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(project) = result {
        if !is_authorized(&project, &user_option, &pool).await? {
            return Ok(HttpResponse::NotFound().body(""));
        }

        let id = project.inner.id;

        use futures::stream::TryStreamExt;

        //TODO: This query is not checked at compile time! Once SQLX parses this query correctly, please use the query! macro instead
        let dependencies = sqlx::query(
            "
            SELECT d.dependency_id, vd.mod_id, d.mod_dependency_id
            FROM versions v
            INNER JOIN dependencies d ON d.dependent_id = v.id
            LEFT JOIN versions vd ON d.dependency_id = vd.id
            WHERE v.mod_id = $1
            ",
        )
        .bind(id as database::models::ProjectId)
        .fetch_many(&**pool)
        .try_filter_map(|e| async {
            Ok(e.right().map(|x| {
                (
                    x.get::<Option<i64>, usize>(0)
                        .map(database::models::VersionId),
                    x.get::<Option<i64>, usize>(1)
                        .map(database::models::ProjectId),
                    x.get::<Option<i64>, usize>(2)
                        .map(database::models::ProjectId),
                )
            }))
        })
        .try_collect::<Vec<(
            Option<database::models::VersionId>,
            Option<database::models::ProjectId>,
            Option<database::models::ProjectId>,
        )>>()
        .await?;

        let (projects_result, versions_result) = futures::join!(
            database::Project::get_many_full(
                dependencies
                    .iter()
                    .filter_map(|x| if x.0.is_none() {
                        if let Some(mod_dependency_id) = x.2 {
                            Some(mod_dependency_id)
                        } else {
                            x.1
                        }
                    } else {
                        x.1
                    })
                    .collect(),
                &**pool,
            ),
            database::Version::get_many_full(
                dependencies.iter().filter_map(|x| x.0).collect(),
                &**pool,
            )
        );

        let mut projects = projects_result?
            .into_iter()
            .map(models::projects::Project::from)
            .collect::<Vec<_>>();
        let mut versions = versions_result?
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

/// A project returned from the API
#[derive(Serialize, Deserialize, Validate)]
pub struct EditProject {
    #[validate(length(min = 3, max = 256))]
    pub title: Option<String>,
    #[validate(length(min = 3, max = 2048))]
    pub description: Option<String>,
    #[validate(length(max = 65536))]
    pub body: Option<String>,
    #[validate(length(max = 3))]
    pub categories: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(url, length(max = 2048))]
    pub issues_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(url, length(max = 2048))]
    pub source_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(url, length(max = 2048))]
    pub wiki_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(url, length(max = 2048))]
    pub license_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(url, length(max = 2048))]
    pub discord_url: Option<Option<String>>,
    #[validate]
    pub donation_urls: Option<Vec<DonationLink>>,
    pub license_id: Option<String>,
    pub client_side: Option<SideType>,
    pub server_side: Option<SideType>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        length(min = 3, max = 64),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    pub slug: Option<Option<String>>,
    pub status: Option<ProjectStatus>,
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
}

#[patch("{id}")]
pub async fn project_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    config: web::Data<SearchConfig>,
    new_project: web::Json<EditProject>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    new_project.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let string = info.into_inner().0;
    let result = database::models::Project::get_full_from_slug_or_project_id(
        &string, &**pool,
    )
    .await?;

    if let Some(project_item) = result {
        let id = project_item.inner.id;

        let team_member = database::models::TeamMember::get_from_user_id(
            project_item.inner.team_id,
            user.id.into(),
            &**pool,
        )
        .await?;
        let permissions;

        if let Some(member) = team_member {
            permissions = Some(member.permissions)
        } else if user.role.is_mod() {
            permissions = Some(Permissions::ALL)
        } else {
            permissions = None
        }

        if let Some(perms) = permissions {
            let mut transaction = pool.begin().await?;

            if let Some(title) = &new_project.title {
                if !perms.contains(Permissions::EDIT_DETAILS) {
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
                    title,
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(description) = &new_project.description {
                if !perms.contains(Permissions::EDIT_DETAILS) {
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(status) = &new_project.status {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the status of this project!"
                            .to_string(),
                    ));
                }

                if (status == &ProjectStatus::Rejected
                    || status == &ProjectStatus::Approved)
                    && !user.role.is_mod()
                {
                    return Err(ApiError::CustomAuthentication(
                        "You don't have permission to set this status"
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
                        SET moderation_message = NULL
                        WHERE (id = $1)
                        ",
                        id as database::models::ids::ProjectId,
                    )
                    .execute(&mut *transaction)
                    .await?;

                    sqlx::query!(
                        "
                        UPDATE mods
                        SET moderation_message_body = NULL
                        WHERE (id = $1)
                        ",
                        id as database::models::ids::ProjectId,
                    )
                    .execute(&mut *transaction)
                    .await?;

                    if let Ok(webhook_url) =
                        dotenv::var("MODERATION_DISCORD_WEBHOOK")
                    {
                        crate::util::webhook::send_discord_webhook(
                            Project::from(project_item.clone()),
                            webhook_url,
                        )
                        .await
                        .ok();
                    }
                }

                let status_id = database::models::StatusId::get_id(
                    status,
                    &mut *transaction,
                )
                .await?
                .ok_or_else(|| {
                    ApiError::InvalidInput(
                        "No database entry for status provided.".to_string(),
                    )
                })?;

                sqlx::query!(
                    "
                    UPDATE mods
                    SET status = $1
                    WHERE (id = $2)
                    ",
                    status_id as database::models::ids::StatusId,
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;

                if project_item.status.is_searchable()
                    && !status.is_searchable()
                {
                    delete_from_index(id.into(), config).await?;
                }
            }

            if let Some(categories) = &new_project.categories {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the categories of this project!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    DELETE FROM mods_categories
                    WHERE joining_mod_id = $1
                    ",
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;

                for category in categories {
                    let category_id =
                        database::models::categories::Category::get_id(
                            category,
                            &mut *transaction,
                        )
                        .await?
                        .ok_or_else(|| {
                            ApiError::InvalidInput(format!(
                                "Category {} does not exist.",
                                category.clone()
                            ))
                        })?;

                    sqlx::query!(
                        "
                        INSERT INTO mods_categories (joining_mod_id, joining_category_id)
                        VALUES ($1, $2)
                        ",
                        id as database::models::ids::ProjectId,
                        category_id as database::models::ids::CategoryId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }

            if let Some(issues_url) = &new_project.issues_url {
                if !perms.contains(Permissions::EDIT_DETAILS) {
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(source_url) = &new_project.source_url {
                if !perms.contains(Permissions::EDIT_DETAILS) {
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(wiki_url) = &new_project.wiki_url {
                if !perms.contains(Permissions::EDIT_DETAILS) {
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(license_url) = &new_project.license_url {
                if !perms.contains(Permissions::EDIT_DETAILS) {
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(discord_url) = &new_project.discord_url {
                if !perms.contains(Permissions::EDIT_DETAILS) {
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(slug) = &new_project.slug {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the slug of this project!"
                            .to_string(),
                    ));
                }

                if let Some(slug) = slug {
                    let slug_project_id_option: Option<ProjectId> =
                        serde_json::from_str(&*format!("\"{}\"", slug)).ok();
                    if let Some(slug_project_id) = slug_project_id_option {
                        let slug_project_id: database::models::ids::ProjectId =
                            slug_project_id.into();
                        let results = sqlx::query!(
                            "
                            SELECT EXISTS(SELECT 1 FROM mods WHERE id=$1)
                            ",
                            slug_project_id as database::models::ids::ProjectId
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
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET slug = LOWER($1)
                    WHERE (id = $2)
                    ",
                    slug.as_deref(),
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(new_side) = &new_project.client_side {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the side type of this mod!"
                            .to_string(),
                    ));
                }

                let side_type_id = database::models::SideTypeId::get_id(
                    new_side,
                    &mut *transaction,
                )
                .await?
                .expect("No database entry found for side type");

                sqlx::query!(
                    "
                    UPDATE mods
                    SET client_side = $1
                    WHERE (id = $2)
                    ",
                    side_type_id as database::models::SideTypeId,
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(new_side) = &new_project.server_side {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the side type of this project!"
                            .to_string(),
                    ));
                }

                let side_type_id = database::models::SideTypeId::get_id(
                    new_side,
                    &mut *transaction,
                )
                .await?
                .expect("No database entry found for side type");

                sqlx::query!(
                    "
                    UPDATE mods
                    SET server_side = $1
                    WHERE (id = $2)
                    ",
                    side_type_id as database::models::SideTypeId,
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(license) = &new_project.license_id {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the license of this project!"
                            .to_string(),
                    ));
                }

                let license_id = database::models::categories::License::get_id(
                    license,
                    &mut *transaction,
                )
                .await?
                .expect("No database entry found for license");

                sqlx::query!(
                    "
                    UPDATE mods
                    SET license = $1
                    WHERE (id = $2)
                    ",
                    license_id as database::models::LicenseId,
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(donations) = &new_project.donation_urls {
                if !perms.contains(Permissions::EDIT_DETAILS) {
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;

                for donation in donations {
                    let platform_id =
                        database::models::DonationPlatformId::get_id(
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
                        id as database::models::ids::ProjectId,
                        platform_id as database::models::ids::DonationPlatformId,
                        donation.url
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }

            if let Some(moderation_message) = &new_project.moderation_message {
                if !user.role.is_mod()
                    && project_item.status != ProjectStatus::Approved
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(moderation_message_body) =
                &new_project.moderation_message_body
            {
                if !user.role.is_mod()
                    && project_item.status != ProjectStatus::Approved
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(body) = &new_project.body {
                if !perms.contains(Permissions::EDIT_BODY) {
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
                    id as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

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

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[patch("{id}/icon")]
pub async fn project_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
) -> Result<HttpResponse, ApiError> {
    if let Some(content_type) =
        crate::util::ext::get_image_content_type(&*ext.ext)
    {
        let cdn_url = dotenv::var("CDN_URL")?;
        let user = get_user_from_headers(req.headers(), &**pool).await?;
        let string = info.into_inner().0;

        let project_item =
            database::models::Project::get_from_slug_or_project_id(
                string.clone(),
                &**pool,
            )
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified project does not exist!".to_string(),
                )
            })?;

        if !user.role.is_mod() {
            let team_member = database::models::TeamMember::get_from_user_id(
                project_item.team_id,
                user.id.into(),
                &**pool,
            )
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified project does not exist!".to_string(),
                )
            })?;

            if !team_member.permissions.contains(Permissions::EDIT_DETAILS) {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to edit this project's icon."
                        .to_string(),
                ));
            }
        }

        if let Some(icon) = project_item.icon_url {
            let name = icon.split('/').next();

            if let Some(icon_path) = name {
                file_host.delete_file_version("", icon_path).await?;
            }
        }

        let bytes = read_from_payload(
            &mut payload,
            262144,
            "Icons must be smaller than 256KiB",
        )
        .await?;
        let hash = sha1::Sha1::from(&bytes).hexdigest();
        let project_id: ProjectId = project_item.id.into();
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
            SET icon_url = $1
            WHERE (id = $2)
            ",
            format!("{}/{}", cdn_url, upload_data.file_name),
            project_item.id as database::models::ids::ProjectId,
        )
        .execute(&mut *transaction)
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
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let string = info.into_inner().0;

    let project_item = database::models::Project::get_from_slug_or_project_id(
        string.clone(),
        &**pool,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "The specified project does not exist!".to_string(),
        )
    })?;

    if !user.role.is_mod() {
        let team_member = database::models::TeamMember::get_from_user_id(
            project_item.team_id,
            user.id.into(),
            &**pool,
        )
        .await
        .map_err(ApiError::Database)?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

        if !team_member.permissions.contains(Permissions::EDIT_DETAILS) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit this project's icon."
                    .to_string(),
            ));
        }
    }

    if let Some(icon) = project_item.icon_url {
        let name = icon.split('/').next();

        if let Some(icon_path) = name {
            file_host.delete_file_version("", icon_path).await?;
        }
    }

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        UPDATE mods
        SET icon_url = NULL
        WHERE (id = $1)
        ",
        project_item.id as database::models::ids::ProjectId,
    )
    .execute(&mut *transaction)
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
}

#[post("{id}/gallery")]
pub async fn add_gallery_item(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    web::Query(item): web::Query<GalleryCreateQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
) -> Result<HttpResponse, ApiError> {
    if let Some(content_type) =
        crate::util::ext::get_image_content_type(&*ext.ext)
    {
        item.validate().map_err(|err| {
            ApiError::Validation(validation_errors_to_string(err, None))
        })?;

        let cdn_url = dotenv::var("CDN_URL")?;
        let user = get_user_from_headers(req.headers(), &**pool).await?;
        let string = info.into_inner().0;

        let project_item =
            database::models::Project::get_from_slug_or_project_id(
                string.clone(),
                &**pool,
            )
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified project does not exist!".to_string(),
                )
            })?;

        if !user.role.is_mod() {
            let team_member = database::models::TeamMember::get_from_user_id(
                project_item.team_id,
                user.id.into(),
                &**pool,
            )
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified project does not exist!".to_string(),
                )
            })?;

            if !team_member.permissions.contains(Permissions::EDIT_DETAILS) {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to edit this project's gallery."
                        .to_string(),
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

        let id: ProjectId = project_item.id.into();
        let url = format!("data/{}/images/{}.{}", id, hash, &*ext.ext);
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
                project_item.id as database::models::ids::ProjectId,
                false,
            )
            .execute(&mut *transaction)
            .await?;
        }

        database::models::project_item::GalleryItem {
            project_id: project_item.id,
            image_url: format!("{}/{}", cdn_url, url),
            featured: item.featured,
            title: item.title,
            description: item.description,
            created: OffsetDateTime::now_utc(),
        }
        .insert(&mut transaction)
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
}

#[patch("{id}/gallery")]
pub async fn edit_gallery_item(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryEditQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let string = info.into_inner().0;

    item.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let project_item = database::models::Project::get_from_slug_or_project_id(
        string.clone(),
        &**pool,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "The specified project does not exist!".to_string(),
        )
    })?;

    if !user.role.is_mod() {
        let team_member = database::models::TeamMember::get_from_user_id(
            project_item.team_id,
            user.id.into(),
            &**pool,
        )
        .await
        .map_err(ApiError::Database)?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

        if !team_member.permissions.contains(Permissions::EDIT_DETAILS) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit this project's gallery."
                    .to_string(),
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
                project_item.id as database::models::ids::ProjectId,
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
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let string = info.into_inner().0;

    let project_item = database::models::Project::get_from_slug_or_project_id(
        string.clone(),
        &**pool,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "The specified project does not exist!".to_string(),
        )
    })?;

    if !user.role.is_mod() {
        let team_member = database::models::TeamMember::get_from_user_id(
            project_item.team_id,
            user.id.into(),
            &**pool,
        )
        .await
        .map_err(ApiError::Database)?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified project does not exist!".to_string(),
            )
        })?;

        if !team_member.permissions.contains(Permissions::EDIT_DETAILS) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit this project's gallery."
                    .to_string(),
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

    let name = item.url.split('/').next();

    if let Some(item_path) = name {
        file_host.delete_file_version("", item_path).await?;
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

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[delete("{id}")]
pub async fn project_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let string = info.into_inner().0;

    let project = database::models::Project::get_from_slug_or_project_id(
        string.clone(),
        &**pool,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "The specified project does not exist!".to_string(),
        )
    })?;

    if !user.role.is_mod() {
        let team_member =
            database::models::TeamMember::get_from_user_id_project(
                project.id,
                user.id.into(),
                &**pool,
            )
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified project does not exist!".to_string(),
                )
            })?;

        if !team_member
            .permissions
            .contains(Permissions::DELETE_PROJECT)
        {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to delete this project!".to_string(),
            ));
        }
    }

    let mut transaction = pool.begin().await?;

    let result =
        database::models::Project::remove_full(project.id, &mut transaction)
            .await?;

    transaction.commit().await?;

    delete_from_index(project.id.into(), config).await?;

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
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let string = info.into_inner().0;

    let result =
        database::models::Project::get_from_slug_or_project_id(string, &**pool)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified project does not exist!".to_string(),
                )
            })?;

    let user_id: database::models::ids::UserId = user.id.into();
    let project_id: database::models::ids::ProjectId = result.id;

    let following = sqlx::query!(
        "
        SELECT EXISTS(SELECT 1 FROM mod_follows mf WHERE mf.follower_id = $1 AND mf.mod_id = $2)
        ",
        user_id as database::models::ids::UserId,
        project_id as database::models::ids::ProjectId
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
            project_id as database::models::ids::ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            INSERT INTO mod_follows (follower_id, mod_id)
            VALUES ($1, $2)
            ",
            user_id as database::models::ids::UserId,
            project_id as database::models::ids::ProjectId
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
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let string = info.into_inner().0;

    let result =
        database::models::Project::get_from_slug_or_project_id(string, &**pool)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified project does not exist!".to_string(),
                )
            })?;

    let user_id: database::models::ids::UserId = user.id.into();
    let project_id = result.id;

    let following = sqlx::query!(
        "
        SELECT EXISTS(SELECT 1 FROM mod_follows mf WHERE mf.follower_id = $1 AND mf.mod_id = $2)
        ",
        user_id as database::models::ids::UserId,
        project_id as database::models::ids::ProjectId
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
            project_id as database::models::ids::ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mod_follows
            WHERE follower_id = $1 AND mod_id = $2
            ",
            user_id as database::models::ids::UserId,
            project_id as database::models::ids::ProjectId
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
    id: crate::models::projects::ProjectId,
    config: web::Data<SearchConfig>,
) -> Result<(), meilisearch_sdk::errors::Error> {
    let client =
        meilisearch_sdk::client::Client::new(&*config.address, &*config.key);

    let indexes: Vec<meilisearch_sdk::indexes::Index> =
        client.get_indexes().await?;
    for index in indexes {
        index.delete_document(format!("{}", id)).await?;
    }

    Ok(())
}
