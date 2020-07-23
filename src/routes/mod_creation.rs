use crate::database::models;
use crate::file_hosting::{FileHost, FileHostingError};
use crate::models::error::ApiError;
use crate::models::mods::{GameVersion, ModId, VersionId, VersionType};
use crate::models::teams::TeamMember;
use actix_multipart::{Field, Multipart};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{post, HttpResponse};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
    #[error("An unknown database error occured")]
    SqlxDatabaseError(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] models::DatabaseError),
    #[error("Error while parsing multipart payload")]
    MultipartError(actix_multipart::MultipartError),
    #[error("Error while parsing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while uploading file")]
    FileHostingError(#[from] FileHostingError),
    #[error("{}", .0)]
    MissingValueError(String),
    #[error("Invalid format for mod icon: {0}")]
    InvalidIconFormat(String),
    #[error("Error with multipart data: {0}")]
    InvalidInput(String),
}

impl actix_web::ResponseError for CreateError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateError::EnvError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::SqlxDatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::DatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::FileHostingError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::SerDeError(..) => StatusCode::BAD_REQUEST,
            CreateError::MultipartError(..) => StatusCode::BAD_REQUEST,
            CreateError::MissingValueError(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidIconFormat(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidInput(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                CreateError::EnvError(..) => "environment_error",
                CreateError::SqlxDatabaseError(..) => "database_error",
                CreateError::DatabaseError(..) => "database_error",
                CreateError::FileHostingError(..) => "file_hosting_error",
                CreateError::SerDeError(..) => "invalid_input",
                CreateError::MultipartError(..) => "invalid_input",
                CreateError::MissingValueError(..) => "invalid_input",
                CreateError::InvalidIconFormat(..) => "invalid_input",
                CreateError::InvalidInput(..) => "invalid_input",
            },
            description: &self.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct InitialVersionData {
    pub file_parts: Vec<String>,
    pub version_number: String,
    pub version_title: String,
    pub version_body: String,
    pub dependencies: Vec<VersionId>,
    pub game_versions: Vec<GameVersion>,
    pub release_channel: VersionType,
    pub loaders: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ModCreateData {
    /// The title or name of the mod.
    pub mod_name: String,
    /// The namespace of the mod
    pub mod_namespace: String,
    /// A short description of the mod.
    pub mod_description: String,
    /// A long description of the mod, in markdown.
    pub mod_body: String,
    /// A list of initial versions to upload with the created mod
    pub initial_versions: Vec<InitialVersionData>,
    /// The team of people that has ownership of this mod.
    pub team_members: Vec<TeamMember>,
    /// A list of the categories that the mod is in.
    pub categories: Vec<String>,
    /// An optional link to where to submit bugs or issues with the mod.
    pub issues_url: Option<String>,
    /// An optional link to the source code for the mod.
    pub source_url: Option<String>,
    /// An optional link to the mod's wiki page or other relevant information.
    pub wiki_url: Option<String>,
}

struct UploadedFile {
    file_id: String,
    file_name: String,
}

async fn undo_uploads(
    file_host: &dyn FileHost,
    uploaded_files: &[UploadedFile],
) -> Result<(), CreateError> {
    for file in uploaded_files {
        file_host
            .delete_file_version(&file.file_id, &file.file_name)
            .await?;
    }
    Ok(())
}

#[post("api/v1/mod")]
pub async fn mod_create(
    payload: Multipart,
    client: Data<PgPool>,
    file_host: Data<std::sync::Arc<dyn FileHost + Send + Sync>>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let result = mod_create_inner(
        payload,
        &mut transaction,
        &***file_host,
        &mut uploaded_files,
    )
    .await;

    if result.is_err() {
        let undo_result = undo_uploads(&***file_host, &uploaded_files).await;
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

async fn mod_create_inner(
    mut payload: Multipart,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
) -> Result<HttpResponse, CreateError> {
    let cdn_url = dotenv::var("CDN_URL")?;

    let mod_id = models::generate_mod_id(transaction).await?.into();

    let mut created_versions: Vec<models::version_item::VersionBuilder> = vec![];

    let mut mod_create_data: Option<ModCreateData> = None;
    let mut icon_url = "".to_string();

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
            mod_create_data = Some(serde_json::from_slice(&data)?);
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

        if name == "icon" {
            icon_url = process_icon_upload(
                uploaded_files,
                mod_id,
                file_name,
                file_extension,
                file_host,
                field,
                &cdn_url,
            )
            .await?;
            continue;
        }

        if &*file_extension == "jar" {
            let create_data = mod_create_data.as_ref().ok_or_else(|| {
                CreateError::InvalidInput(String::from("`data` field must come before file fields"))
            })?;

            let version_data = create_data
                .initial_versions
                .iter()
                .find(|x| x.file_parts.iter().any(|n| n == name))
                .ok_or_else(|| {
                    CreateError::InvalidInput(format!(
                        "Jar file `{}` (field {}) isn't specified in the versions data",
                        file_name, name
                    ))
                })?;

            // If a version has already been created for this version, add the
            // file to it instead of creating a new version.

            let created_version = if let Some(created_version) = created_versions
                .iter_mut()
                .find(|x| x.version_number == version_data.version_number)
            {
                created_version
            } else {
                let version_id: VersionId = models::generate_version_id(transaction).await?.into();

                let body_url = format!("data/{}/changelogs/{}/body.md", mod_id, version_id);

                let uploaded_text = file_host
                    .upload_file(
                        "text/plain",
                        &body_url,
                        version_data.version_body.clone().into_bytes(),
                    )
                    .await?;

                uploaded_files.push(UploadedFile {
                    file_id: uploaded_text.file_id.clone(),
                    file_name: uploaded_text.file_name.clone(),
                });

                // TODO: do a real lookup for the channels
                let release_channel = match version_data.release_channel {
                    VersionType::Release => models::ChannelId(0),
                    VersionType::Beta => models::ChannelId(2),
                    VersionType::Alpha => models::ChannelId(4),
                };

                let version = models::version_item::VersionBuilder {
                    version_id: version_id.into(),
                    mod_id: mod_id.into(),
                    name: version_data.version_title.clone(),
                    version_number: version_data.version_number.clone(),
                    changelog_url: Some(format!("{}/{}", cdn_url, body_url)),
                    files: Vec::with_capacity(1),
                    dependencies: version_data
                        .dependencies
                        .iter()
                        .map(|x| (*x).into())
                        .collect::<Vec<_>>(),
                    // TODO: add game_versions and loaders info
                    game_versions: vec![],
                    loaders: vec![],
                    release_channel,
                };

                created_versions.push(version);
                created_versions.last_mut().unwrap()
            };

            // Upload the new jar file

            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.map_err(CreateError::MultipartError)?);
            }

            let upload_data = file_host
                .upload_file(
                    "application/java-archive",
                    &format!(
                        "{}/{}/{}",
                        create_data.mod_namespace.replace(".", "/"),
                        version_data.version_number,
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
            created_version
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

    let create_data = if let Some(create_data) = mod_create_data {
        create_data
    } else {
        return Err(CreateError::InvalidInput(String::from(
            "Multipart upload missing `data` field",
        )));
    };

    let body_url = format!("data/{}/body.md", mod_id);

    let upload_data = file_host
        .upload_file("text/plain", &body_url, create_data.mod_body.into_bytes())
        .await?;

    uploaded_files.push(UploadedFile {
        file_id: upload_data.file_id.clone(),
        file_name: upload_data.file_name.clone(),
    });

    let team = models::team_item::TeamBuilder {
        members: create_data
            .team_members
            .into_iter()
            .map(|member| models::team_item::TeamMemberBuilder {
                user_id: member.user_id.into(),
                name: member.name,
                role: member.role,
            })
            .collect(),
    };

    let team_id = team.insert(&mut *transaction).await?;

    // Insert the new mod into the database

    let mod_builder = models::mod_item::ModBuilder {
        mod_id: mod_id.into(),
        team_id,
        title: create_data.mod_name,
        description: create_data.mod_description,
        body_url: format!("{}/{}", cdn_url, body_url),
        icon_url: Some(icon_url),
        issues_url: create_data.issues_url,
        source_url: create_data.source_url,
        wiki_url: create_data.wiki_url,

        // TODO: convert `create_data.categories` from Vec<String> to Vec<CategoryId>
        categories: Vec::new(),
        initial_versions: created_versions,
    };

    let _mod_id = mod_builder.insert(&mut *transaction).await?;

    // TODO: respond with the new mod info, or with just the new mod id.
    Ok(HttpResponse::Ok().into())
}

async fn process_icon_upload(
    uploaded_files: &mut Vec<UploadedFile>,
    mod_id: ModId,
    file_name: &str,
    file_extension: &str,
    file_host: &dyn FileHost,
    mut field: actix_multipart::Field,
    cdn_url: &str,
) -> Result<String, CreateError> {
    if let Some(content_type) = get_image_content_type(file_extension) {
        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            data.extend_from_slice(&chunk.map_err(CreateError::MultipartError)?);
        }

        let upload_data = file_host
            .upload_file(
                content_type,
                &format!("mods/icons/{}/{}", mod_id, file_name),
                data,
            )
            .await?;

        uploaded_files.push(UploadedFile {
            file_id: upload_data.file_id.clone(),
            file_name: upload_data.file_name.clone(),
        });

        Ok(format!("{}/{}", cdn_url, upload_data.file_name))
    } else {
        Err(CreateError::InvalidIconFormat(file_extension.to_string()))
    }
}

fn get_image_content_type(extension: &str) -> Option<&'static str> {
    let content_type = match &*extension {
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        "jpeg" | "jpg" | "jpe" => "image/jpeg",
        "png" => "image/png",
        "svg" | "svgz" => "image/svg+xml",
        "webp" => "image/webp",
        "rgb" => "image/x-rgb",
        _ => "",
    };

    if content_type != "" {
        Some(content_type)
    } else {
        None
    }
}
