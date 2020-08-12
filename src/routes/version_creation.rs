use crate::database::models;
use crate::database::models::version_item::{VersionBuilder, VersionFileBuilder};
use crate::file_hosting::FileHost;
use crate::models::mods::{
    GameVersion, ModId, ModLoader, Version, VersionFile, VersionId, VersionType,
};
use crate::routes::mod_creation::{CreateError, UploadedFile};
use actix_multipart::{Field, Multipart};
use actix_web::web::Data;
use actix_web::{post, HttpResponse};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Clone)]
pub struct InitialVersionData {
    pub file_parts: Vec<String>,
    pub version_number: String,
    pub version_title: String,
    pub version_body: String,
    pub dependencies: Vec<VersionId>,
    pub game_versions: Vec<GameVersion>,
    pub release_channel: VersionType,
    pub loaders: Vec<ModLoader>,
}

#[derive(Serialize, Deserialize, Clone)]
struct InitialFileData {
    // TODO: hashes?
}

// under `/api/v1/mod/{mod_id}`
#[post("version")]
pub async fn version_create(
    url_data: actix_web::web::Path<(ModId,)>,
    payload: Multipart,
    client: Data<PgPool>,
    file_host: Data<std::sync::Arc<dyn FileHost + Send + Sync>>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let mod_id = url_data.into_inner().0.into();

    let result = version_create_inner(
        payload,
        &mut transaction,
        &***file_host,
        &mut uploaded_files,
        mod_id,
    )
    .await;

    if result.is_err() {
        let undo_result = super::mod_creation::undo_uploads(&***file_host, &uploaded_files).await;
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
    mut payload: Multipart,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    mod_id: models::ModId,
) -> Result<HttpResponse, CreateError> {
    let cdn_url = dotenv::var("CDN_URL")?;

    let mut initial_version_data = None;
    let mut version_builder = None;

    while let Some(item) = payload.next().await {
        let mut field: Field = item.map_err(CreateError::MultipartError)?;
        let content_disposition = field.content_disposition().ok_or_else(|| {
            CreateError::MissingValueError("Missing content disposition".to_string())
        })?;
        let name = content_disposition
            .get_name()
            .ok_or_else(|| CreateError::MissingValueError("Missing content name".to_string()))?;

        if name == "data" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.map_err(CreateError::MultipartError)?);
            }

            let version_create_data: InitialVersionData = serde_json::from_slice(&data)?;
            initial_version_data = Some(version_create_data);
            let version_create_data = initial_version_data.as_ref().unwrap();

            let results = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM mods WHERE id=$1)",
                mod_id as models::ModId
            )
            .fetch_one(&mut *transaction)
            .await?;

            if !results.exists.unwrap_or(false) {
                return Err(CreateError::InvalidInput(
                    "An invalid mod id was supplied".to_string(),
                ));
            }

            let results = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM versions WHERE (version_number=$1) AND (mod_id=$2))",
                version_create_data.version_number,
                mod_id as models::ModId,
            )
            .fetch_one(&mut *transaction)
            .await?;

            if results.exists.unwrap_or(true) {
                return Err(CreateError::InvalidInput(
                    "A version with that version_number already exists".to_string(),
                ));
            }

            let version_id: VersionId = models::generate_version_id(transaction).await?.into();
            let body_url = format!(
                "data/{}/changelogs/{}/body.md",
                ModId::from(mod_id),
                version_id
            );

            let uploaded_text = file_host
                .upload_file(
                    "text/plain",
                    &body_url,
                    version_create_data.version_body.clone().into_bytes(),
                )
                .await?;

            uploaded_files.push(UploadedFile {
                file_id: uploaded_text.file_id.clone(),
                file_name: uploaded_text.file_name.clone(),
            });

            // TODO: do a real lookup for the channels
            let release_channel = match version_create_data.release_channel {
                VersionType::Release => models::ChannelId(1),
                VersionType::Beta => models::ChannelId(3),
                VersionType::Alpha => models::ChannelId(5),
            };

            version_builder = Some(VersionBuilder {
                version_id: version_id.into(),
                mod_id,
                name: version_create_data.version_title.clone(),
                version_number: version_create_data.version_number.clone(),
                changelog_url: Some(format!("{}/{}", cdn_url, body_url)),
                files: Vec::with_capacity(1),
                dependencies: version_create_data
                    .dependencies
                    .iter()
                    .map(|x| (*x).into())
                    .collect::<Vec<_>>(),
                // TODO: add game_versions and loaders info
                game_versions: vec![],
                loaders: vec![],
                release_channel,
            });

            continue;
        }

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

        if &*file_extension == "jar" {
            let version = version_builder.as_mut().ok_or_else(|| {
                CreateError::InvalidInput(String::from("`data` field must come before file fields"))
            })?;

            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.map_err(CreateError::MultipartError)?);
            }

            let upload_data = file_host
                .upload_file(
                    "application/java-archive",
                    &format!(
                        "{}/{}/{}",
                        ModId::from(version.mod_id),
                        version.version_number,
                        file_name
                    ),
                    data.to_vec(),
                )
                .await?;

            uploaded_files.push(UploadedFile {
                file_id: upload_data.file_id.clone(),
                file_name: upload_data.file_name.clone(),
            });

            // Add the newly uploaded file to the existing or new version

            // TODO: Malware scan + file validation
            version
                .files
                .push(models::version_item::VersionFileBuilder {
                    filename: file_name.to_string(),
                    url: format!("{}/{}", cdn_url, upload_data.file_name),
                    hashes: vec![models::version_item::HashBuilder {
                        algorithm: "sha1".to_string(),
                        // This is an invalid cast - the database expects the hash's
                        // bytes, but this is the string version.
                        hash: upload_data.content_sha1.into_bytes(),
                    }],
                });
        }
    }

    let version_data_safe = initial_version_data
        .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;
    let version_builder_safe = version_builder
        .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;

    let response = Version {
        id: version_builder_safe.version_id.into(),
        mod_id: version_builder_safe.mod_id.into(),
        name: version_builder_safe.name.clone(),
        version_number: version_builder_safe.version_number.clone(),
        changelog_url: version_builder_safe.changelog_url.clone(),
        date_published: chrono::Utc::now(),
        downloads: 0,
        version_type: version_data_safe.release_channel,
        files: version_builder_safe
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
            })
            .collect::<Vec<_>>(),
        dependencies: version_data_safe.dependencies,
        game_versions: version_data_safe.game_versions,
        loaders: version_data_safe.loaders,
    };

    version_builder_safe.insert(transaction).await?;

    Ok(HttpResponse::Ok().json(response))
}

// TODO: file deletion, listing, etc

// under /api/v1/mod/{mod_id}/version/{version_id}
#[post("file")]
pub async fn upload_file_to_version(
    url_data: actix_web::web::Path<(ModId, VersionId)>,
    payload: Multipart,
    client: Data<PgPool>,
    file_host: Data<std::sync::Arc<dyn FileHost + Send + Sync>>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let data = url_data.into_inner();
    let mod_id = models::ModId::from(data.0);
    let version_id = models::VersionId::from(data.1);

    let result = upload_file_to_version_inner(
        payload,
        &mut transaction,
        &***file_host,
        &mut uploaded_files,
        version_id,
        mod_id,
    )
    .await;

    if result.is_err() {
        let undo_result = super::mod_creation::undo_uploads(&***file_host, &uploaded_files).await;
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
    mut payload: Multipart,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    version_id: models::VersionId,
    mod_id: models::ModId,
) -> Result<HttpResponse, CreateError> {
    let cdn_url = dotenv::var("CDN_URL")?;

    let mut initial_file_data: Option<InitialFileData> = None;
    let mut file_builder: Option<VersionFileBuilder> = None;

    let result = sqlx::query!(
        "
        SELECT mod_id, version_number
        FROM versions
        WHERE id = $1
        ",
        version_id as models::VersionId,
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let version = match result {
        Some(v) => v,
        None => {
            return Err(CreateError::InvalidInput(
                "An invalid version id was supplied".to_string(),
            ));
        }
    };
    if version.mod_id as u64 != mod_id.0 as u64 {
        return Err(CreateError::InvalidInput(
            "An invalid version id was supplied".to_string(),
        ));
    }

    let mod_id = ModId(version.mod_id as u64);
    let version_number = version.version_number;

    while let Some(item) = payload.next().await {
        let mut field: Field = item.map_err(CreateError::MultipartError)?;
        let content_disposition = field.content_disposition().ok_or_else(|| {
            CreateError::MissingValueError("Missing content disposition".to_string())
        })?;
        let name = content_disposition
            .get_name()
            .ok_or_else(|| CreateError::MissingValueError("Missing content name".to_string()))?;

        if name == "data" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.map_err(CreateError::MultipartError)?);
            }
            let file_data: InitialFileData = serde_json::from_slice(&data)?;
            // TODO: currently no data here, but still required

            initial_file_data = Some(file_data);
        }

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

        if &*file_extension == "jar" {
            let _file_data = initial_file_data.as_ref().ok_or_else(|| {
                CreateError::InvalidInput(String::from("`data` field must come before file fields"))
            })?;

            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.map_err(CreateError::MultipartError)?);
            }

            let upload_data = file_host
                .upload_file(
                    "application/java-archive",
                    &format!("{}/{}/{}", mod_id, version_number, file_name),
                    data.to_vec(),
                )
                .await?;

            uploaded_files.push(UploadedFile {
                file_id: upload_data.file_id.clone(),
                file_name: upload_data.file_name.clone(),
            });

            // TODO: Malware scan + file validation
            file_builder = Some(models::version_item::VersionFileBuilder {
                filename: file_name.to_string(),
                url: format!("{}/{}", cdn_url, upload_data.file_name),
                hashes: vec![models::version_item::HashBuilder {
                    algorithm: "sha1".to_string(),
                    // This is an invalid cast - the database expects the hash's
                    // bytes, but this is the string version.
                    hash: upload_data.content_sha1.into_bytes(),
                }],
            });
            break;
        }
    }

    if let Some(file_builder) = file_builder {
        file_builder.insert(version_id, &mut *transaction).await?;
    } else {
        return Err(CreateError::InvalidInput(
            "A file must be specified".to_string(),
        ));
    }

    Ok(HttpResponse::Ok().into())
}
