use crate::database::models;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::version_item::{
    VersionBuilder, VersionFileBuilder,
};
use crate::file_hosting::FileHost;
use crate::models::projects::{
    Dependency, GameVersion, Loader, ProjectId, Version, VersionFile,
    VersionId, VersionType,
};
use crate::models::teams::Permissions;
use crate::routes::project_creation::{CreateError, UploadedFile};
use crate::util::auth::get_user_from_headers;
use crate::util::routes::read_from_field;
use crate::util::validate::validation_errors_to_string;
use crate::validate::{validate_file, ValidationResult};
use actix_multipart::{Field, Multipart};
use actix_web::web::Data;
use actix_web::{post, HttpRequest, HttpResponse};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct InitialVersionData {
    #[serde(alias = "mod_id")]
    pub project_id: Option<ProjectId>,
    #[validate(length(min = 1, max = 256))]
    pub file_parts: Vec<String>,
    #[validate(
        length(min = 1, max = 64),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    pub version_number: String,
    #[validate(length(min = 3, max = 256))]
    #[serde(alias = "name")]
    pub version_title: String,
    #[validate(length(max = 65536))]
    #[serde(alias = "changelog")]
    pub version_body: Option<String>,
    #[validate(
        length(min = 0, max = 256),
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
}

#[derive(Serialize, Deserialize, Clone)]
struct InitialFileData {
    // TODO: hashes?
}

// under `/api/v1/version`
#[post("version")]
pub async fn version_create(
    req: HttpRequest,
    payload: Multipart,
    client: Data<PgPool>,
    file_host: Data<std::sync::Arc<dyn FileHost + Send + Sync>>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let result = version_create_inner(
        req,
        payload,
        &mut transaction,
        &***file_host,
        &mut uploaded_files,
    )
    .await;

    if result.is_err() {
        let undo_result = super::project_creation::undo_uploads(
            &***file_host,
            &uploaded_files,
        )
        .await;
        let rollback_result = transaction.rollback().await;

        if let Err(e) = undo_result {
            return Err(e);
        }
        if let Err(e) = rollback_result {
            return Err(e.into());
        }
    } else {
        transaction.commit().await?;
    }

    result
}

async fn version_create_inner(
    req: HttpRequest,
    mut payload: Multipart,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
) -> Result<HttpResponse, CreateError> {
    let cdn_url = dotenv::var("CDN_URL")?;

    let mut initial_version_data = None;
    let mut version_builder = None;

    let all_game_versions =
        models::categories::GameVersion::list(&mut *transaction).await?;
    let all_loaders =
        models::categories::Loader::list(&mut *transaction).await?;

    let user = get_user_from_headers(req.headers(), &mut *transaction).await?;

    while let Some(item) = payload.next().await {
        let mut field: Field = item.map_err(CreateError::MultipartError)?;
        let content_disposition = field.content_disposition().clone();
        let name = content_disposition.get_name().ok_or_else(|| {
            CreateError::MissingValueError("Missing content name".to_string())
        })?;

        if name == "data" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(
                    &chunk.map_err(CreateError::MultipartError)?,
                );
            }

            let version_create_data: InitialVersionData =
                serde_json::from_slice(&data)?;
            initial_version_data = Some(version_create_data);
            let version_create_data = initial_version_data.as_ref().unwrap();
            if version_create_data.project_id.is_none() {
                return Err(CreateError::MissingValueError(
                    "Missing project id".to_string(),
                ));
            }

            version_create_data.validate().map_err(|err| {
                CreateError::ValidationError(validation_errors_to_string(
                    err, None,
                ))
            })?;

            let project_id: models::ProjectId =
                version_create_data.project_id.unwrap().into();

            // Ensure that the project this version is being added to exists
            let results = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM mods WHERE id=$1)",
                project_id as models::ProjectId
            )
            .fetch_one(&mut *transaction)
            .await?;

            if !results.exists.unwrap_or(false) {
                return Err(CreateError::InvalidInput(
                    "An invalid project id was supplied".to_string(),
                ));
            }

            // Check whether there is already a version of this project with the
            // same version number
            let results = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM versions WHERE (version_number=$1) AND (mod_id=$2))",
                version_create_data.version_number,
                project_id as models::ProjectId,
            )
            .fetch_one(&mut *transaction)
            .await?;

            if results.exists.unwrap_or(true) {
                return Err(CreateError::InvalidInput(
                    "A version with that version_number already exists"
                        .to_string(),
                ));
            }

            // Check that the user creating this version is a team member
            // of the project the version is being added to.
            let team_member = models::TeamMember::get_from_user_id_project(
                project_id,
                user.id.into(),
                &mut *transaction,
            )
            .await?
            .ok_or_else(|| {
                CreateError::CustomAuthenticationError(
                    "You don't have permission to upload this version!"
                        .to_string(),
                )
            })?;

            if !team_member
                .permissions
                .contains(Permissions::UPLOAD_VERSION)
            {
                return Err(CreateError::CustomAuthenticationError(
                    "You don't have permission to upload this version!"
                        .to_string(),
                ));
            }

            let version_id: VersionId =
                models::generate_version_id(transaction).await?.into();

            let project_type = sqlx::query!(
                "
                SELECT name FROM project_types pt
                INNER JOIN mods ON mods.project_type = pt.id
                WHERE mods.id = $1
                ",
                project_id as models::ProjectId,
            )
            .fetch_one(&mut *transaction)
            .await?
            .name;

            let game_versions = version_create_data
                .game_versions
                .iter()
                .map(|x| {
                    all_game_versions
                        .iter()
                        .find(|y| y.version == x.0)
                        .ok_or_else(|| {
                            CreateError::InvalidGameVersion(x.0.clone())
                        })
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
                            y.loader == x.0
                                && y.supported_project_types
                                    .contains(&project_type)
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
                })
                .collect::<Vec<_>>();

            version_builder = Some(VersionBuilder {
                version_id: version_id.into(),
                project_id,
                author_id: user.id.into(),
                name: version_create_data.version_title.clone(),
                version_number: version_create_data.version_number.clone(),
                changelog: version_create_data
                    .version_body
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
                files: Vec::new(),
                dependencies,
                game_versions,
                loaders,
                version_type: version_create_data.release_channel.to_string(),
                featured: version_create_data.featured,
            });

            continue;
        }

        let version = version_builder.as_mut().ok_or_else(|| {
            CreateError::InvalidInput(String::from(
                "`data` field must come before file fields",
            ))
        })?;

        let project_type = sqlx::query!(
            "
                SELECT name FROM project_types pt
                INNER JOIN mods ON mods.project_type = pt.id
                WHERE mods.id = $1
                ",
            version.project_id as models::ProjectId,
        )
        .fetch_one(&mut *transaction)
        .await?
        .name;

        let version_data = initial_version_data.clone().ok_or_else(|| {
            CreateError::InvalidInput("`data` field is required".to_string())
        })?;

        upload_file(
            &mut field,
            file_host,
            uploaded_files,
            &mut version.files,
            &cdn_url,
            &content_disposition,
            version.project_id.into(),
            &version.version_number,
            &*project_type,
            version_data.loaders,
            version_data.game_versions,
            all_game_versions.clone(),
            version_data.primary_file.is_some(),
            version_data.primary_file.as_deref() == Some(name),
            transaction,
        )
        .await?;
    }

    let version_data = initial_version_data.ok_or_else(|| {
        CreateError::InvalidInput("`data` field is required".to_string())
    })?;
    let builder = version_builder.ok_or_else(|| {
        CreateError::InvalidInput("`data` field is required".to_string())
    })?;

    if builder.files.is_empty() {
        return Err(CreateError::InvalidInput(
            "Versions must have at least one file uploaded to them".to_string(),
        ));
    }

    let result = sqlx::query!(
        "
        SELECT m.title title, pt.name project_type
        FROM mods m
        INNER JOIN project_types pt ON pt.id = m.project_type
        WHERE m.id = $1
        ",
        builder.project_id as crate::database::models::ids::ProjectId
    )
    .fetch_one(&mut *transaction)
    .await?;

    use futures::stream::TryStreamExt;

    let users = sqlx::query!(
        "
            SELECT follower_id FROM mod_follows
            WHERE mod_id = $1
            ",
        builder.project_id as crate::database::models::ids::ProjectId
    )
    .fetch_many(&mut *transaction)
    .try_filter_map(|e| async {
        Ok(e.right()
            .map(|m| crate::database::models::ids::UserId(m.follower_id)))
    })
    .try_collect::<Vec<crate::database::models::ids::UserId>>()
    .await?;

    let project_id: ProjectId = builder.project_id.into();
    let version_id: VersionId = builder.version_id.into();

    NotificationBuilder {
        notification_type: Some("project_update".to_string()),
        title: format!("**{}** has been updated!", result.title),
        text: format!(
            "The project, {}, has released a new version: {}",
            result.title,
            version_data.version_number.clone()
        ),
        link: format!(
            "/{}/{}/version/{}",
            result.project_type, project_id, version_id
        ),
        actions: vec![],
    }
    .insert_many(users, &mut *transaction)
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
        date_published: chrono::Utc::now(),
        downloads: 0,
        version_type: version_data.release_channel,
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
            })
            .collect::<Vec<_>>(),
        dependencies: version_data.dependencies,
        game_versions: version_data.game_versions,
        loaders: version_data.loaders,
    };

    builder.insert(transaction).await?;

    Ok(HttpResponse::Ok().json(response))
}

// TODO: file deletion, listing, etc

// under /api/v1/version/{version_id}
#[post("{version_id}/file")]
pub async fn upload_file_to_version(
    req: HttpRequest,
    url_data: actix_web::web::Path<(VersionId,)>,
    payload: Multipart,
    client: Data<PgPool>,
    file_host: Data<std::sync::Arc<dyn FileHost + Send + Sync>>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let version_id = models::VersionId::from(url_data.into_inner().0);

    let result = upload_file_to_version_inner(
        req,
        payload,
        client,
        &mut transaction,
        &***file_host,
        &mut uploaded_files,
        version_id,
    )
    .await;

    if result.is_err() {
        let undo_result = super::project_creation::undo_uploads(
            &***file_host,
            &uploaded_files,
        )
        .await;
        let rollback_result = transaction.rollback().await;

        if let Err(e) = undo_result {
            return Err(e);
        }
        if let Err(e) = rollback_result {
            return Err(e.into());
        }
    } else {
        transaction.commit().await?;
    }

    result
}

async fn upload_file_to_version_inner(
    req: HttpRequest,
    mut payload: Multipart,
    client: Data<PgPool>,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    version_id: models::VersionId,
) -> Result<HttpResponse, CreateError> {
    let cdn_url = dotenv::var("CDN_URL")?;

    let mut initial_file_data: Option<InitialFileData> = None;
    let mut file_builders: Vec<VersionFileBuilder> = Vec::new();

    let user = get_user_from_headers(req.headers(), &mut *transaction).await?;

    let result = models::Version::get_full(version_id, &**client).await?;

    let version = match result {
        Some(v) => v,
        None => {
            return Err(CreateError::InvalidInput(
                "An invalid version id was supplied".to_string(),
            ));
        }
    };

    let team_member = models::TeamMember::get_from_user_id_version(
        version_id,
        user.id.into(),
        &mut *transaction,
    )
    .await?
    .ok_or_else(|| {
        CreateError::CustomAuthenticationError(
            "You don't have permission to upload files to this version!"
                .to_string(),
        )
    })?;

    if !team_member
        .permissions
        .contains(Permissions::UPLOAD_VERSION)
    {
        return Err(CreateError::CustomAuthenticationError(
            "You don't have permission to upload files to this version!"
                .to_string(),
        ));
    }

    let project_id = ProjectId(version.project_id.0 as u64);
    let version_number = version.version_number;

    let project_type = sqlx::query!(
        "
        SELECT name FROM project_types pt
        INNER JOIN mods ON mods.project_type = pt.id
        WHERE mods.id = $1
        ",
        version.project_id as models::ProjectId,
    )
    .fetch_one(&mut *transaction)
    .await?
    .name;

    let all_game_versions =
        models::categories::GameVersion::list(&mut *transaction).await?;

    while let Some(item) = payload.next().await {
        let mut field: Field = item.map_err(CreateError::MultipartError)?;
        let content_disposition = field.content_disposition().clone();
        let name = content_disposition.get_name().ok_or_else(|| {
            CreateError::MissingValueError("Missing content name".to_string())
        })?;

        if name == "data" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(
                    &chunk.map_err(CreateError::MultipartError)?,
                );
            }
            let file_data: InitialFileData = serde_json::from_slice(&data)?;
            // TODO: currently no data here, but still required

            initial_file_data = Some(file_data);
            continue;
        }

        let _file_data = initial_file_data.as_ref().ok_or_else(|| {
            CreateError::InvalidInput(String::from(
                "`data` field must come before file fields",
            ))
        })?;

        upload_file(
            &mut field,
            file_host,
            uploaded_files,
            &mut file_builders,
            &cdn_url,
            &content_disposition,
            project_id,
            &version_number,
            &*project_type,
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
            transaction,
        )
        .await?;
    }

    if file_builders.is_empty() {
        return Err(CreateError::InvalidInput(
            "At least one file must be specified".to_string(),
        ));
    } else {
        for file_builder in file_builders {
            file_builder.insert(version_id, &mut *transaction).await?;
        }
    }

    Ok(HttpResponse::Ok().into())
}

// This function is used for adding a file to a version, uploading the initial
// files for a version, and for uploading the initial version files for a project
#[allow(clippy::too_many_arguments)]
pub async fn upload_file(
    field: &mut Field,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    version_files: &mut Vec<models::version_item::VersionFileBuilder>,
    cdn_url: &str,
    content_disposition: &actix_web::http::header::ContentDisposition,
    project_id: crate::models::ids::ProjectId,
    version_number: &str,
    project_type: &str,
    loaders: Vec<Loader>,
    game_versions: Vec<GameVersion>,
    all_game_versions: Vec<models::categories::GameVersion>,
    ignore_primary: bool,
    force_primary: bool,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), CreateError> {
    let (file_name, file_extension) = get_name_ext(content_disposition)?;

    let content_type = crate::util::ext::project_file_type(file_extension)
        .ok_or_else(|| {
            CreateError::InvalidFileType(file_extension.to_string())
        })?;

    let data = read_from_field(
        field, 100 * (1 << 20),
        "Project file exceeds the maximum of 100MiB. Contact a moderator or admin to request permission to upload larger files."
    ).await?;

    let hash = sha1::Sha1::from(&data).hexdigest();
    let exists = sqlx::query!(
        "
        SELECT EXISTS(SELECT 1 FROM hashes h
        WHERE h.algorithm = $2 AND h.hash = $1)
        ",
        hash.as_bytes(),
        "sha1"
    )
    .fetch_one(&mut *transaction)
    .await?
    .exists
    .unwrap_or(false);

    if exists {
        return Err(CreateError::InvalidInput(
            "Duplicate files are not allowed to be uploaded to Modrinth!"
                .to_string(),
        ));
    }

    let validation_result = validate_file(
        data.clone().into(),
        file_extension.to_string(),
        project_type.to_string(),
        loaders,
        game_versions,
        all_game_versions,
    )
    .await?;

    let file_path_encode = format!(
        "data/{}/versions/{}/{}",
        project_id,
        version_number,
        urlencoding::encode(file_name)
    );
    let file_path = format!(
        "data/{}/versions/{}/{}",
        project_id, version_number, &file_name
    );

    let upload_data = file_host
        .upload_file(content_type, &file_path, data.freeze())
        .await?;

    uploaded_files.push(UploadedFile {
        file_id: upload_data.file_id,
        file_name: file_path,
    });

    version_files.push(models::version_item::VersionFileBuilder {
        filename: file_name.to_string(),
        url: format!("{}/{}", cdn_url, file_path_encode),
        hashes: vec![
            models::version_item::HashBuilder {
                algorithm: "sha1".to_string(),
                // This is an invalid cast - the database expects the hash's
                // bytes, but this is the string version.
                hash: upload_data.content_sha1.into_bytes(),
            },
            models::version_item::HashBuilder {
                algorithm: "sha512".to_string(),
                // This is an invalid cast - the database expects the hash's
                // bytes, but this is the string version.
                hash: upload_data.content_sha512.into_bytes(),
            },
        ],
        primary: (validation_result == ValidationResult::Pass
            && version_files.iter().all(|x| !x.primary)
            && !ignore_primary)
            || force_primary,
    });

    Ok(())
}

pub fn get_name_ext(
    content_disposition: &actix_web::http::header::ContentDisposition,
) -> Result<(&str, &str), CreateError> {
    let file_name = content_disposition.get_filename().ok_or_else(|| {
        CreateError::MissingValueError("Missing content file name".to_string())
    })?;
    let file_extension = if let Some(last_period) = file_name.rfind('.') {
        file_name.get((last_period + 1)..).unwrap_or("")
    } else {
        return Err(CreateError::MissingValueError(
            "Missing content file extension".to_string(),
        ));
    };
    Ok((file_name, file_extension))
}
