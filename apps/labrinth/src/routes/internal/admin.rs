use crate::auth::validate::get_user_record_from_bearer_token;
use crate::database::models::thread_item::ThreadMessageBuilder;
use crate::database::redis::RedisPool;
use crate::models::analytics::Download;
use crate::models::ids::ProjectId;
use crate::models::pats::Scopes;
use crate::models::threads::MessageBody;
use crate::queue::analytics::AnalyticsQueue;
use crate::queue::maxmind::MaxMindIndexer;
use crate::queue::moderation::AUTOMOD_ID;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::search::SearchConfig;
use crate::util::date::get_current_tenths_of_ms;
use crate::util::guards::admin_key_guard;
use actix_web::{HttpRequest, HttpResponse, patch, post, web};
use serde::Deserialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::fmt::Write;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tracing::info;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("admin")
            .service(count_download)
            .service(force_reindex)
            .service(delphi_result_ingest),
    );
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

    analytics_queue.add_download(Download {
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
            .filter(|x| {
                !crate::routes::analytics::FILTERED_HEADERS
                    .contains(&&*x.0.to_lowercase())
            })
            .collect(),
    });

    Ok(HttpResponse::NoContent().body(""))
}

#[post("/_force_reindex", guard = "admin_key_guard")]
pub async fn force_reindex(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, ApiError> {
    use crate::search::indexing::index_projects;
    let redis = redis.get_ref();
    index_projects(pool.as_ref().clone(), redis.clone(), &config).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct DelphiIngest {
    pub url: String,
    pub project_id: crate::models::ids::ProjectId,
    pub version_id: crate::models::ids::VersionId,
    pub issues: HashMap<String, HashMap<String, String>>,
}

#[post("/_delphi", guard = "admin_key_guard")]
pub async fn delphi_result_ingest(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    body: web::Json<DelphiIngest>,
) -> Result<HttpResponse, ApiError> {
    if body.issues.is_empty() {
        info!("No issues found for file {}", body.url);
        return Ok(HttpResponse::NoContent().finish());
    }

    let webhook_url = dotenvy::var("DELPHI_SLACK_WEBHOOK")?;

    let project = crate::database::models::DBProject::get_id(
        body.project_id.into(),
        &**pool,
        &redis,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(format!(
            "Project {} does not exist",
            body.project_id
        ))
    })?;

    let mut header = format!("Suspicious traces found at {}", body.url);

    for (issue, trace) in &body.issues {
        for (path, code) in trace {
            write!(
                &mut header,
                "\n issue {issue} found at file {path}: \n ```\n{code}\n```"
            )
            .unwrap();
        }
    }

    crate::util::webhook::send_slack_webhook(
        body.project_id,
        &pool,
        &redis,
        webhook_url,
        Some(header),
    )
    .await
    .ok();

    let mut thread_header = format!(
        "Suspicious traces found at [version {}](https://modrinth.com/project/{}/version/{})",
        body.version_id, body.project_id, body.version_id
    );

    for (issue, trace) in &body.issues {
        for path in trace.keys() {
            write!(
                &mut thread_header,
                "\n\n- issue {issue} found at file {path}"
            )
            .unwrap();
        }

        if trace.is_empty() {
            write!(&mut thread_header, "\n\n- issue {issue} found").unwrap();
        }
    }

    let mut transaction = pool.begin().await?;
    ThreadMessageBuilder {
        author_id: Some(crate::database::models::DBUserId(AUTOMOD_ID)),
        body: MessageBody::Text {
            body: thread_header,
            private: true,
            replying_to: None,
            associated_images: vec![],
        },
        thread_id: project.thread_id,
        hide_identity: false,
    }
    .insert(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
