use crate::auth::{get_user_from_headers, AuthenticationError};
use crate::database::models;
use crate::database::models::StatusId;
use crate::file_hosting::{FileHost, FileHostingError};
use crate::models::error::ApiError;
use crate::models::mods::{ModId, ModStatus, VersionId};
use crate::models::teams::TeamMember;
use crate::models::users::UserId;
use crate::routes::version_creation::InitialVersionData;
use crate::search::indexing::queue::CreationQueue;
use actix_multipart::{Field, Multipart};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{post, HttpRequest, HttpResponse};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::borrow::Cow;
use std::sync::Arc;
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
    #[error("Invalid game version: {0}")]
    InvalidGameVersion(String),
    #[error("Invalid loader: {0}")]
    InvalidLoader(String),
    #[error("Invalid category: {0}")]
    InvalidCategory(String),
    #[error("Invalid file type for version file: {0}")]
    InvalidFileType(String),
    #[error("Authentication Error: {0}")]
    Unauthorized(#[from] AuthenticationError),
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
            CreateError::InvalidGameVersion(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidLoader(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidCategory(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidFileType(..) => StatusCode::BAD_REQUEST,
            CreateError::Unauthorized(..) => StatusCode::UNAUTHORIZED,
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
                CreateError::InvalidGameVersion(..) => "invalid_input",
                CreateError::InvalidLoader(..) => "invalid_input",
                CreateError::InvalidCategory(..) => "invalid_input",
                CreateError::InvalidFileType(..) => "invalid_input",
                CreateError::Unauthorized(..) => "unauthorized",
            },
            description: &self.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ModCreateData {
    /// The title or name of the mod.
    pub mod_name: String,
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

pub struct UploadedFile {
    pub file_id: String,
    pub file_name: String,
}

pub async fn undo_uploads(
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

#[post("mod")]
pub async fn mod_create(
    req: HttpRequest,
    payload: Multipart,
    client: Data<PgPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    indexing_queue: Data<Arc<CreationQueue>>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let result = mod_create_inner(
        req,
        payload,
        &mut transaction,
        &***file_host,
        &mut uploaded_files,
        &***indexing_queue,
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
    req: HttpRequest,
    mut payload: Multipart,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    indexing_queue: &CreationQueue,
) -> Result<HttpResponse, CreateError> {
    let cdn_url = dotenv::var("CDN_URL")?;

    let mod_id = models::generate_mod_id(transaction).await?.into();
    let user = get_user_from_headers(req.headers(), &mut *transaction).await?;

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
            let create_data: ModCreateData = serde_json::from_slice(&data)?;

            check_length("mod_name", 3, 255, &*create_data.mod_name)?;
            check_length("mod_description", 3, 2048, &*create_data.mod_description)?;

            mod_create_data = Some(create_data);
            continue;
        }

        let create_data = mod_create_data.as_ref().ok_or_else(|| {
            CreateError::InvalidInput(String::from("`data` field must come before file fields"))
        })?;

        let (file_name, file_extension) =
            super::version_creation::get_name_ext(&content_disposition)?;

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

        let version_data = create_data
            .initial_versions
            .iter()
            .find(|x| x.file_parts.iter().any(|n| n == name))
            .ok_or_else(|| {
                CreateError::InvalidInput(format!(
                    "File `{}` (field {}) isn't specified in the versions data",
                    file_name, name
                ))
            })?;

        // If a version has already been created for this version, add the
        // file to it instead of creating a new version.
        // Versions must have at least one jar file to be uploaded

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

            let release_channel = models::ChannelId(
                sqlx::query!(
                    "
                    SELECT id
                    FROM release_channels
                    WHERE channel = $1
                    ",
                    version_data.release_channel.to_string()
                )
                .fetch_one(&mut *transaction)
                .await?
                .id,
            );

            let mut game_versions = Vec::with_capacity(version_data.game_versions.len());
            for v in &version_data.game_versions {
                let id = models::categories::GameVersion::get_id(&v.0, &mut *transaction)
                    .await?
                    .ok_or_else(|| CreateError::InvalidGameVersion(v.0.clone()))?;
                game_versions.push(id);
            }

            let mut loaders = Vec::with_capacity(version_data.loaders.len());
            for l in &version_data.loaders {
                let id = models::categories::Loader::get_id(&l.0, &mut *transaction)
                    .await?
                    .ok_or_else(|| CreateError::InvalidLoader(l.0.clone()))?;
                loaders.push(id);
            }

            let version = models::version_item::VersionBuilder {
                version_id: version_id.into(),
                mod_id: mod_id.into(),
                author_id: user.id.into(),
                name: version_data.version_title.clone(),
                version_number: version_data.version_number.clone(),
                changelog_url: Some(format!("{}/{}", cdn_url, body_url)),
                files: Vec::with_capacity(1),
                dependencies: version_data
                    .dependencies
                    .iter()
                    .map(|x| (*x).into())
                    .collect::<Vec<_>>(),
                game_versions,
                loaders,
                release_channel,
            };

            created_versions.push(version);
            created_versions.last_mut().unwrap()
        };

        // Upload the new jar file
        let file_builder = super::version_creation::upload_file(
            &mut field,
            file_host,
            uploaded_files,
            &cdn_url,
            &content_disposition,
            mod_id,
            &version_data.version_number,
        )
        .await?;

        // Add the newly uploaded file to the existing or new version
        created_version.files.push(file_builder);
    }

    let create_data = if let Some(create_data) = mod_create_data {
        create_data
    } else {
        return Err(CreateError::InvalidInput(String::from(
            "Multipart upload missing `data` field",
        )));
    };

    let ids: Vec<UserId> = (&create_data.team_members)
        .iter()
        .map(|m| m.user_id)
        .collect();
    if !ids.contains(&user.id) {
        return Err(CreateError::InvalidInput(String::from(
            "Team members must include yourself!",
        )));
    }

    let mut categories = Vec::with_capacity(create_data.categories.len());
    for category in &create_data.categories {
        let id = models::categories::Category::get_id(&category, &mut *transaction)
            .await?
            .ok_or_else(|| CreateError::InvalidCategory(category.clone()))?;
        categories.push(id);
    }

    let body_url = format!("data/{}/body.md", mod_id);

    let upload_data = file_host
        .upload_file("text/plain", &body_url, create_data.mod_body.into_bytes())
        .await?;

    uploaded_files.push(UploadedFile {
        file_id: upload_data.file_id.clone(),
        file_name: upload_data.file_name.clone(),
    });

    let mut author_username = None;
    let mut author_id = None;

    let team = models::team_item::TeamBuilder {
        members: create_data
            .team_members
            .into_iter()
            .map(|member| {
                if member.role == crate::models::teams::OWNER_ROLE {
                    author_id = Some(member.user_id);
                    author_username = Some(member.name.clone());
                }
                models::team_item::TeamMemberBuilder {
                    user_id: member.user_id.into(),
                    name: member.name,
                    role: member.role,
                }
            })
            .collect(),
    };

    let (author_username, author_id) = if let (Some(u), Some(id)) = (author_username, author_id) {
        (u, id)
    } else {
        return Err(CreateError::InvalidInput(String::from(
            "A mod must have an author",
        )));
    };

    let team_id = team.insert(&mut *transaction).await?;

    let status = ModStatus::Processing;
    let status_id = sqlx::query!(
        "
        SELECT id
        FROM statuses
        WHERE status = $1
        ",
        status.to_string()
    )
    .fetch_one(&mut *transaction)
    .await?
    .id;

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

        categories,
        initial_versions: created_versions,
        status: StatusId(status_id),
    };

    let versions_list = mod_builder
        .initial_versions
        .iter()
        .flat_map(|v| {
            v.game_versions.iter().map(|id| id.0.to_string())
            // TODO: proper version identifiers, once game versions
            // have been implemented
        })
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .collect::<Vec<_>>();

    let now = chrono::Utc::now();
    let timestamp = now.timestamp();

    let index_mod = crate::search::UploadSearchMod {
        mod_id: format!("local-{}", mod_id),
        title: mod_builder.title.clone(),
        description: mod_builder.description.clone(),
        categories: create_data.categories.clone(),
        versions: versions_list,
        page_url: format!("https://modrinth.com/mod/{}", mod_id),
        icon_url: mod_builder.icon_url.clone().unwrap(),
        author: author_username,
        author_url: format!("https://modrinth.com/user/{}", author_id),
        // TODO: latest version info
        latest_version: String::new(),
        downloads: 0,
        date_created: now,
        created_timestamp: timestamp,
        date_modified: now,
        modified_timestamp: timestamp,
        host: Cow::Borrowed("modrinth"),
        empty: Cow::Borrowed("{}{}{}"),
    };

    indexing_queue.add(index_mod);

    let response = crate::models::mods::Mod {
        id: mod_id,
        team: team_id.into(),
        title: mod_builder.title.clone(),
        description: mod_builder.description.clone(),
        body_url: mod_builder.body_url.clone(),
        published: now,
        updated: now,
        status,
        downloads: 0,
        categories: create_data.categories.clone(),
        versions: mod_builder
            .initial_versions
            .iter()
            .map(|v| v.version_id.into())
            .collect::<Vec<_>>(),
        icon_url: mod_builder.icon_url.clone(),
        issues_url: mod_builder.issues_url.clone(),
        source_url: mod_builder.source_url.clone(),
        wiki_url: mod_builder.wiki_url.clone(),
    };

    let _mod_id = mod_builder.insert(&mut *transaction).await?;

    // TODO: respond with the new mod info, or with just the new mod id.
    Ok(HttpResponse::Ok().json(response))
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

fn check_length(
    var_name: &str,
    min_length: usize,
    max_length: usize,
    string: &str,
) -> Result<(), CreateError> {
    if string.len() > max_length || string.len() < min_length {
        Err(CreateError::InvalidInput(format!(
            "The {} must be between {} and {} characters; got {}.",
            var_name, string, min_length, max_length
        )))
    } else {
        Ok(())
    }
}
