use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::ids::ImageId;
use crate::models::projects::{
    Dependency, FileType, Loader, ProjectId, Version, VersionId, VersionStatus, VersionType,
};
use crate::models::v2::projects::LegacyVersion;
use crate::queue::session::AuthQueue;
use crate::routes::v3::project_creation::CreateError;
use crate::routes::{v2_reroute, v3};
use actix_multipart::Multipart;
use actix_web::web::Data;
use actix_web::{post, web, HttpRequest, HttpResponse};
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
        regex = "crate::util::validate::RE_URL_SAFE"
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
) -> Result<HttpResponse, CreateError> {
    let payload = v2_reroute::alter_actix_multipart(
        payload,
        req.headers().clone(),
        |legacy_create: InitialVersionData| {
            // Convert input data to V3 format
            let mut fields = HashMap::new();
            fields.insert(
                "game_versions".to_string(),
                json!(legacy_create.game_versions),
            );

            // TODO: will be overhauled with side-types overhaul
            // TODO: if not, should default to previous version
            fields.insert("client_side".to_string(), json!("required"));
            fields.insert("server_side".to_string(), json!("optional"));

            // TODO: Some kind of handling here to ensure project type is fine.
            // We expect the version uploaded to be of loader type modpack, but there might  not be a way to check here for that.
            // After all, theoretically, they could be creating a genuine 'fabric' mod, and modpack no longer carries information on whether its a mod or modpack,
            // as those are out to the versions.

            // Ideally this would, if the project 'should' be a modpack:
            // - change the loaders to mrpack only
            // - add loader fields to the project for the corresponding loaders

            Ok(v3::version_creation::InitialVersionData {
                project_id: legacy_create.project_id,
                file_parts: legacy_create.file_parts,
                version_number: legacy_create.version_number,
                version_title: legacy_create.version_title,
                version_body: legacy_create.version_body,
                dependencies: legacy_create.dependencies,
                release_channel: legacy_create.release_channel,
                loaders: legacy_create.loaders,
                featured: legacy_create.featured,
                primary_file: legacy_create.primary_file,
                status: legacy_create.status,
                file_types: legacy_create.file_types,
                uploaded_images: legacy_create.uploaded_images,
                ordering: legacy_create.ordering,
                fields,
            })
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
