use crate::auth::{get_user_from_headers, AuthenticationError};
use crate::database::models;
use crate::file_hosting::{FileHost, FileHostingError};
use crate::models::error::ApiError;
use crate::models::mods::{DonationLink, License, ModId, ModStatus, SideType, VersionId};
use crate::models::users::UserId;
use crate::routes::version_creation::InitialVersionData;
use crate::search::indexing::{queue::CreationQueue, IndexingError};
use actix_multipart::{Field, Multipart};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{post, HttpRequest, HttpResponse};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
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
    #[error("Indexing Error: {0}")]
    IndexingError(#[from] IndexingError),
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
            },
            description: &self.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ModCreateData {
    /// The title or name of the mod.
    pub mod_name: String,
    /// The slug of a mod, used for vanity URLs
    pub mod_slug: Option<String>,
    /// A short description of the mod.
    pub mod_description: String,
    /// A long description of the mod, in markdown.
    pub mod_body: String,
    /// A list of initial versions to upload with the created mod
    pub initial_versions: Vec<InitialVersionData>,
    /// A list of the categories that the mod is in.
    pub categories: Vec<String>,
    /// An optional link to where to submit bugs or issues with the mod.
    pub issues_url: Option<String>,
    /// An optional link to the source code for the mod.
    pub source_url: Option<String>,
    /// An optional link to the mod's wiki page or other relevant information.
    pub wiki_url: Option<String>,
    /// An optional link to the mod's license page
    pub license_url: Option<String>,
    /// An optional link to the mod's discord.
    pub discord_url: Option<String>,
    /// An optional boolean. If true, the mod will be created as a draft.
    pub is_draft: Option<bool>,
    /// The support range for the client mod
    pub client_side: SideType,
    /// The support range for the server mod
    pub server_side: SideType,
    /// The license id that the mod follows
    pub license_id: String,
    /// An optional list of all donation links the mod has
    pub donation_urls: Option<Vec<DonationLink>>,
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

/*

Mod Creation Steps:
Get logged in user
    Must match the author in the version creation

1. Data
    - Gets "data" field from multipart form; must be first
    - Verification: string lengths
    - Create versions
        - Some shared logic with version creation
        - Create list of VersionBuilders
    - Create ModBuilder

2. Upload
    - Icon: check file format & size
        - Upload to backblaze & record URL
    - Mod files
        - Check for matching version
        - File size limits?
        - Check file type
            - Eventually, malware scan
        - Upload to backblaze & create VersionFileBuilder
    -

3. Creation
    - Database stuff
    - Add mod data to indexing queue
*/

async fn mod_create_inner(
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

    let mod_id: ModId = models::generate_mod_id(transaction).await?.into();

    let mod_create_data;
    let mut versions;
    let mut versions_map = std::collections::HashMap::new();

    {
        // The first multipart field must be named "data" and contain a
        // JSON `ModCreateData` object.

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
        let create_data: ModCreateData = serde_json::from_slice(&data)?;

        {
            // Verify the lengths of various fields in the mod create data
            /*
            # ModCreateData
            mod_name: 3..=256
            mod_description: 3..=2048,
            mod_body: max of 64KiB?,
            categories: Vec<String>, 1..=256
            issues_url: 0..=2048, (Validate url?)
            source_url: 0..=2048,
            wiki_url: 0..=2048,

            initial_versions: Vec<InitialVersionData>,
            team_members: Vec<TeamMember>,

            # TeamMember:
            name: 3..=64
            role: 3..=64
            */

            check_length(3..=256, "mod name", &create_data.mod_name)?;
            check_length(3..=2048, "mod description", &create_data.mod_description)?;
            check_length(..65536, "mod body", &create_data.mod_body)?;

            if create_data.categories.len() > 3 {
                return Err(CreateError::InvalidInput(
                    "The maximum number of categories for a mod is four.".to_string(),
                ));
            }

            create_data
                .categories
                .iter()
                .map(|f| check_length(1..=256, "category", f))
                .collect::<Result<(), _>>()?;

            if let Some(url) = &create_data.issues_url {
                check_length(..=2048, "url", url)?;
            }
            if let Some(url) = &create_data.wiki_url {
                check_length(..=2048, "url", url)?;
            }
            if let Some(url) = &create_data.source_url {
                check_length(..=2048, "url", url)?;
            }

            create_data
                .initial_versions
                .iter()
                .map(|v| super::version_creation::check_version(v))
                .collect::<Result<(), _>>()?;
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
            versions
                .push(create_initial_version(data, mod_id, current_user.id, transaction).await?);
        }

        mod_create_data = create_data;
    }

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
                    "Mods can only have one icon",
                )));
            }
            // Upload the icon to the cdn
            icon_url = Some(
                process_icon_upload(
                    uploaded_files,
                    mod_id,
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
        let version_data = mod_create_data.initial_versions.get(index).unwrap();

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

    {
        // Check to make sure that all specified files were uploaded
        for (version_data, builder) in mod_create_data.initial_versions.iter().zip(versions.iter())
        {
            if version_data.file_parts.len() != builder.files.len() {
                return Err(CreateError::InvalidInput(String::from(
                    "Some files were specified in initial_versions but not uploaded",
                )));
            }
        }

        // Convert the list of category names to actual categories
        let mut categories = Vec::with_capacity(mod_create_data.categories.len());
        for category in &mod_create_data.categories {
            let id = models::categories::Category::get_id(&category, &mut *transaction)
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
        if mod_create_data.is_draft.unwrap_or(false) {
            status = ModStatus::Draft;
        } else {
            status = ModStatus::Processing;
        }

        let status_id = models::StatusId::get_id(&status, &mut *transaction)
            .await?
            .ok_or_else(|| {
                CreateError::InvalidInput(format!("Status {} does not exist.", status.clone()))
            })?;
        let client_side_id =
            models::SideTypeId::get_id(&mod_create_data.client_side, &mut *transaction)
                .await?
                .ok_or_else(|| {
                    CreateError::InvalidInput(
                        "Client side type specified does not exist.".to_string(),
                    )
                })?;

        let server_side_id =
            models::SideTypeId::get_id(&mod_create_data.server_side, &mut *transaction)
                .await?
                .ok_or_else(|| {
                    CreateError::InvalidInput(
                        "Server side type specified does not exist.".to_string(),
                    )
                })?;

        let license_id =
            models::categories::License::get_id(&mod_create_data.license_id, &mut *transaction)
                .await?
                .ok_or_else(|| {
                    CreateError::InvalidInput("License specified does not exist.".to_string())
                })?;
        let mut donation_urls = vec![];

        if let Some(urls) = &mod_create_data.donation_urls {
            for url in urls {
                let platform_id = models::DonationPlatformId::get_id(&url.id, &mut *transaction)
                    .await?
                    .ok_or_else(|| {
                        CreateError::InvalidInput(format!(
                            "Donation platform {} does not exist.",
                            url.id.clone()
                        ))
                    })?;

                donation_urls.push(models::mod_item::DonationUrl {
                    mod_id: mod_id.into(),
                    platform_id,
                    platform_short: "".to_string(),
                    platform_name: "".to_string(),
                    url: url.url.clone(),
                })
            }
        }

        let mod_builder = models::mod_item::ModBuilder {
            mod_id: mod_id.into(),
            team_id,
            title: mod_create_data.mod_name,
            description: mod_create_data.mod_description,
            body: mod_create_data.mod_body,
            icon_url,
            issues_url: mod_create_data.issues_url,
            source_url: mod_create_data.source_url,
            wiki_url: mod_create_data.wiki_url,

            license_url: mod_create_data.license_url,
            discord_url: mod_create_data.discord_url,
            categories,
            initial_versions: versions,
            status: status_id,
            client_side: client_side_id,
            server_side: server_side_id,
            license: license_id,
            slug: mod_create_data.mod_slug,
            donation_urls,
        };

        let now = chrono::Utc::now();

        let response = crate::models::mods::Mod {
            id: mod_id,
            slug: mod_builder.slug.clone(),
            team: team_id.into(),
            title: mod_builder.title.clone(),
            description: mod_builder.description.clone(),
            body: mod_builder.body.clone(),
            body_url: None,
            published: now,
            updated: now,
            status: status.clone(),
            license: License {
                id: mod_create_data.license_id.clone(),
                name: "".to_string(),
                url: mod_builder.license_url.clone(),
            },
            client_side: mod_create_data.client_side,
            server_side: mod_create_data.server_side,
            downloads: 0,
            categories: mod_create_data.categories,
            versions: mod_builder
                .initial_versions
                .iter()
                .map(|v| v.version_id.into())
                .collect::<Vec<_>>(),
            icon_url: mod_builder.icon_url.clone(),
            issues_url: mod_builder.issues_url.clone(),
            source_url: mod_builder.source_url.clone(),
            wiki_url: mod_builder.wiki_url.clone(),
            discord_url: mod_builder.discord_url.clone(),
            donation_urls: mod_create_data.donation_urls.clone(),
        };

        let _mod_id = mod_builder.insert(&mut *transaction).await?;

        if status.is_searchable() {
            let index_mod =
                crate::search::indexing::local_import::query_one(mod_id.into(), &mut *transaction)
                    .await?;
            indexing_queue.add(index_mod);
        }

        Ok(HttpResponse::Ok().json(response))
    }
}

async fn create_initial_version(
    version_data: &InitialVersionData,
    mod_id: ModId,
    author: UserId,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<models::version_item::VersionBuilder, CreateError> {
    if version_data.mod_id.is_some() {
        return Err(CreateError::InvalidInput(String::from(
            "Found mod id in initial version for new mod",
        )));
    }

    check_length(3..=256, "version name", &version_data.version_title)?;
    check_length(1..=32, "version number", &version_data.version_number)?;

    // Randomly generate a new id to be used for the version
    let version_id: VersionId = models::generate_version_id(transaction).await?.into();

    let release_channel =
        models::ChannelId::get_id(version_data.release_channel.as_str(), &mut *transaction)
            .await?
            .expect("Release Channel not found in database");

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

    let dependencies = version_data
        .dependencies
        .iter()
        .map(|x| (*x).into())
        .collect::<Vec<_>>();

    let version = models::version_item::VersionBuilder {
        version_id: version_id.into(),
        mod_id: mod_id.into(),
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
    mod_id: ModId,
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
                &format!("data/{}/icon.{}", mod_id, file_extension),
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

    if content_type != "" {
        Some(content_type)
    } else {
        None
    }
}

pub fn check_length(
    range: impl std::ops::RangeBounds<usize> + std::fmt::Debug,
    field_name: &str,
    field: &str,
) -> Result<(), CreateError> {
    use std::ops::Bound;

    let length = field.len();
    if !range.contains(&length) {
        let bounds = match (range.start_bound(), range.end_bound()) {
            (Bound::Included(a), Bound::Included(b)) => format!("between {} and {} bytes", a, b),
            (Bound::Included(a), Bound::Excluded(b)) => {
                format!("between {} and {} bytes", a, b - 1)
            }
            (Bound::Included(a), Bound::Unbounded) => format!("more than {} bytes", a),
            (Bound::Unbounded, Bound::Included(b)) => format!("less than or equal to {} bytes", b),
            (Bound::Unbounded, Bound::Excluded(b)) => format!("less than {} bytes", b),
            _ => format!("{:?}", range),
        };
        Err(CreateError::InvalidInput(format!(
            "The {} must be {}; got {}.",
            field_name, bounds, length
        )))
    } else {
        Ok(())
    }
}
