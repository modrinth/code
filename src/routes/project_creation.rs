use crate::database::models;
use crate::file_hosting::{FileHost, FileHostingError};
use crate::models::error::ApiError;
use crate::models::projects::{
    DonationLink, License, ProjectId, ProjectStatus, SideType, VersionId,
};
use crate::models::users::UserId;
use crate::routes::version_creation::InitialVersionData;
use crate::search::indexing::{queue::CreationQueue, IndexingError};
use crate::util::auth::{get_user_from_headers, AuthenticationError};
use crate::util::validate::validation_errors_to_string;
use actix_multipart::{Field, Multipart};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{post, HttpRequest, HttpResponse};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::sync::Arc;
use thiserror::Error;
use validator::Validate;

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
    #[error("An unknown database error occurred")]
    SqlxDatabaseError(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] models::DatabaseError),
    #[error("Indexing Error: {0}")]
    IndexingError(#[from] IndexingError),
    #[error("Error while parsing multipart payload")]
    MultipartError(actix_multipart::MultipartError),
    #[error("Error while parsing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while validating input: {0}")]
    ValidationError(String),
    #[error("Error while uploading file")]
    FileHostingError(#[from] FileHostingError),
    #[error("Error while validating uploaded file: {0}")]
    FileValidationError(#[from] crate::validate::ValidationError),
    #[error("{}", .0)]
    MissingValueError(String),
    #[error("Invalid format for project icon: {0}")]
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
    #[error("Slug collides with other project's id!")]
    SlugCollision,
    #[error("Authentication Error: {0}")]
    Unauthorized(#[from] AuthenticationError),
    #[error("Authentication Error: {0}")]
    CustomAuthenticationError(String),
}

impl actix_web::ResponseError for CreateError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateError::EnvError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::SqlxDatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::DatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::IndexingError(..) => StatusCode::INTERNAL_SERVER_ERROR,
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
            CreateError::CustomAuthenticationError(..) => StatusCode::UNAUTHORIZED,
            CreateError::SlugCollision => StatusCode::BAD_REQUEST,
            CreateError::ValidationError(..) => StatusCode::BAD_REQUEST,
            CreateError::FileValidationError(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                CreateError::EnvError(..) => "environment_error",
                CreateError::SqlxDatabaseError(..) => "database_error",
                CreateError::DatabaseError(..) => "database_error",
                CreateError::IndexingError(..) => "indexing_error",
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
                CreateError::CustomAuthenticationError(..) => "unauthorized",
                CreateError::SlugCollision => "invalid_input",
                CreateError::ValidationError(..) => "invalid_input",
                CreateError::FileValidationError(..) => "invalid_input",
            },
            description: &self.to_string(),
        })
    }
}

fn default_project_type() -> String {
    "mod".to_string()
}

#[derive(Serialize, Deserialize, Validate, Clone)]
struct ProjectCreateData {
    #[validate(length(min = 3, max = 256))]
    #[serde(alias = "mod_name")]
    /// The title or name of the project.
    pub title: String,
    #[validate(length(min = 1, max = 64))]
    #[serde(default = "default_project_type")]
    /// The project type of this mod
    pub project_type: String,
    #[validate(
        length(min = 3, max = 64),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    #[serde(alias = "mod_slug")]
    /// The slug of a project, used for vanity URLs
    pub slug: String,
    #[validate(length(min = 3, max = 2048))]
    #[serde(alias = "mod_description")]
    /// A short description of the project.
    pub description: String,
    #[validate(length(max = 65536))]
    #[serde(alias = "mod_body")]
    /// A long description of the project, in markdown.
    pub body: String,

    /// The support range for the client project
    pub client_side: SideType,
    /// The support range for the server project
    pub server_side: SideType,

    #[validate(length(max = 64))]
    #[validate]
    /// A list of initial versions to upload with the created project
    pub initial_versions: Vec<InitialVersionData>,
    #[validate(length(max = 3))]
    /// A list of the categories that the project is in.
    pub categories: Vec<String>,

    #[validate(url, length(max = 2048))]
    /// An optional link to where to submit bugs or issues with the project.
    pub issues_url: Option<String>,
    #[validate(url, length(max = 2048))]
    /// An optional link to the source code for the project.
    pub source_url: Option<String>,
    #[validate(url, length(max = 2048))]
    /// An optional link to the project's wiki page or other relevant information.
    pub wiki_url: Option<String>,
    #[validate(url, length(max = 2048))]
    /// An optional link to the project's license page
    pub license_url: Option<String>,
    #[validate(url, length(max = 2048))]
    /// An optional link to the project's discord.
    pub discord_url: Option<String>,
    /// An optional list of all donation links the project has\
    #[validate]
    pub donation_urls: Option<Vec<DonationLink>>,

    /// An optional boolean. If true, the project will be created as a draft.
    pub is_draft: Option<bool>,

    /// The license id that the project follows
    pub license_id: String,
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

#[post("project")]
pub async fn project_create(
    req: HttpRequest,
    payload: Multipart,
    client: Data<PgPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    indexing_queue: Data<Arc<CreationQueue>>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let result = project_create_inner(
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

/*

Project Creation Steps:
Get logged in user
    Must match the author in the version creation

1. Data
    - Gets "data" field from multipart form; must be first
    - Verification: string lengths
    - Create versions
        - Some shared logic with version creation
        - Create list of VersionBuilders
    - Create ProjectBuilder

2. Upload
    - Icon: check file format & size
        - Upload to backblaze & record URL
    - Project files
        - Check for matching version
        - File size limits?
        - Check file type
            - Eventually, malware scan
        - Upload to backblaze & create VersionFileBuilder
    -

3. Creation
    - Database stuff
    - Add project data to indexing queue
*/

pub async fn project_create_inner(
    req: HttpRequest,
    mut payload: Multipart,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    indexing_queue: &CreationQueue,
) -> Result<HttpResponse, CreateError> {
    // The base URL for files uploaded to backblaze
    let cdn_url = dotenv::var("CDN_URL")?;

    // The currently logged in user
    let current_user = get_user_from_headers(req.headers(), &mut *transaction).await?;

    let project_id: ProjectId = models::generate_project_id(transaction).await?.into();

    let project_create_data;
    let mut versions;
    let mut versions_map = std::collections::HashMap::new();

    let all_game_versions = models::categories::GameVersion::list(&mut *transaction).await?;
    let all_loaders = models::categories::Loader::list(&mut *transaction).await?;

    {
        // The first multipart field must be named "data" and contain a
        // JSON `ProjectCreateData` object.

        let mut field = payload
            .next()
            .await
            .map(|m| m.map_err(CreateError::MultipartError))
            .unwrap_or_else(|| {
                Err(CreateError::MissingValueError(String::from(
                    "No `data` field in multipart upload",
                )))
            })?;

        let content_disposition = field.content_disposition().ok_or_else(|| {
            CreateError::MissingValueError(String::from("Missing content disposition"))
        })?;
        let name = content_disposition
            .get_name()
            .ok_or_else(|| CreateError::MissingValueError(String::from("Missing content name")))?;

        if name != "data" {
            return Err(CreateError::InvalidInput(String::from(
                "`data` field must come before file fields",
            )));
        }

        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            data.extend_from_slice(&chunk.map_err(CreateError::MultipartError)?);
        }
        let create_data: ProjectCreateData = serde_json::from_slice(&data)?;

        create_data
            .validate()
            .map_err(|err| CreateError::InvalidInput(validation_errors_to_string(err, None)))?;

        let slug_project_id_option: Option<ProjectId> =
            serde_json::from_str(&*format!("\"{}\"", create_data.slug)).ok();

        if let Some(slug_project_id) = slug_project_id_option {
            let slug_project_id: models::ids::ProjectId = slug_project_id.into();
            let results = sqlx::query!(
                "
                SELECT EXISTS(SELECT 1 FROM mods WHERE id=$1)
                ",
                slug_project_id as models::ids::ProjectId
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(|e| CreateError::DatabaseError(e.into()))?;

            if results.exists.unwrap_or(true) {
                return Err(CreateError::SlugCollision);
            }
        }

        // Create VersionBuilders for the versions specified in `initial_versions`
        versions = Vec::with_capacity(create_data.initial_versions.len());
        for (i, data) in create_data.initial_versions.iter().enumerate() {
            // Create a map of multipart field names to version indices
            for name in &data.file_parts {
                if versions_map.insert(name.to_owned(), i).is_some() {
                    // If the name is already used
                    return Err(CreateError::InvalidInput(String::from(
                        "Duplicate multipart field name",
                    )));
                }
            }
            versions.push(
                create_initial_version(
                    data,
                    project_id,
                    current_user.id,
                    &all_game_versions,
                    &all_loaders,
                    &create_data.project_type,
                    transaction,
                )
                .await?,
            );
        }

        project_create_data = create_data;
    }

    let project_type_id =
        models::ProjectTypeId::get_id(project_create_data.project_type.clone(), &mut *transaction)
            .await?
            .ok_or_else(|| {
                CreateError::InvalidInput(format!(
                    "Project Type {} does not exist.",
                    project_create_data.project_type.clone()
                ))
            })?;

    let mut icon_url = None;

    while let Some(item) = payload.next().await {
        let mut field: Field = item.map_err(CreateError::MultipartError)?;
        let content_disposition = field.content_disposition().ok_or_else(|| {
            CreateError::MissingValueError("Missing content disposition".to_string())
        })?;

        let name = content_disposition
            .get_name()
            .ok_or_else(|| CreateError::MissingValueError("Missing content name".to_string()))?;

        let (file_name, file_extension) =
            super::version_creation::get_name_ext(&content_disposition)?;

        if name == "icon" {
            if icon_url.is_some() {
                return Err(CreateError::InvalidInput(String::from(
                    "Projects can only have one icon",
                )));
            }
            // Upload the icon to the cdn
            icon_url = Some(
                process_icon_upload(
                    uploaded_files,
                    project_id,
                    file_extension,
                    file_host,
                    field,
                    &cdn_url,
                )
                .await?,
            );
            continue;
        }

        let index = if let Some(i) = versions_map.get(name) {
            *i
        } else {
            return Err(CreateError::InvalidInput(format!(
                "File `{}` (field {}) isn't specified in the versions data",
                file_name, name
            )));
        };

        // `index` is always valid for these lists
        let created_version = versions.get_mut(index).unwrap();
        let version_data = project_create_data.initial_versions.get(index).unwrap();

        // Upload the new jar file
        super::version_creation::upload_file(
            &mut field,
            file_host,
            uploaded_files,
            &mut created_version.files,
            &cdn_url,
            &content_disposition,
            project_id,
            &version_data.version_number,
            &*project_create_data.project_type,
            version_data.loaders.clone(),
            version_data.game_versions.clone(),
            &all_game_versions,
            false,
        )
        .await?;
    }

    {
        // Check to make sure that all specified files were uploaded
        for (version_data, builder) in project_create_data
            .initial_versions
            .iter()
            .zip(versions.iter())
        {
            if version_data.file_parts.len() != builder.files.len() {
                return Err(CreateError::InvalidInput(String::from(
                    "Some files were specified in initial_versions but not uploaded",
                )));
            }
        }

        // Convert the list of category names to actual categories
        let mut categories = Vec::with_capacity(project_create_data.categories.len());
        for category in &project_create_data.categories {
            let id = models::categories::Category::get_id_project(
                &category,
                project_type_id,
                &mut *transaction,
            )
            .await?
            .ok_or_else(|| CreateError::InvalidCategory(category.clone()))?;
            categories.push(id);
        }

        let team = models::team_item::TeamBuilder {
            members: vec![models::team_item::TeamMemberBuilder {
                user_id: current_user.id.into(),
                role: crate::models::teams::OWNER_ROLE.to_owned(),
                permissions: crate::models::teams::Permissions::ALL,
                accepted: true,
            }],
        };

        let team_id = team.insert(&mut *transaction).await?;

        let status;
        if project_create_data.is_draft.unwrap_or(false) {
            status = ProjectStatus::Draft;
        } else {
            status = ProjectStatus::Processing;

            if project_create_data.initial_versions.is_empty() {
                return Err(CreateError::InvalidInput(String::from(
                    "Project submitted for review with no initial versions",
                )));
            }
        }

        let status_id = models::StatusId::get_id(&status, &mut *transaction)
            .await?
            .ok_or_else(|| {
                CreateError::InvalidInput(format!("Status {} does not exist.", status.clone()))
            })?;
        let client_side_id =
            models::SideTypeId::get_id(&project_create_data.client_side, &mut *transaction)
                .await?
                .ok_or_else(|| {
                    CreateError::InvalidInput(
                        "Client side type specified does not exist.".to_string(),
                    )
                })?;

        let server_side_id =
            models::SideTypeId::get_id(&project_create_data.server_side, &mut *transaction)
                .await?
                .ok_or_else(|| {
                    CreateError::InvalidInput(
                        "Server side type specified does not exist.".to_string(),
                    )
                })?;

        let license_id =
            models::categories::License::get_id(&project_create_data.license_id, &mut *transaction)
                .await?
                .ok_or_else(|| {
                    CreateError::InvalidInput("License specified does not exist.".to_string())
                })?;
        let mut donation_urls = vec![];

        if let Some(urls) = &project_create_data.donation_urls {
            for url in urls {
                let platform_id = models::DonationPlatformId::get_id(&url.id, &mut *transaction)
                    .await?
                    .ok_or_else(|| {
                        CreateError::InvalidInput(format!(
                            "Donation platform {} does not exist.",
                            url.id.clone()
                        ))
                    })?;

                donation_urls.push(models::project_item::DonationUrl {
                    project_id: project_id.into(),
                    platform_id,
                    platform_short: "".to_string(),
                    platform_name: "".to_string(),
                    url: url.url.clone(),
                })
            }
        }

        let project_builder = models::project_item::ProjectBuilder {
            project_id: project_id.into(),
            project_type_id,
            team_id,
            title: project_create_data.title,
            description: project_create_data.description,
            body: project_create_data.body,
            icon_url,
            issues_url: project_create_data.issues_url,
            source_url: project_create_data.source_url,
            wiki_url: project_create_data.wiki_url,

            license_url: project_create_data.license_url,
            discord_url: project_create_data.discord_url,
            categories,
            initial_versions: versions,
            status: status_id,
            client_side: client_side_id,
            server_side: server_side_id,
            license: license_id,
            slug: Some(project_create_data.slug),
            donation_urls,
        };

        let now = chrono::Utc::now();

        let response = crate::models::projects::Project {
            id: project_id,
            slug: project_builder.slug.clone(),
            project_type: project_create_data.project_type.clone(),
            team: team_id.into(),
            title: project_builder.title.clone(),
            description: project_builder.description.clone(),
            body: project_builder.body.clone(),
            body_url: None,
            published: now,
            updated: now,
            status: status.clone(),
            rejection_data: None,
            license: License {
                id: project_create_data.license_id.clone(),
                name: "".to_string(),
                url: project_builder.license_url.clone(),
            },
            client_side: project_create_data.client_side,
            server_side: project_create_data.server_side,
            downloads: 0,
            followers: 0,
            categories: project_create_data.categories,
            versions: project_builder
                .initial_versions
                .iter()
                .map(|v| v.version_id.into())
                .collect::<Vec<_>>(),
            icon_url: project_builder.icon_url.clone(),
            issues_url: project_builder.issues_url.clone(),
            source_url: project_builder.source_url.clone(),
            wiki_url: project_builder.wiki_url.clone(),
            discord_url: project_builder.discord_url.clone(),
            donation_urls: project_create_data.donation_urls.clone(),
        };

        let _project_id = project_builder.insert(&mut *transaction).await?;

        if status.is_searchable() {
            let index_project = crate::search::indexing::local_import::query_one(
                project_id.into(),
                &mut *transaction,
            )
            .await?;
            indexing_queue.add(index_project);

            if let Ok(webhook_url) = dotenv::var("MODERATION_DISCORD_WEBHOOK") {
                crate::util::webhook::send_discord_webhook(response.clone(), webhook_url)
                    .await
                    .ok();
            }
        }

        Ok(HttpResponse::Ok().json(response))
    }
}

async fn create_initial_version(
    version_data: &InitialVersionData,
    project_id: ProjectId,
    author: UserId,
    all_game_versions: &[models::categories::GameVersion],
    all_loaders: &[models::categories::Loader],
    project_type: &str,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<models::version_item::VersionBuilder, CreateError> {
    if version_data.project_id.is_some() {
        return Err(CreateError::InvalidInput(String::from(
            "Found project id in initial version for new project",
        )));
    }

    version_data
        .validate()
        .map_err(|err| CreateError::ValidationError(validation_errors_to_string(err, None)))?;

    // Randomly generate a new id to be used for the version
    let version_id: VersionId = models::generate_version_id(transaction).await?.into();

    let release_channel =
        models::ChannelId::get_id(version_data.release_channel.as_str(), &mut *transaction)
            .await?
            .expect("Release Channel not found in database");

    let game_versions = version_data
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

    let loaders = version_data
        .loaders
        .iter()
        .map(|x| {
            all_loaders
                .iter()
                .find(|y| {
                    y.loader == x.0
                        && y.supported_project_types
                            .contains(&project_type.to_string())
                })
                .ok_or_else(|| CreateError::InvalidLoader(x.0.clone()))
                .map(|y| y.id)
        })
        .collect::<Result<Vec<models::LoaderId>, CreateError>>()?;

    let dependencies = version_data
        .dependencies
        .iter()
        .map(|d| models::version_item::DependencyBuilder {
            version_id: d.version_id.map(|x| x.into()),
            project_id: d.project_id.map(|x| x.into()),
            dependency_type: d.dependency_type.to_string(),
        })
        .collect::<Vec<_>>();

    let version = models::version_item::VersionBuilder {
        version_id: version_id.into(),
        project_id: project_id.into(),
        author_id: author.into(),
        name: version_data.version_title.clone(),
        version_number: version_data.version_number.clone(),
        changelog: version_data
            .version_body
            .clone()
            .unwrap_or_else(|| "".to_string()),
        files: Vec::new(),
        dependencies,
        game_versions,
        loaders,
        release_channel,
        featured: version_data.featured,
    };

    Ok(version)
}

async fn process_icon_upload(
    uploaded_files: &mut Vec<UploadedFile>,
    project_id: ProjectId,
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

        if data.len() >= 262144 {
            return Err(CreateError::InvalidInput(String::from(
                "Icons must be smaller than 256KiB",
            )));
        }

        let upload_data = file_host
            .upload_file(
                content_type,
                &format!("data/{}/icon.{}", project_id, file_extension),
                data,
            )
            .await?;

        uploaded_files.push(UploadedFile {
            file_id: upload_data.file_id,
            file_name: upload_data.file_name.clone(),
        });

        Ok(format!("{}/{}", cdn_url, upload_data.file_name))
    } else {
        Err(CreateError::InvalidIconFormat(file_extension.to_string()))
    }
}

pub fn get_image_content_type(extension: &str) -> Option<&'static str> {
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

    if !content_type.is_empty() {
        Some(content_type)
    } else {
        None
    }
}
