use crate::auth::validate::get_user_record_from_bearer_token;
use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::models::analytics::{Download, DownloadReason};
use crate::models::ids::ProjectId;
use crate::models::pats::Scopes;
use crate::queue::analytics::AnalyticsQueue;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::search::SearchBackend;
use crate::util::date::get_current_tenths_of_ms;
use crate::util::error::Context;
use crate::util::guards::admin_key_guard;
use crate::util::tags::valid_download_tags;
use actix_web::{HttpRequest, HttpResponse, patch, post, web};
use eyre::eyre;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tracing::trace;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(
        utoipa_actix_web::scope("/admin")
            .service(count_download)
            .service(force_reindex),
    );
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DownloadBody {
    pub url: String,
    pub project_id: ProjectId,
    pub version_name: String,

    pub ip: String,
    pub headers: HashMap<String, String>,
}

/// Extra data attached to each download request, transmitted through the
/// [`DOWNLOAD_META_HEADER`] header.
#[derive(Debug, Clone, Deserialize)]
pub struct DownloadMeta {
    pub reason: Option<DownloadReason>,
    pub game_version: Option<String>,
    pub loader: Option<String>,
}

pub const DOWNLOAD_META_HEADER: &str = "modrinth-download-meta";

// This is an internal route, cannot be used without key
#[utoipa::path(
    patch,
    operation_id = "countDownload",
    responses(
        (status = 204, description = "Download counted successfully"),
        (status = 400, description = "Invalid input")
    )
)]
#[patch("/_count-download", guard = "admin_key_guard")]
#[allow(clippy::too_many_arguments)]
pub async fn count_download(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    analytics_queue: web::Data<Arc<AnalyticsQueue>>,
    session_queue: web::Data<AuthQueue>,
    download_body: web::Json<DownloadBody>,
) -> Result<HttpResponse, ApiError> {
    let token = download_body
        .headers
        .iter()
        .find(|x| x.0.to_lowercase() == "authorization")
        .map(|x| &**x.1);

    let user = get_user_record_from_bearer_token(
        &req,
        token,
        &**pool,
        &redis,
        &session_queue,
        false,
    )
    .await
    .ok()
    .flatten();

    let project_id: crate::database::models::ids::DBProjectId =
        download_body.project_id.into();

    let id_option =
        ariadne::ids::base62_impl::parse_base62(&download_body.version_name)
            .ok()
            .map(|x| x as i64);

    let (version_id, project_id) = if let Some(version) = sqlx::query!(
        "
            SELECT v.id id, v.mod_id mod_id FROM files f
            INNER JOIN versions v ON v.id = f.version_id
            WHERE f.url = $1
            ",
        download_body.url,
    )
    .fetch_optional(pool.as_ref())
    .await?
    {
        (version.id, version.mod_id)
    } else if let Some(version) = sqlx::query!(
        "
        SELECT id, mod_id FROM versions
        WHERE ((version_number = $1 OR id = $3) AND mod_id = $2)
        ",
        download_body.version_name,
        project_id as crate::database::models::ids::DBProjectId,
        id_option
    )
    .fetch_optional(pool.as_ref())
    .await?
    {
        (version.id, version.mod_id)
    } else {
        return Err(ApiError::InvalidInput(
            "Specified version does not exist!".to_string(),
        ));
    };

    let url = url::Url::parse(&download_body.url).map_err(|_| {
        ApiError::InvalidInput("invalid download URL specified!".to_string())
    })?;

    let ip = crate::util::ip::convert_to_ip_v6(&download_body.ip)
        .unwrap_or_else(|_| Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped());

    let meta =
        if let Some(meta) = download_body.headers.get(DOWNLOAD_META_HEADER) {
            serde_json::from_str::<DownloadMeta>(meta)
                .map(Some)
                .wrap_request_err("invalid download meta")?
        } else {
            None
        };

    if let Some(meta) = &meta {
        let valid_download_tags = valid_download_tags(&pool, &redis)
            .await
            .wrap_internal_err("failed to fetch valid download tags")?;
        if let Some(loader) = &meta.loader
            && !valid_download_tags.loaders.contains(loader)
        {
            return Err(ApiError::Request(eyre!(
                "invalid download loader specified"
            )));
        }

        if let Some(game_version) = &meta.game_version
            && !valid_download_tags.game_versions.contains(game_version)
        {
            return Err(ApiError::Request(eyre!(
                "invalid download game version specified"
            )));
        }
    }

    let download = Download {
        recorded: get_current_tenths_of_ms(),
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
        country: download_body
            .headers
            .get("cf-ipcountry")
            .cloned()
            .unwrap_or_default(),
        user_agent: download_body
            .headers
            .get("user-agent")
            .cloned()
            .unwrap_or_default(),
        headers: download_body
            .headers
            .clone()
            .into_iter()
            .filter(|x| {
                !crate::routes::analytics::FILTERED_HEADERS
                    .contains(&&*x.0.to_lowercase())
            })
            .collect(),
        reason: meta
            .as_ref()
            .and_then(|m| m.reason.as_ref())
            .map(|s| s.to_string())
            .unwrap_or_default(),
        game_version: meta
            .as_ref()
            .and_then(|m| m.game_version.as_ref())
            .map(|s| s.to_string())
            .unwrap_or_default(),
        loader: meta
            .as_ref()
            .and_then(|m| m.loader.as_ref())
            .map(|s| s.to_string())
            .unwrap_or_default(),
    };
    trace!("added download {download:#?}");

    analytics_queue.add_download(download);

    Ok(HttpResponse::NoContent().body(""))
}

#[utoipa::path(
    post,
    operation_id = "forceReindex",
    responses(
        (status = 204, description = "Search index rebuilt successfully"),
        (status = 401, description = "Unauthorized")
    )
)]
#[post("/_force_reindex", guard = "admin_key_guard")]
pub async fn force_reindex(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    search_backend: web::Data<dyn SearchBackend>,
) -> Result<HttpResponse, ApiError> {
    let redis = redis.get_ref();
    search_backend
        .index_projects(pool.as_ref().clone(), redis.clone())
        .await
        .wrap_internal_err("failed to index projects")?;
    Ok(HttpResponse::NoContent().finish())
}
