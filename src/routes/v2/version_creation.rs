use super::project_creation::{CreateError, UploadedFile};
use crate::auth::get_user_from_headers;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::version_item::{
    DependencyBuilder, VersionBuilder, VersionFileBuilder,
};
use crate::database::models::{self, image_item, Organization};
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::images::{Image, ImageContext, ImageId};
use crate::models::notifications::NotificationBody;
use crate::models::pack::PackFileHash;
use crate::models::pats::Scopes;
use crate::models::projects::{
    Dependency, DependencyType, FileType, GameVersion, Loader, ProjectId, Version, VersionFile,
    VersionId, VersionStatus, VersionType,
};
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::util::routes::read_from_field;
use crate::util::validate::validation_errors_to_string;
use crate::validate::{validate_file, ValidationResult};
use actix_multipart::{Field, Multipart};
use actix_web::web::Data;
use actix_web::{post, web, HttpRequest, HttpResponse};
use chrono::Utc;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use validator::Validate;

fn default_requested_status() -> VersionStatus {
    VersionStatus::Listed
}

#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct InitialVersionData {
    #[serde(alias = "mod_id")]
    pub project_id: Option<ProjectId>,
    #[validate(length(min = 1, max = 256))]
    pub file_parts: Vec<String>,
    #[validate(
        length(min = 1, max = 32),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    pub version_number: String,
    #[validate(
        length(min = 1, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    #[serde(alias = "name")]
    pub version_title: String,
    #[validate(length(max = 65536))]
    #[serde(alias = "changelog")]
    pub version_body: Option<String>,
    #[validate(
        length(min = 0, max = 4096),
        custom(function = "crate::util::validate::validate_deps")
    )]
    pub dependencies: Vec<Dependency>,
    #[validate(length(min = 1))]
    pub game_versions: Vec<GameVersion>,
    #[serde(alias = "version_type")]
    pub release_channel: VersionType,
    #[validate(length(min = 1))]
    pub loaders: Vec<Loader>,
    pub featured: bool,
    pub primary_file: Option<String>,
    #[serde(default = "default_requested_status")]
    pub status: VersionStatus,
    #[serde(default = "HashMap::new")]
    pub file_types: HashMap<String, Option<FileType>>,
    // Associations to uploaded images in changelog
    #[validate(length(max = 10))]
    #[serde(default)]
    pub uploaded_images: Vec<ImageId>,
}

#[derive(Serialize, Deserialize, Clone)]
struct InitialFileData {
    #[serde(default = "HashMap::new")]
    pub file_types: HashMap<String, Option<FileType>>,
}

// under `/api/v1/version`
#[post("version")]
pub async fn version_create(
    req: HttpRequest,
    mut payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let result = version_create_inner(
        req,
        &mut payload,
        &mut transaction,
        &redis,
        &***file_host,
        &mut uploaded_files,
        &client,
        &session_queue,
    )
    .await;

    if result.is_err() {
        let undo_result =
            super::project_creation::undo_uploads(&***file_host, &uploaded_files).await;
        let rollback_result = transaction.rollback().await;

        undo_result?;
        if let Err(e) = rollback_result {
            return Err(e.into());
        }
    } else {
        transaction.commit().await?;
    }

    result
}

#[allow(clippy::too_many_arguments)]
async fn version_create_inner(
    req: HttpRequest,
    payload: &mut Multipart,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    redis: &RedisPool,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    pool: &PgPool,
    session_queue: &AuthQueue,
) -> Result<HttpResponse, CreateError> {
    let cdn_url = dotenvy::var("CDN_URL")?;

    let mut initial_version_data = None;
    let mut version_builder = None;

    let all_game_versions =
        models::categories::GameVersion::list(&mut **transaction, redis).await?;
    let all_loaders = models::categories::Loader::list(&mut **transaction, redis).await?;

    let user = get_user_from_headers(
        &req,
        pool,
        redis,
        session_queue,
        Some(&[Scopes::VERSION_CREATE]),
    )
    .await?
    .1;

    let mut error = None;
    while let Some(item) = payload.next().await {
        let mut field: Field = item?;

        if error.is_some() {
            continue;
        }

        let result = async {
            let content_disposition = field.content_disposition().clone();
            let name = content_disposition.get_name().ok_or_else(|| {
                CreateError::MissingValueError("Missing content name".to_string())
            })?;

            if name == "data" {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk?);
                }

                let version_create_data: InitialVersionData = serde_json::from_slice(&data)?;
                initial_version_data = Some(version_create_data);
                let version_create_data = initial_version_data.as_ref().unwrap();
                if version_create_data.project_id.is_none() {
                    return Err(CreateError::MissingValueError(
                        "Missing project id".to_string(),
                    ));
                }

                version_create_data.validate().map_err(|err| {
                    CreateError::ValidationError(validation_errors_to_string(err, None))
                })?;

                if !version_create_data.status.can_be_requested() {
                    return Err(CreateError::InvalidInput(
                        "Status specified cannot be requested".to_string(),
                    ));
                }

                let project_id: models::ProjectId = version_create_data.project_id.unwrap().into();

                // Ensure that the project this version is being added to exists
                let results = sqlx::query!(
                    "SELECT EXISTS(SELECT 1 FROM mods WHERE id=$1)",
                    project_id as models::ProjectId
                )
                .fetch_one(&mut **transaction)
                .await?;

                if !results.exists.unwrap_or(false) {
                    return Err(CreateError::InvalidInput(
                        "An invalid project id was supplied".to_string(),
                    ));
                }

                // Check that the user creating this version is a team member
                // of the project the version is being added to.
                let team_member = models::TeamMember::get_from_user_id_project(
                    project_id,
                    user.id.into(),
                    &mut **transaction,
                )
                .await?;

                // Get organization attached, if exists, and the member project permissions
                let organization = models::Organization::get_associated_organization_project_id(
                    project_id,
                    &mut **transaction,
                )
                .await?;

                let organization_team_member = if let Some(organization) = &organization {
                    models::TeamMember::get_from_user_id(
                        organization.team_id,
                        user.id.into(),
                        &mut **transaction,
                    )
                    .await?
                } else {
                    None
                };

                let permissions = ProjectPermissions::get_permissions_by_role(
                    &user.role,
                    &team_member,
                    &organization_team_member,
                )
                .unwrap_or_default();

                if !permissions.contains(ProjectPermissions::UPLOAD_VERSION) {
                    return Err(CreateError::CustomAuthenticationError(
                        "You don't have permission to upload this version!".to_string(),
                    ));
                }

                let version_id: VersionId = models::generate_version_id(transaction).await?.into();

                let project_type = sqlx::query!(
                    "
                SELECT name FROM project_types pt
                INNER JOIN mods ON mods.project_type = pt.id
                WHERE mods.id = $1
                ",
                    project_id as models::ProjectId,
                )
                .fetch_one(&mut **transaction)
                .await?
                .name;

                let game_versions = version_create_data
                    .game_versions
                    .iter()
                    .map(|x| {
                        all_game_versions
                            .iter()
                            .find(|y| y.version == x.0)
                            .ok_or_else(|| CreateError::InvalidGameVersion(x.0.clone()))
                            .map(|y| y.id)
                    })
                    .collect::<Result<Vec<models::GameVersionId>, CreateError>>()?;

                let loaders = version_create_data
                    .loaders
                    .iter()
                    .map(|x| {
                        all_loaders
                            .iter()
                            .find(|y| {
                                y.loader == x.0 && y.supported_project_types.contains(&project_type)
                            })
                            .ok_or_else(|| CreateError::InvalidLoader(x.0.clone()))
                            .map(|y| y.id)
                    })
                    .collect::<Result<Vec<models::LoaderId>, CreateError>>()?;

                let dependencies = version_create_data
                    .dependencies
                    .iter()
                    .map(|d| models::version_item::DependencyBuilder {
                        version_id: d.version_id.map(|x| x.into()),
                        project_id: d.project_id.map(|x| x.into()),
                        dependency_type: d.dependency_type.to_string(),
                        file_name: None,
                    })
                    .collect::<Vec<_>>();

                version_builder = Some(VersionBuilder {
                    version_id: version_id.into(),
                    project_id,
                    author_id: user.id.into(),
                    name: version_create_data.version_title.clone(),
                    version_number: version_create_data.version_number.clone(),
                    changelog: version_create_data.version_body.clone().unwrap_or_default(),
                    files: Vec::new(),
                    dependencies,
                    game_versions,
                    loaders,
                    version_type: version_create_data.release_channel.to_string(),
                    featured: version_create_data.featured,
                    status: version_create_data.status,
                    requested_status: None,
                });

                return Ok(());
            }

            let version = version_builder.as_mut().ok_or_else(|| {
                CreateError::InvalidInput(String::from("`data` field must come before file fields"))
            })?;

            let project_type = sqlx::query!(
                "
            SELECT name FROM project_types pt
            INNER JOIN mods ON mods.project_type = pt.id
            WHERE mods.id = $1
            ",
                version.project_id as models::ProjectId,
            )
            .fetch_one(&mut **transaction)
            .await?
            .name;

            let version_data = initial_version_data
                .clone()
                .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;

            upload_file(
                &mut field,
                file_host,
                version_data.file_parts.len(),
                uploaded_files,
                &mut version.files,
                &mut version.dependencies,
                &cdn_url,
                &content_disposition,
                version.project_id.into(),
                version.version_id.into(),
                &project_type,
                version_data.loaders,
                version_data.game_versions,
                all_game_versions.clone(),
                version_data.primary_file.is_some(),
                version_data.primary_file.as_deref() == Some(name),
                version_data.file_types.get(name).copied().flatten(),
                transaction,
            )
            .await?;

            Ok(())
        }
        .await;

        if result.is_err() {
            error = result.err();
        }
    }

    if let Some(error) = error {
        return Err(error);
    }

    let version_data = initial_version_data
        .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;
    let builder = version_builder
        .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;

    if builder.files.is_empty() {
        return Err(CreateError::InvalidInput(
            "Versions must have at least one file uploaded to them".to_string(),
        ));
    }

    use futures::stream::TryStreamExt;

    let users = sqlx::query!(
        "
        SELECT follower_id FROM mod_follows
        WHERE mod_id = $1
        ",
        builder.project_id as crate::database::models::ids::ProjectId
    )
    .fetch_many(&mut **transaction)
    .try_filter_map(|e| async { Ok(e.right().map(|m| models::ids::UserId(m.follower_id))) })
    .try_collect::<Vec<models::ids::UserId>>()
    .await?;

    let project_id: ProjectId = builder.project_id.into();
    let version_id: VersionId = builder.version_id.into();

    NotificationBuilder {
        body: NotificationBody::ProjectUpdate {
            project_id,
            version_id,
        },
    }
    .insert_many(users, transaction, redis)
    .await?;

    let response = Version {
        id: builder.version_id.into(),
        project_id: builder.project_id.into(),
        author_id: user.id,
        featured: builder.featured,
        name: builder.name.clone(),
        version_number: builder.version_number.clone(),
        changelog: builder.changelog.clone(),
        changelog_url: None,
        date_published: Utc::now(),
        downloads: 0,
        version_type: version_data.release_channel,
        status: builder.status,
        requested_status: builder.requested_status,
        files: builder
            .files
            .iter()
            .map(|file| VersionFile {
                hashes: file
                    .hashes
                    .iter()
                    .map(|hash| {
                        (
                            hash.algorithm.clone(),
                            // This is a hack since the hashes are currently stored as ASCII
                            // in the database, but represented here as a Vec<u8>.  At some
                            // point we need to change the hash to be the real bytes  in the
                            // database and add more processing here.
                            String::from_utf8(hash.hash.clone()).unwrap(),
                        )
                    })
                    .collect(),
                url: file.url.clone(),
                filename: file.filename.clone(),
                primary: file.primary,
                size: file.size,
                file_type: file.file_type,
            })
            .collect::<Vec<_>>(),
        dependencies: version_data.dependencies,
        game_versions: version_data.game_versions,
        loaders: version_data.loaders,
    };

    let project_id = builder.project_id;
    builder.insert(transaction).await?;

    for image_id in version_data.uploaded_images {
        if let Some(db_image) =
            image_item::Image::get(image_id.into(), &mut **transaction, redis).await?
        {
            let image: Image = db_image.into();
            if !matches!(image.context, ImageContext::Report { .. })
                || image.context.inner_id().is_some()
            {
                return Err(CreateError::InvalidInput(format!(
                    "Image {} is not unused and in the 'version' context",
                    image_id
                )));
            }

            sqlx::query!(
                "
                UPDATE uploaded_images
                SET version_id = $1
                WHERE id = $2
                ",
                version_id.0 as i64,
                image_id.0 as i64
            )
            .execute(&mut **transaction)
            .await?;

            image_item::Image::clear_cache(image.id.into(), redis).await?;
        } else {
            return Err(CreateError::InvalidInput(format!(
                "Image {} does not exist",
                image_id
            )));
        }
    }

    models::Project::update_game_versions(project_id, transaction).await?;
    models::Project::update_loaders(project_id, transaction).await?;
    models::Project::clear_cache(project_id, None, Some(true), redis).await?;

    Ok(HttpResponse::Ok().json(response))
}

// under /api/v1/version/{version_id}
#[post("{version_id}/file")]
pub async fn upload_file_to_version(
    req: HttpRequest,
    url_data: web::Path<(VersionId,)>,
    mut payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let version_id = models::VersionId::from(url_data.into_inner().0);

    let result = upload_file_to_version_inner(
        req,
        &mut payload,
        client,
        &mut transaction,
        redis,
        &***file_host,
        &mut uploaded_files,
        version_id,
        &session_queue,
    )
    .await;

    if result.is_err() {
        let undo_result =
            super::project_creation::undo_uploads(&***file_host, &uploaded_files).await;
        let rollback_result = transaction.rollback().await;

        undo_result?;
        if let Err(e) = rollback_result {
            return Err(e.into());
        }
    } else {
        transaction.commit().await?;
    }

    result
}

#[allow(clippy::too_many_arguments)]
async fn upload_file_to_version_inner(
    req: HttpRequest,
    payload: &mut Multipart,
    client: Data<PgPool>,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    redis: Data<RedisPool>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    version_id: models::VersionId,
    session_queue: &AuthQueue,
) -> Result<HttpResponse, CreateError> {
    let cdn_url = dotenvy::var("CDN_URL")?;

    let mut initial_file_data: Option<InitialFileData> = None;
    let mut file_builders: Vec<VersionFileBuilder> = Vec::new();

    let user = get_user_from_headers(
        &req,
        &**client,
        &redis,
        session_queue,
        Some(&[Scopes::VERSION_WRITE]),
    )
    .await?
    .1;

    let result = models::Version::get(version_id, &**client, &redis).await?;

    let version = match result {
        Some(v) => v,
        None => {
            return Err(CreateError::InvalidInput(
                "An invalid version id was supplied".to_string(),
            ));
        }
    };

    if !user.role.is_admin() {
        let team_member = models::TeamMember::get_from_user_id_project(
            version.inner.project_id,
            user.id.into(),
            &mut **transaction,
        )
        .await?;

        let organization = Organization::get_associated_organization_project_id(
            version.inner.project_id,
            &**client,
        )
        .await?;

        let organization_team_member = if let Some(organization) = &organization {
            models::TeamMember::get_from_user_id(
                organization.team_id,
                user.id.into(),
                &mut **transaction,
            )
            .await?
        } else {
            None
        };

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::UPLOAD_VERSION) {
            return Err(CreateError::CustomAuthenticationError(
                "You don't have permission to upload files to this version!".to_string(),
            ));
        }
    }

    let project_id = ProjectId(version.inner.project_id.0 as u64);

    let project_type = sqlx::query!(
        "
        SELECT name FROM project_types pt
        INNER JOIN mods ON mods.project_type = pt.id
        WHERE mods.id = $1
        ",
        version.inner.project_id as models::ProjectId,
    )
    .fetch_one(&mut **transaction)
    .await?
    .name;

    let all_game_versions =
        models::categories::GameVersion::list(&mut **transaction, &redis).await?;

    let mut error = None;
    while let Some(item) = payload.next().await {
        let mut field: Field = item?;

        if error.is_some() {
            continue;
        }

        let result = async {
            let content_disposition = field.content_disposition().clone();
            let name = content_disposition.get_name().ok_or_else(|| {
                CreateError::MissingValueError("Missing content name".to_string())
            })?;

            if name == "data" {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk?);
                }
                let file_data: InitialFileData = serde_json::from_slice(&data)?;

                initial_file_data = Some(file_data);
                return Ok(());
            }

            let file_data = initial_file_data.as_ref().ok_or_else(|| {
                CreateError::InvalidInput(String::from("`data` field must come before file fields"))
            })?;

            let mut dependencies = version
                .dependencies
                .iter()
                .map(|x| DependencyBuilder {
                    project_id: x.project_id,
                    version_id: x.version_id,
                    file_name: x.file_name.clone(),
                    dependency_type: x.dependency_type.clone(),
                })
                .collect();

            upload_file(
                &mut field,
                file_host,
                0,
                uploaded_files,
                &mut file_builders,
                &mut dependencies,
                &cdn_url,
                &content_disposition,
                project_id,
                version_id.into(),
                &project_type,
                version.loaders.clone().into_iter().map(Loader).collect(),
                version
                    .game_versions
                    .clone()
                    .into_iter()
                    .map(GameVersion)
                    .collect(),
                all_game_versions.clone(),
                true,
                false,
                file_data.file_types.get(name).copied().flatten(),
                transaction,
            )
            .await?;

            Ok(())
        }
        .await;

        if result.is_err() {
            error = result.err();
        }
    }

    if let Some(error) = error {
        return Err(error);
    }

    if file_builders.is_empty() {
        return Err(CreateError::InvalidInput(
            "At least one file must be specified".to_string(),
        ));
    } else {
        VersionFileBuilder::insert_many(file_builders, version_id, transaction).await?;
    }

    // Clear version cache
    models::Version::clear_cache(&version, &redis).await?;

    Ok(HttpResponse::NoContent().body(""))
}

// This function is used for adding a file to a version, uploading the initial
// files for a version, and for uploading the initial version files for a project
#[allow(clippy::too_many_arguments)]
pub async fn upload_file(
    field: &mut Field,
    file_host: &dyn FileHost,
    total_files_len: usize,
    uploaded_files: &mut Vec<UploadedFile>,
    version_files: &mut Vec<VersionFileBuilder>,
    dependencies: &mut Vec<DependencyBuilder>,
    cdn_url: &str,
    content_disposition: &actix_web::http::header::ContentDisposition,
    project_id: ProjectId,
    version_id: VersionId,
    project_type: &str,
    loaders: Vec<Loader>,
    game_versions: Vec<GameVersion>,
    all_game_versions: Vec<models::categories::GameVersion>,
    ignore_primary: bool,
    force_primary: bool,
    file_type: Option<FileType>,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), CreateError> {
    let (file_name, file_extension) = get_name_ext(content_disposition)?;

    if file_name.contains('/') {
        return Err(CreateError::InvalidInput(
            "File names must not contain slashes!".to_string(),
        ));
    }

    let content_type = crate::util::ext::project_file_type(file_extension)
        .ok_or_else(|| CreateError::InvalidFileType(file_extension.to_string()))?;

    let data = read_from_field(
        field, 500 * (1 << 20),
        "Project file exceeds the maximum of 500MiB. Contact a moderator or admin to request permission to upload larger files."
    ).await?;

    let hash = sha1::Sha1::from(&data).hexdigest();
    let exists = sqlx::query!(
        "
        SELECT EXISTS(SELECT 1 FROM hashes h
        INNER JOIN files f ON f.id = h.file_id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = $1 AND v.mod_id != $3)
        ",
        hash.as_bytes(),
        "sha1",
        project_id.0 as i64
    )
    .fetch_one(&mut **transaction)
    .await?
    .exists
    .unwrap_or(false);

    if exists {
        return Err(CreateError::InvalidInput(
            "Duplicate files are not allowed to be uploaded to Modrinth!".to_string(),
        ));
    }

    let validation_result = validate_file(
        data.clone().into(),
        file_extension.to_string(),
        project_type.to_string(),
        loaders.clone(),
        game_versions.clone(),
        all_game_versions.clone(),
        file_type,
    )
    .await?;

    if let ValidationResult::PassWithPackDataAndFiles {
        ref format,
        ref files,
    } = validation_result
    {
        if dependencies.is_empty() {
            let hashes: Vec<Vec<u8>> = format
                .files
                .iter()
                .filter_map(|x| x.hashes.get(&PackFileHash::Sha1))
                .map(|x| x.as_bytes().to_vec())
                .collect();

            let res = sqlx::query!(
                "
                    SELECT v.id version_id, v.mod_id project_id, h.hash hash FROM hashes h
                    INNER JOIN files f on h.file_id = f.id
                    INNER JOIN versions v on f.version_id = v.id
                    WHERE h.algorithm = 'sha1' AND h.hash = ANY($1)
                    ",
                &*hashes
            )
            .fetch_all(&mut **transaction)
            .await?;

            for file in &format.files {
                if let Some(dep) = res.iter().find(|x| {
                    Some(&*x.hash) == file.hashes.get(&PackFileHash::Sha1).map(|x| x.as_bytes())
                }) {
                    dependencies.push(DependencyBuilder {
                        project_id: Some(models::ProjectId(dep.project_id)),
                        version_id: Some(models::VersionId(dep.version_id)),
                        file_name: None,
                        dependency_type: DependencyType::Embedded.to_string(),
                    });
                } else if let Some(first_download) = file.downloads.first() {
                    dependencies.push(DependencyBuilder {
                        project_id: None,
                        version_id: None,
                        file_name: Some(
                            first_download
                                .rsplit('/')
                                .next()
                                .unwrap_or(first_download)
                                .to_string(),
                        ),
                        dependency_type: DependencyType::Embedded.to_string(),
                    });
                }
            }

            for file in files {
                if !file.is_empty() {
                    dependencies.push(DependencyBuilder {
                        project_id: None,
                        version_id: None,
                        file_name: Some(file.to_string()),
                        dependency_type: DependencyType::Embedded.to_string(),
                    });
                }
            }
        }
    }

    let data = data.freeze();

    let primary = (validation_result.is_passed()
        && version_files.iter().all(|x| !x.primary)
        && !ignore_primary)
        || force_primary
        || total_files_len == 1;

    let file_path_encode = format!(
        "data/{}/versions/{}/{}",
        project_id,
        version_id,
        urlencoding::encode(file_name)
    );
    let file_path = format!("data/{}/versions/{}/{}", project_id, version_id, &file_name);

    let upload_data = file_host
        .upload_file(content_type, &file_path, data)
        .await?;

    uploaded_files.push(UploadedFile {
        file_id: upload_data.file_id,
        file_name: file_path,
    });

    let sha1_bytes = upload_data.content_sha1.into_bytes();
    let sha512_bytes = upload_data.content_sha512.into_bytes();

    if version_files.iter().any(|x| {
        x.hashes
            .iter()
            .any(|y| y.hash == sha1_bytes || y.hash == sha512_bytes)
    }) {
        return Err(CreateError::InvalidInput(
            "Duplicate files are not allowed to be uploaded to Modrinth!".to_string(),
        ));
    }

    if let ValidationResult::Warning(msg) = validation_result {
        if primary {
            return Err(CreateError::InvalidInput(msg.to_string()));
        }
    }

    version_files.push(VersionFileBuilder {
        filename: file_name.to_string(),
        url: format!("{cdn_url}/{file_path_encode}"),
        hashes: vec![
            models::version_item::HashBuilder {
                algorithm: "sha1".to_string(),
                // This is an invalid cast - the database expects the hash's
                // bytes, but this is the string version.
                hash: sha1_bytes,
            },
            models::version_item::HashBuilder {
                algorithm: "sha512".to_string(),
                // This is an invalid cast - the database expects the hash's
                // bytes, but this is the string version.
                hash: sha512_bytes,
            },
        ],
        primary,
        size: upload_data.content_length,
        file_type,
    });

    Ok(())
}

pub fn get_name_ext(
    content_disposition: &actix_web::http::header::ContentDisposition,
) -> Result<(&str, &str), CreateError> {
    let file_name = content_disposition
        .get_filename()
        .ok_or_else(|| CreateError::MissingValueError("Missing content file name".to_string()))?;
    let file_extension = if let Some(last_period) = file_name.rfind('.') {
        file_name.get((last_period + 1)..).unwrap_or("")
    } else {
        return Err(CreateError::MissingValueError(
            "Missing content file extension".to_string(),
        ));
    };
    Ok((file_name, file_extension))
}
