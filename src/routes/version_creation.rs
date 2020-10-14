use crate::auth::get_user_from_headers;
use crate::database::models;
use crate::database::models::version_item::{VersionBuilder, VersionFileBuilder};
use crate::file_hosting::FileHost;
use crate::models::mods::{
    GameVersion, ModId, ModLoader, Version, VersionFile, VersionId, VersionType,
};
use crate::routes::mod_creation::{CreateError, UploadedFile};
use actix_multipart::{Field, Multipart};
use actix_web::web::Data;
use actix_web::{post, HttpRequest, HttpResponse};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Clone)]
pub struct InitialVersionData {
    pub mod_id: ModId,
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

/// TODO: Update mod timestamp when new version is created
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

    let user = get_user_from_headers(req.headers(), &mut *transaction).await?;

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
            let mod_id: models::ModId = version_create_data.mod_id.into();

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

            let team_id = sqlx::query!(
                "SELECT team_id FROM mods WHERE id=$1",
                mod_id as models::ModId,
            )
            .fetch_one(&mut *transaction)
            .await?
            .team_id;

            let member_ids_rows =
                sqlx::query!("SELECT user_id FROM team_members WHERE team_id=$1", team_id,)
                    .fetch_all(&mut *transaction)
                    .await?;

            let member_ids: Vec<i64> = member_ids_rows.iter().map(|m| m.user_id).collect();

            if !member_ids.contains(&(user.id.0 as i64)) {
                return Err(CreateError::InvalidInput("Unauthorized".to_string()));
            }

            let version_id: VersionId = models::generate_version_id(transaction).await?.into();
            let body_url = format!(
                "data/{}/changelogs/{}/body.md",
                version_create_data.mod_id, version_id
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

            let release_channel = models::ChannelId(
                sqlx::query!(
                    "
                SELECT id
                FROM release_channels
                WHERE channel = $1
                ",
                    version_create_data.release_channel.to_string()
                )
                .fetch_one(&mut *transaction)
                .await?
                .id,
            );

            let mut game_versions = Vec::with_capacity(version_create_data.game_versions.len());
            for v in &version_create_data.game_versions {
                let id = models::categories::GameVersion::get_id(&v.0, &mut *transaction)
                    .await?
                    .ok_or_else(|| CreateError::InvalidGameVersion(v.0.clone()))?;
                game_versions.push(id);
            }

            let mut loaders = Vec::with_capacity(version_create_data.loaders.len());
            for l in &version_create_data.loaders {
                let id = models::categories::Loader::get_id(&l.0, &mut *transaction)
                    .await?
                    .ok_or_else(|| CreateError::InvalidLoader(l.0.clone()))?;
                loaders.push(id);
            }

            version_builder = Some(VersionBuilder {
                version_id: version_id.into(),
                mod_id: version_create_data.mod_id.into(),
                author_id: user.id.into(),
                name: version_create_data.version_title.clone(),
                version_number: version_create_data.version_number.clone(),
                changelog_url: Some(format!("{}/{}", cdn_url, body_url)),
                files: Vec::with_capacity(1),
                dependencies: version_create_data
                    .dependencies
                    .iter()
                    .map(|x| (*x).into())
                    .collect::<Vec<_>>(),
                game_versions,
                loaders,
                release_channel,
            });

            continue;
        }

        let version = version_builder.as_mut().ok_or_else(|| {
            CreateError::InvalidInput(String::from("`data` field must come before file fields"))
        })?;

        let file_builder = upload_file(
            &mut field,
            file_host,
            uploaded_files,
            &cdn_url,
            &content_disposition,
            version.mod_id.into(),
            &version.version_number,
        )
        .await?;

        // Add the newly uploaded file to the existing or new version
        version.files.push(file_builder);
    }

    let version_data_safe = initial_version_data
        .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;
    let version_builder_safe = version_builder
        .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;

    let response = Version {
        id: version_builder_safe.version_id.into(),
        mod_id: version_builder_safe.mod_id.into(),
        author_id: user.id,
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

    let version_id = models::VersionId::from(url_data.0);

    let result = upload_file_to_version_inner(
        req,
        payload,
        &mut transaction,
        &***file_host,
        &mut uploaded_files,
        version_id,
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
    req: HttpRequest,
    mut payload: Multipart,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    version_id: models::VersionId,
) -> Result<HttpResponse, CreateError> {
    let cdn_url = dotenv::var("CDN_URL")?;

    let mut initial_file_data: Option<InitialFileData> = None;
    let mut file_builders: Vec<VersionFileBuilder> = Vec::new();

    let user = get_user_from_headers(req.headers(), &mut *transaction).await?;

    let result = sqlx::query!(
        "
        SELECT mod_id, version_number, author_id
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

    if version.author_id as u64 != user.id.0 {
        return Err(CreateError::InvalidInput("Unauthorized".to_string()));
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
            continue;
        }

        let _file_data = initial_file_data.as_ref().ok_or_else(|| {
            CreateError::InvalidInput(String::from("`data` field must come before file fields"))
        })?;

        let file_builder = upload_file(
            &mut field,
            file_host,
            uploaded_files,
            &cdn_url,
            &content_disposition,
            mod_id,
            &version_number,
        )
        .await?;

        // TODO: Malware scan + file validation
        file_builders.push(file_builder);
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
// files for a version, and for uploading the initial version files for a mod
pub async fn upload_file(
    field: &mut Field,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    cdn_url: &str,
    content_disposition: &actix_web::http::header::ContentDisposition,
    mod_id: crate::models::ids::ModId,
    version_number: &str,
) -> Result<models::version_item::VersionFileBuilder, CreateError> {
    let (file_name, file_extension) = get_name_ext(content_disposition)?;

    let content_type = mod_file_type(file_extension)
        .ok_or_else(|| CreateError::InvalidFileType(file_extension.to_string()))?;

    let mut data = Vec::new();
    while let Some(chunk) = field.next().await {
        data.extend_from_slice(&chunk.map_err(CreateError::MultipartError)?);
    }

    let upload_data = file_host
        .upload_file(
            content_type,
            &format!("{}/{}/{}", mod_id, version_number, file_name),
            data.to_vec(),
        )
        .await?;

    uploaded_files.push(UploadedFile {
        file_id: upload_data.file_id.clone(),
        file_name: upload_data.file_name.clone(),
    });

    // TODO: Malware scan + file validation
    Ok(models::version_item::VersionFileBuilder {
        filename: file_name.to_string(),
        url: format!("{}/{}", cdn_url, upload_data.file_name),
        hashes: vec![models::version_item::HashBuilder {
            algorithm: "sha1".to_string(),
            // This is an invalid cast - the database expects the hash's
            // bytes, but this is the string version.
            hash: upload_data.content_sha1.into_bytes(),
        }],
    })
}

// Currently we only support jar mods; this may change in the future (datapacks?)
fn mod_file_type(ext: &str) -> Option<&str> {
    match ext {
        "jar" => Some("application/java-archive"),
        _ => None,
    }
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
