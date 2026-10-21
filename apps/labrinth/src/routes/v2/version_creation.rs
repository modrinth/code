use crate::database::PgPool;
use crate::database::models::{project_item, version_item};
use crate::file_hosting::FileHost;
use crate::models::exp::minecraft::{Environment, ModVersion, MrpackLoader};
use crate::models::ids::{ImageId, ProjectId, VersionId};
use crate::models::projects::{
    Dependency, FileType, Loader, Version, VersionStatus, VersionType,
};
use crate::models::v2::projects::LegacySideType;
use crate::models::v2::projects::LegacyVersion;
use crate::queue::session::AuthQueue;
use crate::routes::v3::project_creation::CreateError;
use crate::routes::v3::version_creation;
use crate::routes::{v2_reroute, v3};
use crate::search::SearchState;
use crate::util::MinecraftMeta;
use crate::util::kafka::KafkaClientState;
use actix_multipart::Multipart;
use actix_web::http::header::ContentDisposition;
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse, post, web};
use minecraft_meta_model::model::MinecraftVersionName;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;
use xredis::RedisPool;

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
        regex(path = *crate::util::validate::RE_URL_SAFE_RELAXED)
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
    pub game_versions: Vec<MinecraftVersionName>,
    pub environment: Option<Environment>,
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

pub(super) fn parse_mrpack_loaders(
    loaders: &[Loader],
) -> Result<Vec<MrpackLoader>, eyre::Report> {
    loaders.iter().map(|loader| loader.0.parse()).collect()
}

pub(super) fn environment_from_sides(
    client: LegacySideType,
    server: LegacySideType,
) -> Environment {
    use LegacySideType::{Optional, Required, Unsupported};
    match (client, server) {
        (Required, Required) => Environment::ClientAndServer,
        (Required, Unsupported) => Environment::ClientOnly,
        (Required, Optional) => Environment::ClientOnlyServerOptional,
        (Unsupported, Required) => Environment::ServerOnly,
        (Optional, Required) => Environment::ServerOnlyClientOptional,
        (Optional, Optional) => Environment::ClientOrServer,
        _ => Environment::Unknown,
    }
}

// under `/api/v1/version`
/// Create a version on an existing project.
#[utoipa::path(
	tag = "version creation",
    post,
    operation_id = "createVersion",
    request_body(
        content(("multipart/form-data")),
        description = "Multipart payload containing `data` and uploaded files"
    ),
    responses(
        (status = 200, description = "Expected response to a valid request", body = LegacyVersion),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["VERSION_CREATE"]))
)]
#[post("/version")]
pub async fn version_create(
    minecraft_meta: web::Data<MinecraftMeta>,
    req: HttpRequest,
    payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<dyn FileHost>,
    session_queue: Data<AuthQueue>,
    kafka_client: Data<KafkaClientState>,
    search_state: Data<SearchState>,
) -> Result<HttpResponse, CreateError> {
    let payload = v2_reroute::alter_actix_multipart(
        payload,
        req.headers().clone(),
        |mut legacy_create: InitialVersionData,
         content_dispositions: Vec<ContentDisposition>| {
            let client = client.clone();
            let redis = redis.clone();
            async move {
                if let Some(project_id) = legacy_create.project_id {
                    legacy_create.project_id = Some(
                        crate::routes::resolve_ref(
                            &project_id.to_string(),
                            client.as_ref(),
                            redis.as_ref(),
                        )
                        .await?
                        .unwrap_or(project_id),
                    );
                }
                let dependency_project_refs = legacy_create
                    .dependencies
                    .iter()
                    .filter_map(|dependency| dependency.project_id)
                    .map(|project_id| project_id.to_string())
                    .collect::<Vec<_>>();
                let resolved_dependency_project_ids =
                    crate::routes::resolve_refs(
                        &dependency_project_refs,
                        client.as_ref(),
                        redis.as_ref(),
                    )
                    .await?;
                for (dependency, resolved_project_id) in legacy_create
                    .dependencies
                    .iter_mut()
                    .filter(|dependency| dependency.project_id.is_some())
                    .zip(resolved_dependency_project_ids)
                {
                    if let Some(project_id) = resolved_project_id {
                        dependency.project_id = Some(project_id);
                    }
                }

                let environment = match legacy_create.environment {
                    Some(environment) => environment,
                    None => get_example_environment(
                        legacy_create.project_id,
                        client,
                        &redis,
                    )
                    .await?
                    .unwrap_or(Environment::Unknown),
                };
                // Handle project type via file extension prediction
                let mut project_type = None;
                for file_part in &legacy_create.file_parts {
                    if let Some("mrpack" | "mrpack-primary") =
                        file_part.split('.').next_back()
                    {
                        project_type = Some("modpack");
                        break;
                    }
                }

                // Similarly, check actual content disposition for mrpacks, in case file_parts is wrong
                for content_disposition in content_dispositions {
                    // Uses version_create functions to get the file name and extension
                    let (_, file_extension) =
                        version_creation::get_name_and_extension(
                            &content_disposition,
                        )?;
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

                let component = ModVersion {
                    game_versions: legacy_create.game_versions,
                    environment: Some(environment),
                    mrpack_loaders: if project_type == Some("modpack") {
                        parse_mrpack_loaders(&legacy_create.loaders)?
                    } else {
                        Vec::new()
                    },
                };

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
                    minecraft_mod: Some(component),
                    fields: HashMap::new(),
                })
            }
        },
    )
    .await?;

    // Call V3 project creation
    let response = v3::version_creation::version_create(
        minecraft_meta.clone(),
        req,
        payload,
        client.clone(),
        redis.clone(),
        file_host,
        session_queue,
        kafka_client,
        search_state,
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

async fn get_example_environment(
    project_id: Option<ProjectId>,
    pool: Data<PgPool>,
    redis: &RedisPool,
) -> Result<Option<Environment>, CreateError> {
    let Some(project_id) = project_id else {
        return Ok(None);
    };

    let Some(vid) =
        project_item::DBProject::get_id(project_id.into(), &**pool, redis)
            .await?
            .and_then(|p| p.versions.first().copied())
    else {
        return Ok(None);
    };

    let Some(example_version) =
        version_item::DBVersion::get(vid, &**pool, redis).await?
    else {
        return Ok(None);
    };
    Ok(example_version
        .inner
        .components
        .minecraft_mod
        .and_then(|component| component.environment))
}

// under /api/v1/version/{version_id}
/// Add files to an existing version.
#[utoipa::path(
	tag = "version creation",
    post,
    operation_id = "addFilesToVersion",
    params(
        ("version_id" = VersionId, Path, description = "The ID of the version")
    ),
    request_body(
        content(("multipart/form-data")),
        description = "Multipart payload containing files to upload"
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["VERSION_WRITE"]))
)]
#[post("/{version_id}/file")]
pub async fn upload_file_to_version(
    minecraft_meta: web::Data<MinecraftMeta>,
    req: HttpRequest,
    url_data: web::Path<(VersionId,)>,
    payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<dyn FileHost>,
    session_queue: web::Data<AuthQueue>,
    kafka_client: web::Data<KafkaClientState>,
    search_state: Data<SearchState>,
) -> Result<HttpResponse, CreateError> {
    // Returns NoContent, so no need to convert to V2
    let response = v3::version_creation::upload_file_to_version(
        minecraft_meta.clone(),
        req,
        url_data,
        payload,
        client.clone(),
        redis.clone(),
        file_host,
        session_queue,
        kafka_client,
        search_state,
    )
    .await?;
    Ok(response)
}
