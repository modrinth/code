use super::ApiError;
use crate::auth::{check_is_moderator_from_headers, get_user_from_headers};
use crate::file_hosting::FileHost;
use crate::models;
use crate::models::teams::Permissions;
use crate::{database, Pepper};
use actix_web::{delete, get, patch, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::borrow::Borrow;
use std::sync::Arc;

// TODO: this needs filtering, and a better response type
// Currently it only gives a list of ids, which have to be
// requested manually.  This route could give a list of the
// ids as well as the supported versions and loaders, or
// other info that is needed for selecting the right version.
#[get("version")]
pub async fn version_list(
    info: web::Path<(models::ids::ModId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0.into();

    let mod_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM mods WHERE id = $1)",
        id as database::models::ModId,
    )
    .fetch_one(&**pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?
    .exists;

    if mod_exists.unwrap_or(false) {
        let mod_data = database::models::Version::get_mod_versions(id, &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;

        let response = mod_data
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<models::ids::VersionId>>();

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
    req: HttpRequest,
    web::Query(ids): web::Query<VersionIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let version_ids = serde_json::from_str::<Vec<models::ids::VersionId>>(&*ids.ids)?
        .into_iter()
        .map(|x| x.into())
        .collect();
    let versions_data = database::models::Version::get_many_full(version_ids, &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    let mut versions = Vec::new();

    for version_data in versions_data {
        if let Some(version) = version_data {
            let mut authorized = version.accepted;

            if let Some(user) = &user_option {
                if !authorized {
                    if user.role.is_mod() {
                        authorized = true;
                    } else {
                        let user_id: database::models::ids::UserId = user.id.into();

                        let member_exists = sqlx::query!(
                            "SELECT EXISTS(SELECT 1 FROM team_members tm INNER JOIN mods m ON m.team_id = tm.team_id AND m.id = $1 WHERE tm.user_id = $2)",
                            version.mod_id as database::models::ModId,
                            user_id as database::models::ids::UserId,
                        )
                            .fetch_one(&**pool)
                            .await
                            .map_err(|e| ApiError::DatabaseError(e.into()))?
                            .exists;

                        authorized = member_exists.unwrap_or(false);
                    }
                }
            }

            if authorized {
                versions.push(convert_version(version));
            }
        }
    }

    Ok(HttpResponse::Ok().json(versions))
}

#[get("{version_id}")]
pub async fn version_get(
    req: HttpRequest,
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let version_data = database::models::Version::get_full(id.into(), &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;
    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(data) = version_data {
        if !data.accepted {
            if let Some(user) = user_option {
                if !user.role.is_mod() {
                    let user_id: database::models::ids::UserId = user.id.into();

                    let member_exists = sqlx::query!(
                        "SELECT EXISTS(SELECT 1 FROM team_members tm INNER JOIN mods m ON m.team_id = tm.team_id AND m.id = $1 WHERE tm.user_id = $2)",
                        data.mod_id as database::models::ModId,
                        user_id as database::models::ids::UserId,
                    )
                        .fetch_one(&**pool)
                        .await
                        .map_err(|e| ApiError::DatabaseError(e.into()))?
                        .exists;

                    if !member_exists.unwrap_or(false) {
                        return Ok(HttpResponse::NotFound().body(""));
                    }
                }
            } else {
                return Ok(HttpResponse::NotFound().body(""));
            }
        }

        Ok(HttpResponse::Ok().json(convert_version(data)))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

fn convert_version(data: database::models::version_item::QueryVersion) -> models::mods::Version {
    use models::mods::VersionType;

    models::mods::Version {
        id: data.id.into(),
        mod_id: data.mod_id.into(),
        author_id: data.author_id.into(),

        featured: data.featured,
        name: data.name,
        version_number: data.version_number,
        changelog_url: data.changelog_url,
        date_published: data.date_published,
        downloads: data.downloads as u32,
        version_type: match data.release_channel.as_str() {
            "release" => VersionType::Release,
            "beta" => VersionType::Beta,
            "alpha" => VersionType::Alpha,
            _ => VersionType::Release,
        },

        files: data
            .files
            .into_iter()
            .map(|f| {
                models::mods::VersionFile {
                    url: f.url,
                    filename: f.filename,
                    // FIXME: Hashes are currently stored as an ascii byte slice instead
                    // of as an actual byte array in the database
                    hashes: f
                        .hashes
                        .into_iter()
                        .map(|(k, v)| Some((k, String::from_utf8(v).ok()?)))
                        .collect::<Option<_>>()
                        .unwrap_or_else(Default::default),
                    primary: f.primary,
                }
            })
            .collect(),
        dependencies: Vec::new(), // TODO: dependencies
        game_versions: data
            .game_versions
            .into_iter()
            .map(models::mods::GameVersion)
            .collect(),
        loaders: data
            .loaders
            .into_iter()
            .map(models::mods::ModLoader)
            .collect(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct EditVersion {
    pub name: Option<String>,
    pub changelog: Option<String>,
    pub version_type: Option<models::mods::VersionType>,
    pub dependencies: Option<Vec<models::ids::VersionId>>,
    pub game_versions: Option<Vec<models::mods::GameVersion>>,
    pub loaders: Option<Vec<models::mods::ModLoader>>,
    pub accepted: Option<bool>,
    pub featured: Option<bool>,
    pub primary_file: Option<(String, String)>,
}

#[patch("{id}")]
pub async fn version_edit(
    req: HttpRequest,
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    new_version: web::Json<EditVersion>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let version_id = info.into_inner().0;
    let id = version_id.into();

    let result = database::models::Version::get_full(id, &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if let Some(version_item) = result {
        let mod_item = database::models::Mod::get(version_item.mod_id, &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?
            .ok_or_else(|| {
                ApiError::InvalidInputError(
                    "Attempted to edit version not attached to mod. How did this happen?"
                        .to_string(),
                )
            })?;

        let team_member = database::models::TeamMember::get_from_user_id(
            mod_item.team_id,
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
                    "You do not have the permissions to edit this version!".to_string(),
                ));
            }

            let mut transaction = pool
                .begin()
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

            if let Some(accepted) = &new_version.accepted {
                if !user.role.is_mod() {
                    return Err(ApiError::CustomAuthenticationError(
                        "You do not have the permissions to edit the approval of this version!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE versions
                    SET accepted = $1
                    WHERE (id = $2)
                    ",
                    accepted,
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

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
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

            if let Some(version_type) = &new_version.version_type {
                let channel = database::models::ids::ChannelId::get_id(
                    version_type.as_str(),
                    &mut *transaction,
                )
                .await?
                .ok_or_else(|| {
                    ApiError::InvalidInputError(
                        "No database entry for version type provided.".to_string(),
                    )
                })?;

                sqlx::query!(
                    "
                    UPDATE versions
                    SET release_channel = $1
                    WHERE (id = $2)
                    ",
                    channel as database::models::ids::ChannelId,
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

            if let Some(dependencies) = &new_version.dependencies {
                sqlx::query!(
                    "
                    DELETE FROM dependencies WHERE dependent_id = $1
                    ",
                    id as database::models::ids::VersionId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

                for dependency in dependencies {
                    let dependency_id: database::models::ids::VersionId = dependency.clone().into();

                    sqlx::query!(
                        "
                        INSERT INTO dependencies (dependent_id, dependency_id)
                        VALUES ($1, $2)
                        ",
                        id as database::models::ids::VersionId,
                        dependency_id as database::models::ids::VersionId,
                    )
                    .execute(&mut *transaction)
                    .await
                    .map_err(|e| ApiError::DatabaseError(e.into()))?;
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
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

                for game_version in game_versions {
                    let game_version_id = database::models::categories::GameVersion::get_id(
                        &game_version.0,
                        &mut *transaction,
                    )
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInputError(
                            "No database entry for game version provided.".to_string(),
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
                    .await
                    .map_err(|e| ApiError::DatabaseError(e.into()))?;
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
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

                for loader in loaders {
                    let loader_id =
                        database::models::categories::Loader::get_id(&loader.0, &mut *transaction)
                            .await?
                            .ok_or_else(|| {
                                ApiError::InvalidInputError(
                                    "No database entry for loader provided.".to_string(),
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
                    .await
                    .map_err(|e| ApiError::DatabaseError(e.into()))?;
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
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

            if let Some(primary_file) = &new_version.primary_file {
                let result = sqlx::query!(
                    "
                    SELECT id FROM files
                    INNER JOIN hashes ON hash = $1 AND algorithm = $2
                    ",
                    primary_file.1.as_bytes(),
                    primary_file.0
                )
                .fetch_optional(&**pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?
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
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

                sqlx::query!(
                    "
                    UPDATE files
                    SET is_primary = TRUE
                    WHERE (id = $1)
                    ",
                    result.id,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

            if let Some(body) = &new_version.changelog {
                let mod_id: models::mods::ModId = version_item.mod_id.into();
                let body_path = format!(
                    "data/{}/versions/{}/changelog.md",
                    mod_id, version_item.version_number
                );

                file_host.delete_file_version("", &*body_path).await?;

                file_host
                    .upload_file("text/plain", &body_path, body.clone().into_bytes())
                    .await?;
            }

            transaction
                .commit()
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            Ok(HttpResponse::Ok().body(""))
        } else {
            Err(ApiError::CustomAuthenticationError(
                "You do not have permission to edit this version!".to_string(),
            ))
        }
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

    if !user.role.is_mod() {
        let version = database::models::Version::get(id.into(), &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?
            .ok_or_else(|| {
                ApiError::InvalidInputError("An invalid version ID was specified".to_string())
            })?;
        let mod_item = database::models::Mod::get(version.mod_id, &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?
            .ok_or_else(|| {
                ApiError::InvalidInputError("The version is not attached to a mod".to_string())
            })?;
        let team_member = database::models::TeamMember::get_from_user_id(
            mod_item.team_id,
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
                "You do not have permission to delete versions in this team".to_string(),
            ));
        }
    }

    let result = database::models::Version::remove_full(id.into(), &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Deserialize)]
pub struct Algorithm {
    #[serde(default = "default_algorithm")]
    algorithm: String,
}

fn default_algorithm() -> String {
    "sha1".into()
}

// under /api/v1/version_file/{hash}
#[get("{version_id}")]
pub async fn get_version_from_hash(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    algorithm: web::Query<Algorithm>,
) -> Result<HttpResponse, ApiError> {
    let hash = info.into_inner().0;

    let result = sqlx::query!(
        "
        SELECT f.version_id version_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        WHERE h.algorithm = $2 AND h.hash = $1
        ",
        hash.as_bytes(),
        algorithm.algorithm
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if let Some(id) = result {
        let version_data = database::models::Version::get_full(
            database::models::VersionId(id.version_id),
            &**pool,
        )
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

        if let Some(data) = version_data {
            Ok(HttpResponse::Ok().json(convert_version(data)))
        } else {
            Ok(HttpResponse::NotFound().body(""))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Serialize, Deserialize)]
pub struct DownloadRedirect {
    pub url: String,
}

// under /api/v1/version_file/{hash}/download
#[allow(clippy::await_holding_refcell_ref)]
#[get("{version_id}/download")]
pub async fn download_version(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    algorithm: web::Query<Algorithm>,
    pepper: web::Data<Pepper>,
) -> Result<HttpResponse, ApiError> {
    let hash = info.into_inner().0;

    let result = sqlx::query!(
        "
        SELECT f.url url, f.id id, f.version_id version_id, v.mod_id mod_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = $1
        ",
        hash.as_bytes(),
        algorithm.algorithm
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if let Some(id) = result {
        let real_ip = req.connection_info();
        let ip_option = real_ip.borrow().remote_addr();

        if let Some(ip) = ip_option {
            let hash = sha1::Sha1::from(format!("{}{}", ip, pepper.pepper)).hexdigest();

            let download_exists = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM downloads WHERE version_id = $1 AND date > (CURRENT_DATE - INTERVAL '30 minutes ago') AND identifier = $2)",
                id.version_id,
                hash,
            )
                .fetch_one(&**pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?
                .exists.unwrap_or(false);

            if !download_exists {
                sqlx::query!(
                    "
                INSERT INTO downloads (
                    version_id, identifier
                )
                VALUES (
                    $1, $2
                )
                ",
                    id.version_id,
                    hash
                )
                .execute(&**pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

                sqlx::query!(
                    "
                UPDATE versions
                SET downloads = downloads + 1
                WHERE id = $1
                ",
                    id.version_id,
                )
                .execute(&**pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

                sqlx::query!(
                    "
                UPDATE mods
                SET downloads = downloads + 1
                WHERE id = $1
                ",
                    id.mod_id,
                )
                .execute(&**pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }
        }
        Ok(HttpResponse::TemporaryRedirect()
            .header("Location", &*id.url)
            .json(DownloadRedirect { url: id.url }))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

// under /api/v1/version_file/{hash}
#[delete("{version_id}")]
pub async fn delete_file(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    algorithm: web::Query<Algorithm>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(req.headers(), &**pool).await?;

    let hash = info.into_inner().0;

    let result = sqlx::query!(
        "
        SELECT f.id id, f.version_id version_id, f.filename filename, v.version_number version_number, v.mod_id mod_id FROM hashes h
        INNER JOIN files f ON h.file_id = f.id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = $1
        ",
        hash.as_bytes(),
        algorithm.algorithm
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if let Some(row) = result {
        let mut transaction = pool
            .begin()
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;

        sqlx::query!(
            "
                DELETE FROM hashes
                WHERE hash = $1 AND algorithm = $2
                ",
            hash.as_bytes(),
            algorithm.algorithm
        )
        .execute(&mut *transaction)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

        sqlx::query!(
            "
                DELETE FROM files
                WHERE files.id = $1
                ",
            row.id,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

        let mod_id: models::mods::ModId = database::models::ids::ModId(row.mod_id).into();
        file_host
            .delete_file_version(
                "",
                &format!(
                    "data/{}/versions/{}/{}",
                    mod_id, row.version_number, row.filename
                ),
            )
            .await?;

        transaction
            .commit()
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;

        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
