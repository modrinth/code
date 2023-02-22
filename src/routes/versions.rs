use super::ApiError;
use crate::database;
use crate::models;
use crate::models::projects::{
    Dependency, FileType, VersionStatus, VersionType,
};
use crate::models::teams::Permissions;
use crate::util::auth::{
    filter_authorized_versions, get_user_from_headers, is_authorized,
    is_authorized_version,
};
use crate::util::validate::validation_errors_to_string;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

#[derive(Serialize, Deserialize, Clone)]
pub struct VersionListFilters {
    pub game_versions: Option<String>,
    pub loaders: Option<String>,
    pub featured: Option<bool>,
    pub version_type: Option<VersionType>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[get("version")]
pub async fn version_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    web::Query(filters): web::Query<VersionListFilters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let result = database::models::Project::get_from_slug_or_project_id(
        &string, &**pool,
    )
    .await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(project) = result {
        if !is_authorized(&project, &user_option, &pool).await? {
            return Ok(HttpResponse::NotFound().body(""));
        }

        let id = project.id;

        let version_ids = database::models::Version::get_project_versions(
            id,
            filters
                .game_versions
                .as_ref()
                .map(|x| serde_json::from_str(x).unwrap_or_default()),
            filters
                .loaders
                .as_ref()
                .map(|x| serde_json::from_str(x).unwrap_or_default()),
            filters.version_type,
            filters.limit,
            filters.offset,
            &**pool,
        )
        .await?;

        let mut versions =
            database::models::Version::get_many_full(version_ids, &**pool)
                .await?;

        let mut response = versions
            .iter()
            .filter(|version| {
                filters
                    .featured
                    .map(|featured| featured == version.inner.featured)
                    .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();

        versions.sort_by(|a, b| {
            b.inner.date_published.cmp(&a.inner.date_published)
        });

        // Attempt to populate versions with "auto featured" versions
        if response.is_empty()
            && !versions.is_empty()
            && filters.featured.unwrap_or(false)
        {
            let (loaders, game_versions) = futures::future::try_join(
                database::models::categories::Loader::list(&**pool),
                database::models::categories::GameVersion::list_filter(
                    None,
                    Some(true),
                    &**pool,
                ),
            )
            .await?;

            let mut joined_filters = Vec::new();
            for game_version in &game_versions {
                for loader in &loaders {
                    joined_filters.push((game_version, loader))
                }
            }

            joined_filters.into_iter().for_each(|filter| {
                versions
                    .iter()
                    .find(|version| {
                        version.game_versions.contains(&filter.0.version)
                            && version.loaders.contains(&filter.1.loader)
                    })
                    .map(|version| response.push(version.clone()))
                    .unwrap_or(());
            });

            if response.is_empty() {
                versions
                    .into_iter()
                    .for_each(|version| response.push(version));
            }
        }

        response.sort_by(|a, b| {
            b.inner.date_published.cmp(&a.inner.date_published)
        });
        response.dedup_by(|a, b| a.inner.id == b.inner.id);

        let response =
            filter_authorized_versions(response, &user_option, &pool).await?;

        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

// Given a project ID/slug and a version slug
#[get("version/{slug}")]
pub async fn version_project_get(
    req: HttpRequest,
    info: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner();
    let version_data =
        database::models::Version::get_full_from_id_slug(&id.0, &id.1, &**pool)
            .await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(data) = version_data {
        if is_authorized_version(&data.inner, &user_option, &pool).await? {
            return Ok(
                HttpResponse::Ok().json(models::projects::Version::from(data))
            );
        }
    }

    Ok(HttpResponse::NotFound().body(""))
}

#[derive(Serialize, Deserialize)]
pub struct VersionIds {
    pub ids: String,
}

#[get("versions")]
pub async fn versions_get(
    req: HttpRequest,
    web::Query(ids): web::Query<VersionIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let version_ids =
        serde_json::from_str::<Vec<models::ids::VersionId>>(&ids.ids)?
            .into_iter()
            .map(|x| x.into())
            .collect();
    let versions_data =
        database::models::Version::get_many_full(version_ids, &**pool).await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    let versions =
        filter_authorized_versions(versions_data, &user_option, &pool).await?;

    Ok(HttpResponse::Ok().json(versions))
}

#[get("{version_id}")]
pub async fn version_get(
    req: HttpRequest,
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let version_data =
        database::models::Version::get_full(id.into(), &**pool).await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(data) = version_data {
        if is_authorized_version(&data.inner, &user_option, &pool).await? {
            return Ok(
                HttpResponse::Ok().json(models::projects::Version::from(data))
            );
        }
    }

    Ok(HttpResponse::NotFound().body(""))
}

#[derive(Serialize, Deserialize, Validate)]
pub struct EditVersion {
    #[validate(length(min = 1, max = 64))]
    pub name: Option<String>,
    #[validate(
        length(min = 1, max = 32),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    pub version_number: Option<String>,
    #[validate(length(max = 65536))]
    pub changelog: Option<String>,
    pub version_type: Option<models::projects::VersionType>,
    #[validate(
        length(min = 0, max = 4096),
        custom(function = "crate::util::validate::validate_deps")
    )]
    pub dependencies: Option<Vec<Dependency>>,
    pub game_versions: Option<Vec<models::projects::GameVersion>>,
    pub loaders: Option<Vec<models::projects::Loader>>,
    pub featured: Option<bool>,
    pub primary_file: Option<(String, String)>,
    pub downloads: Option<u32>,
    pub status: Option<VersionStatus>,
    pub file_types: Option<Vec<EditVersionFileType>>,
}

#[derive(Serialize, Deserialize)]
pub struct EditVersionFileType {
    pub algorithm: String,
    pub hash: String,
    pub file_type: Option<FileType>,
}

#[patch("{id}")]
pub async fn version_edit(
    req: HttpRequest,
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
    new_version: web::Json<EditVersion>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    new_version.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let version_id = info.into_inner().0;
    let id = version_id.into();

    let result = database::models::Version::get_full(id, &**pool).await?;

    if let Some(version_item) = result {
        let project_item = database::models::Project::get_full(
            version_item.inner.project_id,
            &**pool,
        )
        .await?;

        let team_member =
            database::models::TeamMember::get_from_user_id_version(
                version_item.inner.id,
                user.id.into(),
                &**pool,
            )
            .await?;
        let permissions;

        if user.role.is_admin() {
            permissions = Some(Permissions::ALL)
        } else if let Some(member) = team_member {
            permissions = Some(member.permissions)
        } else if user.role.is_mod() {
            permissions =
                Some(Permissions::EDIT_DETAILS | Permissions::EDIT_BODY)
        } else {
            permissions = None
        }

        if let Some(perms) = permissions {
            if !perms.contains(Permissions::UPLOAD_VERSION) {
                return Err(ApiError::CustomAuthentication(
                    "You do not have the permissions to edit this version!"
                        .to_string(),
                ));
            }

            let mut transaction = pool.begin().await?;

            if let Some(name) = &new_version.name {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET name = $1
                    WHERE (id = $2)
                    ",
                    name,
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(number) = &new_version.version_number {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET version_number = $1
                    WHERE (id = $2)
                    ",
                    number,
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(version_type) = &new_version.version_type {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET version_type = $1
                    WHERE (id = $2)
                    ",
                    version_type.as_str(),
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(dependencies) = &new_version.dependencies {
                if let Some(project) = project_item {
                    if project.project_type != "modpack" {
                        sqlx::query!(
                            "
                            DELETE FROM dependencies WHERE dependent_id = $1
                            ",
                            id as database::models::ids::VersionId,
                        )
                        .execute(&mut *transaction)
                        .await?;

                        let builders = dependencies
                            .iter()
                            .map(|x| database::models::version_item::DependencyBuilder {
                                project_id: x.project_id.map(|x| x.into()),
                                version_id: x.version_id.map(|x| x.into()),
                                file_name: x.file_name.clone(),
                                dependency_type: x.dependency_type.to_string(),
                            })
                            .collect::<Vec<database::models::version_item::DependencyBuilder>>();

                        for dependency in builders {
                            dependency
                                .insert(version_item.inner.id, &mut transaction)
                                .await?;
                        }
                    }
                }
            }

            if let Some(game_versions) = &new_version.game_versions {
                sqlx::query!(
                    "
                    DELETE FROM game_versions_versions WHERE joining_version_id = $1
                    ",
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;

                for game_version in game_versions {
                    let game_version_id =
                        database::models::categories::GameVersion::get_id(
                            &game_version.0,
                            &mut *transaction,
                        )
                        .await?
                        .ok_or_else(|| {
                            ApiError::InvalidInput(
                                "No database entry for game version provided."
                                    .to_string(),
                            )
                        })?;

                    sqlx::query!(
                        "
                        INSERT INTO game_versions_versions (game_version_id, joining_version_id)
                        VALUES ($1, $2)
                        ",
                        game_version_id as database::models::ids::GameVersionId,
                        id as database::models::ids::VersionId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }

                database::models::Project::update_game_versions(
                    version_item.inner.project_id,
                    &mut transaction,
                )
                .await?;
            }

            if let Some(loaders) = &new_version.loaders {
                sqlx::query!(
                    "
                    DELETE FROM loaders_versions WHERE version_id = $1
                    ",
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;

                for loader in loaders {
                    let loader_id =
                        database::models::categories::Loader::get_id(
                            &loader.0,
                            &mut *transaction,
                        )
                        .await?
                        .ok_or_else(|| {
                            ApiError::InvalidInput(
                                "No database entry for loader provided."
                                    .to_string(),
                            )
                        })?;

                    sqlx::query!(
                        "
                        INSERT INTO loaders_versions (loader_id, version_id)
                        VALUES ($1, $2)
                        ",
                        loader_id as database::models::ids::LoaderId,
                        id as database::models::ids::VersionId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }

                database::models::Project::update_loaders(
                    version_item.inner.project_id,
                    &mut transaction,
                )
                .await?;
            }

            if let Some(featured) = &new_version.featured {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET featured = $1
                    WHERE (id = $2)
                    ",
                    featured,
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(primary_file) = &new_version.primary_file {
                let result = sqlx::query!(
                    "
                    SELECT f.id id FROM hashes h
                    INNER JOIN files f ON h.file_id = f.id
                    WHERE h.algorithm = $2 AND h.hash = $1
                    ",
                    primary_file.1.as_bytes(),
                    primary_file.0
                )
                .fetch_optional(&**pool)
                .await?
                .ok_or_else(|| {
                    ApiError::InvalidInput(format!(
                        "Specified file with hash {} does not exist.",
                        primary_file.1.clone()
                    ))
                })?;

                sqlx::query!(
                    "
                    UPDATE files
                    SET is_primary = FALSE
                    WHERE (version_id = $1)
                    ",
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;

                sqlx::query!(
                    "
                    UPDATE files
                    SET is_primary = TRUE
                    WHERE (id = $1)
                    ",
                    result.id,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(body) = &new_version.changelog {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET changelog = $1
                    WHERE (id = $2)
                    ",
                    body,
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(downloads) = &new_version.downloads {
                if !user.role.is_mod() {
                    return Err(ApiError::CustomAuthentication(
                        "You don't have permission to set the downloads of this mod"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE versions
                    SET downloads = $1
                    WHERE (id = $2)
                    ",
                    *downloads as i32,
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;

                let diff = *downloads - (version_item.inner.downloads as u32);

                sqlx::query!(
                    "
                    UPDATE mods
                    SET downloads = downloads + $1
                    WHERE (id = $2)
                    ",
                    diff as i32,
                    version_item.inner.project_id
                        as database::models::ids::ProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(status) = &new_version.status {
                if !status.can_be_requested() {
                    return Err(ApiError::InvalidInput(
                        "The requested status cannot be set!".to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE versions
                    SET status = $1
                    WHERE (id = $2)
                    ",
                    status.as_str(),
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(file_types) = &new_version.file_types {
                for file_type in file_types {
                    let result = sqlx::query!(
                        "
                        SELECT f.id id FROM hashes h
                        INNER JOIN files f ON h.file_id = f.id
                        WHERE h.algorithm = $2 AND h.hash = $1
                        ",
                        file_type.hash.as_bytes(),
                        file_type.algorithm
                    )
                    .fetch_optional(&**pool)
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInput(format!(
                            "Specified file with hash {} does not exist.",
                            file_type.algorithm.clone()
                        ))
                    })?;

                    sqlx::query!(
                        "
                        UPDATE files
                        SET file_type = $2
                        WHERE (id = $1)
                        ",
                        result.id,
                        file_type.file_type.as_ref().map(|x| x.as_str()),
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }

            transaction.commit().await?;
            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthentication(
                "You do not have permission to edit this version!".to_string(),
            ))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Deserialize)]
pub struct SchedulingData {
    pub time: DateTime<Utc>,
    pub requested_status: VersionStatus,
}

#[post("{id}/schedule")]
pub async fn version_schedule(
    req: HttpRequest,
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
    scheduling_data: web::Json<SchedulingData>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    if scheduling_data.time < Utc::now() {
        return Err(ApiError::InvalidInput(
            "You cannot schedule a version to be released in the past!"
                .to_string(),
        ));
    }

    if !scheduling_data.requested_status.can_be_requested() {
        return Err(ApiError::InvalidInput(
            "Specified requested status cannot be requested!".to_string(),
        ));
    }

    let string = info.into_inner().0;
    let result =
        database::models::Version::get_full(string.into(), &**pool).await?;

    if let Some(version_item) = result {
        let team_member =
            database::models::TeamMember::get_from_user_id_version(
                version_item.inner.id,
                user.id.into(),
                &**pool,
            )
            .await?;

        if user.role.is_mod()
            || team_member
                .map(|x| x.permissions.contains(Permissions::EDIT_DETAILS))
                .unwrap_or(false)
        {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to edit this version's scheduling data!".to_string(),
            ));
        }

        sqlx::query!(
            "
            UPDATE versions
            SET status = $1, date_published = $2
            WHERE (id = $3)
            ",
            VersionStatus::Scheduled.as_str(),
            scheduling_data.time,
            version_item.inner.id as database::models::ids::VersionId,
        )
        .execute(&**pool)
        .await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[delete("{version_id}")]
pub async fn version_delete(
    req: HttpRequest,
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let id = info.into_inner().0;

    if !user.role.is_admin() {
        let team_member = database::models::TeamMember::get_from_user_id_version(
            id.into(),
            user.id.into(),
            &**pool,
        )
        .await
        .map_err(ApiError::Database)?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "You do not have permission to delete versions in this team".to_string(),
            )
        })?;

        if !team_member
            .permissions
            .contains(Permissions::DELETE_VERSION)
        {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to delete versions in this team"
                    .to_string(),
            ));
        }
    }

    let mut transaction = pool.begin().await?;

    let result =
        database::models::Version::remove_full(id.into(), &mut transaction)
            .await?;

    transaction.commit().await?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
