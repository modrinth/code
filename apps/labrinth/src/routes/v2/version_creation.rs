use crate::database::models::loader_fields::VersionField;
use crate::database::models::{project_item, version_item};
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::ids::{ImageId, ProjectId, VersionId};
use crate::models::projects::{
    Dependency, FileType, Loader, Version, VersionStatus, VersionType,
};
use crate::models::v2::projects::LegacyVersion;
use crate::queue::moderation::AutomatedModerationQueue;
use crate::queue::session::AuthQueue;
use crate::routes::v3::project_creation::CreateError;
use crate::routes::v3::version_creation;
use crate::routes::{v2_reroute, v3};
use actix_multipart::Multipart;
use actix_web::http::header::ContentDisposition;
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse, post, web};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use validator::Validate;

pub fn default_requested_status() -> VersionStatus {
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
        regex(path = *crate::util::validate::RE_URL_SAFE)
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
    pub game_versions: Vec<String>,
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

    // The ordering relative to other versions
    pub ordering: Option<i32>,
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
    payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: Data<AuthQueue>,
    moderation_queue: Data<AutomatedModerationQueue>,
) -> Result<HttpResponse, CreateError> {
    let payload = v2_reroute::alter_actix_multipart(
        payload,
        req.headers().clone(),
        |legacy_create: InitialVersionData,
         content_dispositions: Vec<ContentDisposition>| {
            let client = client.clone();
            let redis = redis.clone();
            async move {
                // Convert input data to V3 format
                let mut fields = HashMap::new();
                fields.insert(
                    "game_versions".to_string(),
                    json!(legacy_create.game_versions),
                );

                // Get all possible side-types for loaders given- we will use these to check if we need to convert/apply singleplayer, etc.
                let loaders =
                    match v3::tags::loader_list(client.clone(), redis.clone())
                        .await
                    {
                        Ok(loader_response) => {
                            (v2_reroute::extract_ok_json::<
                                Vec<v3::tags::LoaderData>,
                            >(loader_response)
                            .await)
                                .unwrap_or_default()
                        }
                        Err(_) => vec![],
                    };

                let loader_fields_aggregate = loaders
                    .into_iter()
                    .filter_map(|loader| {
                        if legacy_create
                            .loaders
                            .contains(&Loader(loader.name.clone()))
                        {
                            Some(loader.supported_fields)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                // Copies side types of another version of the project.
                // If no version exists, defaults to all false.
                // This is inherently lossy, but not much can be done about it, as side types are no longer associated with projects,
                // so the 'missing' ones can't be easily accessed, and versions do need to have these fields explicitly set.
                let side_type_loader_field_names = [
                    "singleplayer",
                    "client_and_server",
                    "client_only",
                    "server_only",
                ];

                // Check if loader_fields_aggregate contains any of these side types
                // We assume these four fields are linked together.
                if loader_fields_aggregate
                    .iter()
                    .any(|f| side_type_loader_field_names.contains(&f.as_str()))
                {
                    // If so, we get the fields of the example version of the project, and set the side types to match.
                    fields.extend(
                        side_type_loader_field_names
                            .iter()
                            .map(|f| (f.to_string(), json!(false))),
                    );
                    if let Some(example_version_fields) =
                        get_example_version_fields(
                            legacy_create.project_id,
                            client,
                            &redis,
                        )
                        .await?
                    {
                        fields.extend(
                            example_version_fields.into_iter().filter_map(
                                |f| {
                                    if side_type_loader_field_names
                                        .contains(&f.field_name.as_str())
                                    {
                                        Some((
                                            f.field_name,
                                            f.value.serialize_internal(),
                                        ))
                                    } else {
                                        None
                                    }
                                },
                            ),
                        );
                    }
                }
                // Handle project type via file extension prediction
                let mut project_type = None;
                for file_part in &legacy_create.file_parts {
                    if let Some(ext) = file_part.split('.').next_back() {
                        match ext {
                            "mrpack" | "mrpack-primary" => {
                                project_type = Some("modpack");
                                break;
                            }
                            // No other type matters
                            _ => {}
                        }
                        break;
                    }
                }

                // Similarly, check actual content disposition for mrpacks, in case file_parts is wrong
                for content_disposition in content_dispositions {
                    // Uses version_create functions to get the file name and extension
                    let (_, file_extension) =
                        version_creation::get_name_ext(&content_disposition)?;
                    crate::util::ext::project_file_type(file_extension)
                        .ok_or_else(|| {
                            CreateError::InvalidFileType(
                                file_extension.to_string(),
                            )
                        })?;

                    if file_extension == "mrpack" {
                        project_type = Some("modpack");
                        break;
                    }
                }

                // Modpacks now use the "mrpack" loader, and loaders are converted to loader fields.
                // Setting of 'project_type' directly is removed, it's loader-based now.
                if project_type == Some("modpack") {
                    fields.insert(
                        "mrpack_loaders".to_string(),
                        json!(legacy_create.loaders),
                    );
                }

                let loaders = if project_type == Some("modpack") {
                    vec![Loader("mrpack".to_string())]
                } else {
                    legacy_create.loaders
                };

                Ok(v3::version_creation::InitialVersionData {
                    project_id: legacy_create.project_id,
                    file_parts: legacy_create.file_parts,
                    version_number: legacy_create.version_number,
                    version_title: legacy_create.version_title,
                    version_body: legacy_create.version_body,
                    dependencies: legacy_create.dependencies,
                    release_channel: legacy_create.release_channel,
                    loaders,
                    featured: legacy_create.featured,
                    primary_file: legacy_create.primary_file,
                    status: legacy_create.status,
                    file_types: legacy_create.file_types,
                    uploaded_images: legacy_create.uploaded_images,
                    ordering: legacy_create.ordering,
                    fields,
                })
            }
        },
    )
    .await?;

    // Call V3 project creation
    let response = v3::version_creation::version_create(
        req,
        payload,
        client.clone(),
        redis.clone(),
        file_host,
        session_queue,
        moderation_queue,
    )
    .await?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Version>(response).await {
        Ok(version) => {
            let v2_version = LegacyVersion::from(version);
            Ok(HttpResponse::Ok().json(v2_version))
        }
        Err(response) => Ok(response),
    }
}

// Gets version fields of an example version of a project, if one exists.
async fn get_example_version_fields(
    project_id: Option<ProjectId>,
    pool: Data<PgPool>,
    redis: &RedisPool,
) -> Result<Option<Vec<VersionField>>, CreateError> {
    let project_id = match project_id {
        Some(project_id) => project_id,
        None => return Ok(None),
    };

    let vid = match project_item::DBProject::get_id(
        project_id.into(),
        &**pool,
        redis,
    )
    .await?
    .and_then(|p| p.versions.first().cloned())
    {
        Some(vid) => vid,
        None => return Ok(None),
    };

    let example_version =
        match version_item::DBVersion::get(vid, &**pool, redis).await? {
            Some(version) => version,
            None => return Ok(None),
        };
    Ok(Some(example_version.version_fields))
}

// under /api/v1/version/{version_id}
#[post("{version_id}/file")]
pub async fn upload_file_to_version(
    req: HttpRequest,
    url_data: web::Path<(VersionId,)>,
    payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, CreateError> {
    // Returns NoContent, so no need to convert to V2
    let response = v3::version_creation::upload_file_to_version(
        req,
        url_data,
        payload,
        client.clone(),
        redis.clone(),
        file_host,
        session_queue,
    )
    .await?;
    Ok(response)
}
