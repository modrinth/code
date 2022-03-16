use super::ApiError;
use crate::database;
use crate::database::models as db_models;
use crate::models;
use crate::models::projects::{Dependency, Version};
use crate::models::teams::Permissions;
use crate::util::auth::{get_user_from_headers, is_authorized};
use crate::util::guards::admin_key_guard;
use crate::util::validate::validation_errors_to_string;
use actix_web::{delete, get, patch, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

#[derive(Serialize, Deserialize, Clone)]
pub struct VersionListFilters {
    pub game_versions: Option<String>,
    pub loaders: Option<String>,
    pub featured: Option<bool>,
}

#[get("version")]
pub async fn version_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    web::Query(filters): web::Query<VersionListFilters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let result =
        database::models::Project::get_full_from_slug_or_project_id(&string, &**pool)
            .await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(project) = result {
        if !is_authorized(&project, &user_option, &pool).await? {
            return Ok(HttpResponse::NotFound().body(""));
        }

        let id = project.inner.id;

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
            &**pool,
        )
        .await?;

        let mut versions =
            database::models::Version::get_many_full(version_ids, &**pool)
                .await?;

        let mut response = versions
            .iter()
            .cloned()
            .filter(|version| {
                filters
                    .featured
                    .map(|featured| featured == version.featured)
                    .unwrap_or(true)
            })
            .map(Version::from)
            .collect::<Vec<_>>();

        versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));

        // Attempt to populate versions with "auto featured" versions
        if response.is_empty()
            && !versions.is_empty()
            && filters.featured.unwrap_or(false)
        {
            let (loaders, game_versions) = futures::join!(
                database::models::categories::Loader::list(&**pool),
                database::models::categories::GameVersion::list_filter(
                    None,
                    Some(true),
                    &**pool
                )
            );

            let (loaders, game_versions) = (loaders?, game_versions?);

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
                    .map(|version| {
                        response.push(Version::from(version.clone()))
                    })
                    .unwrap_or(());
            });

            if response.is_empty() {
                versions
                    .into_iter()
                    .for_each(|version| response.push(Version::from(version)));
            }
        }

        response.sort_by(|a, b| b.date_published.cmp(&a.date_published));
        response.dedup_by(|a, b| a.id == b.id);

        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Serialize, Deserialize)]
pub struct VersionIds {
    pub ids: String,
}

#[get("versions")]
pub async fn versions_get(
    web::Query(ids): web::Query<VersionIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let version_ids =
        serde_json::from_str::<Vec<models::ids::VersionId>>(&*ids.ids)?
            .into_iter()
            .map(|x| x.into())
            .collect();
    let versions_data =
        database::models::Version::get_many_full(version_ids, &**pool).await?;

    let versions = versions_data
        .into_iter()
        .map(Version::from)
        .collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(versions))
}

#[get("{version_id}")]
pub async fn version_get(
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let version_data =
        database::models::Version::get_full(id.into(), &**pool).await?;

    if let Some(data) = version_data {
        Ok(HttpResponse::Ok().json(models::projects::Version::from(data)))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct EditVersion {
    #[validate(length(min = 3, max = 256))]
    pub name: Option<String>,
    #[validate(
        length(min = 1, max = 64),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    pub version_number: Option<String>,
    #[validate(length(max = 65536))]
    pub changelog: Option<String>,
    pub version_type: Option<models::projects::VersionType>,
    #[validate(
        length(min = 0, max = 256),
        custom(function = "crate::util::validate::validate_deps")
    )]
    pub dependencies: Option<Vec<Dependency>>,
    pub game_versions: Option<Vec<models::projects::GameVersion>>,
    pub loaders: Option<Vec<models::projects::Loader>>,
    pub featured: Option<bool>,
    pub primary_file: Option<(String, String)>,
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
        ApiError::ValidationError(validation_errors_to_string(err, None))
    })?;

    let version_id = info.into_inner().0;
    let id = version_id.into();

    let result = database::models::Version::get_full(id, &**pool).await?;

    if let Some(version_item) = result {
        let team_member =
            database::models::TeamMember::get_from_user_id_version(
                version_item.id,
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
            if !perms.contains(Permissions::UPLOAD_VERSION) {
                return Err(ApiError::CustomAuthenticationError(
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
                        dependency_type: x.dependency_type.to_string(),
                    })
                    .collect::<Vec<database::models::version_item::DependencyBuilder>>();

                for dependency in builders {
                    dependency
                        .insert(version_item.id, &mut transaction)
                        .await?;
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
                            ApiError::InvalidInputError(
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
                            ApiError::InvalidInputError(
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
                    ApiError::InvalidInputError(format!(
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

            transaction.commit().await?;
            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthenticationError(
                "You do not have permission to edit this version!".to_string(),
            ))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

// This is an internal route, cannot be used without key
#[patch(
    "{project_id}/{version_name}/_count-download",
    guard = "admin_key_guard"
)]
pub async fn version_count_patch(
    info: web::Path<(models::ids::ProjectId, String)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let (project, version_name) = info.into_inner();
    let project = db_models::ids::ProjectId::from(project);

    let version = sqlx::query!(
        "SELECT id FROM versions
         WHERE (version_number = $1 AND mod_id = $2)",
        version_name,
        project as db_models::ids::ProjectId
    )
    .fetch_optional(pool.as_ref())
    .await?;
    let version = match version {
        Some(version) => db_models::ids::VersionId(version.id),
        _ => {
            return Ok(HttpResponse::NotFound().body("Could not find version!"))
        }
    };

    futures::future::try_join(
        sqlx::query!(
            "UPDATE versions
             SET downloads = downloads + 1
             WHERE (id = $1)",
            version as db_models::ids::VersionId
        )
        .execute(pool.as_ref()),
        sqlx::query!(
            "UPDATE mods
             SET downloads = downloads + 1
             WHERE (id = $1)",
            project as db_models::ids::ProjectId
        )
        .execute(pool.as_ref()),
    )
    .await
    .map_err(ApiError::SqlxDatabaseError)?;

    Ok(HttpResponse::Ok().body(""))
}

#[delete("{version_id}")]
pub async fn version_delete(
    req: HttpRequest,
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let id = info.into_inner().0;

    if !user.role.is_mod() {
        let team_member = database::models::TeamMember::get_from_user_id_version(
            id.into(),
            user.id.into(),
            &**pool,
        )
        .await
        .map_err(ApiError::DatabaseError)?
        .ok_or_else(|| {
            ApiError::InvalidInputError(
                "You do not have permission to delete versions in this team".to_string(),
            )
        })?;

        if !team_member
            .permissions
            .contains(Permissions::DELETE_VERSION)
        {
            return Err(ApiError::CustomAuthenticationError(
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
