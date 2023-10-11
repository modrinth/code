use crate::auth::validate::get_user_record_from_bearer_token;
use crate::database::redis::RedisPool;
use crate::models::analytics::Download;
use crate::models::ids::ProjectId;
use crate::models::pats::Scopes;
use crate::queue::analytics::AnalyticsQueue;
use crate::queue::download::DownloadQueue;
use crate::queue::maxmind::MaxMindIndexer;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::guards::admin_key_guard;
use actix_web::{patch, web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use uuid::Uuid;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("admin").service(count_download));
}

#[derive(Deserialize)]
pub struct DownloadBody {
    pub url: String,
    pub project_id: ProjectId,
    pub version_name: String,

    pub ip: String,
    pub headers: HashMap<String, String>,
}

// This is an internal route, cannot be used without key
#[patch("/_count-download", guard = "admin_key_guard")]
#[allow(clippy::too_many_arguments)]
pub async fn count_download(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    maxmind: web::Data<Arc<MaxMindIndexer>>,
    analytics_queue: web::Data<Arc<AnalyticsQueue>>,
    session_queue: web::Data<AuthQueue>,
    download_body: web::Json<DownloadBody>,
    download_queue: web::Data<DownloadQueue>,
) -> Result<HttpResponse, ApiError> {
    let token = download_body
        .headers
        .iter()
        .find(|x| x.0.to_lowercase() == "authorization")
        .map(|x| &**x.1);

    let user = get_user_record_from_bearer_token(&req, token, &**pool, &redis, &session_queue)
        .await
        .ok()
        .flatten();

    let project_id: crate::database::models::ids::ProjectId = download_body.project_id.into();

    let id_option = crate::models::ids::base62_impl::parse_base62(&download_body.version_name)
        .ok()
        .map(|x| x as i64);

    let (version_id, project_id, file_type) = if let Some(version) = sqlx::query!(
        "
            SELECT v.id id, v.mod_id mod_id, file_type FROM files f
            INNER JOIN versions v ON v.id = f.version_id
            WHERE f.url = $1
            ",
        download_body.url,
    )
    .fetch_optional(pool.as_ref())
    .await?
    {
        (version.id, version.mod_id, version.file_type)
    } else if let Some(version) = sqlx::query!(
        "
        SELECT id, mod_id FROM versions
        WHERE ((version_number = $1 OR id = $3) AND mod_id = $2)
        ",
        download_body.version_name,
        project_id as crate::database::models::ids::ProjectId,
        id_option
    )
    .fetch_optional(pool.as_ref())
    .await?
    {
        (version.id, version.mod_id, None)
    } else {
        return Err(ApiError::InvalidInput(
            "Specified version does not exist!".to_string(),
        ));
    };

    if file_type.is_none() {
        download_queue
            .add(
                crate::database::models::ProjectId(project_id),
                crate::database::models::VersionId(version_id),
            )
            .await;
    }

    let url = url::Url::parse(&download_body.url)
        .map_err(|_| ApiError::InvalidInput("invalid download URL specified!".to_string()))?;

    let ip = crate::routes::analytics::convert_to_ip_v6(&download_body.ip)
        .unwrap_or_else(|_| Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped());

    analytics_queue.add_download(Download {
        id: Uuid::new_v4(),
        recorded: Utc::now().timestamp_nanos() / 100_000,
        domain: url.host_str().unwrap_or_default().to_string(),
        site_path: url.path().to_string(),
        user_id: user
            .and_then(|(scopes, x)| {
                if scopes.contains(Scopes::PERFORM_ANALYTICS) {
                    Some(x.id.0 as u64)
                } else {
                    None
                }
            })
            .unwrap_or(0),
        project_id: project_id as u64,
        version_id: version_id as u64,
        ip,
        country: maxmind.query(ip).await.unwrap_or_default(),
        user_agent: download_body
            .headers
            .get("user-agent")
            .cloned()
            .unwrap_or_default(),
        headers: download_body
            .headers
            .clone()
            .into_iter()
            .filter(|x| !crate::routes::analytics::FILTERED_HEADERS.contains(&&*x.0.to_lowercase()))
            .collect(),
    });

    Ok(HttpResponse::NoContent().body(""))
}
