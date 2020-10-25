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
    pub mod_id: Option<ModId>,
    pub file_parts: Vec<String>,
    pub version_number: String,
    pub version_title: String,
    pub version_body: Option<String>,
    pub dependencies: Vec<VersionId>,
    pub game_versions: Vec<GameVersion>,
    pub release_channel: VersionType,
    pub loaders: Vec<ModLoader>,
}

#[derive(Serialize, Deserialize, Clone)]
struct InitialFileData {
    // TODO: hashes?
}

pub fn check_version(version: &InitialVersionData) -> Result<(), CreateError> {
    /*
    # InitialVersionData
    file_parts: Vec<String>, 1..=256
    version_number: 1..=64,
    version_title: 3..=256,
    version_body: max of 64KiB,
    game_versions: Vec<GameVersion>, 1..=256
    release_channel: VersionType,
    loaders: Vec<ModLoader>, 1..=256
    */
    use super::mod_creation::check_length;

    version
        .file_parts
        .iter()
        .map(|f| check_length(1..=256, "file part name", f))
        .collect::<Result<_, _>>()?;

    check_length(1..=64, "version number", &version.version_number)?;
    check_length(3..=256, "version title", &version.version_title)?;
    if let Some(body) = &version.version_body {
        check_length(..65536, "version body", body)?;
    }

    version
        .game_versions
        .iter()
        .map(|v| check_length(1..=256, "game version", &v.0))
        .collect::<Result<_, _>>()?;
    version
        .loaders
        .iter()
        .map(|l| check_length(1..=256, "loader name", &l.0))
        .collect::<Result<_, _>>()?;

    Ok(())
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
            if version_create_data.mod_id.is_none() {
                return Err(CreateError::MissingValueError("Missing mod id".to_string()));
            }

            check_version(version_create_data)?;

            let mod_id: models::ModId = version_create_data.mod_id.unwrap().into();

            // Ensure that the mod this version is being added to exists
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

            // Check whether there is already a version of this mod with the
            // same version number
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

            // Check that the user creating this version is a team member
            // of the mod the version is being added to.
            let member_ids = sqlx::query!(
                "
                SELECT user_id FROM team_members tm
                INNER JOIN mods ON mods.team_id = tm.team_id
                WHERE mods.id = $1
                ",
                mod_id as models::ModId,
            )
            .fetch_all(&mut *transaction)
            .await?;

            let member_ids: Vec<models::UserId> = member_ids
                .iter()
                .map(|m| models::UserId(m.user_id))
                .collect();

            if !member_ids.contains(&user.id.into()) {
                // TODO: Some team members may not have the permissions
                // to upload mods; We need a more in depth permissions
                // system.
                return Err(CreateError::InvalidInput("Unauthorized".to_string()));
            }

            let version_id: VersionId = models::generate_version_id(transaction).await?.into();

            let body_path;

            if let Some(body) = &version_create_data.version_body {
                let path = format!(
                    "data/{}/versions/{}/changelog.md",
                    version_create_data.mod_id.unwrap(),
                    version_create_data.version_number
                );

                let uploaded_text = file_host
                    .upload_file("text/plain", &path, body.clone().into_bytes())
                    .await?;

                uploaded_files.push(UploadedFile {
                    file_id: uploaded_text.file_id,
                    file_name: uploaded_text.file_name.clone(),
                });
                body_path = Some(path);
            } else {
                body_path = None;
            }

            let release_channel = models::ChannelId::get_id(
                version_create_data.release_channel.as_str(),
                &mut *transaction,
            )
            .await?
            .expect("Release channel not found in database");

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
                mod_id: version_create_data.mod_id.unwrap().into(),
                author_id: user.id.into(),
                name: version_create_data.version_title.clone(),
                version_number: version_create_data.version_number.clone(),
                changelog_url: body_path.map(|path| format!("{}/{}", cdn_url, path)),
                files: Vec::new(),
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

    let version_data = initial_version_data
        .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;
    let builder = version_builder
        .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;

    let response = Version {
        id: builder.version_id.into(),
        mod_id: builder.mod_id.into(),
        author_id: user.id,
        name: builder.name.clone(),
        version_number: builder.version_number.clone(),
        changelog_url: builder.changelog_url.clone(),
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

    // Mod file size limit of 25MiB
    const FILE_SIZE_CAP: usize = 25 * (2 << 30);

    // TODO: override file size cap for authorized users or mods
    if data.len() >= FILE_SIZE_CAP {
        return Err(CreateError::InvalidInput(
            String::from("Mod file exceeds the maximum of 25MiB. Contact a moderator or admin to request permission to upload larger files.")
        ));
    }

    let upload_data = file_host
        .upload_file(
            content_type,
            &format!("data/{}/versions/{}/{}", mod_id, version_number, file_name),
            data.to_vec(),
        )
        .await?;

    uploaded_files.push(UploadedFile {
        file_id: upload_data.file_id,
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
