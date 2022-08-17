use crate::models::ids::ProjectId;
use crate::routes::ApiError;
use crate::util::guards::admin_key_guard;
use crate::DownloadQueue;
use actix_web::{patch, web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct DownloadBody {
    pub url: String,
    pub hash: ProjectId,
    pub version_name: String,
}

// This is an internal route, cannot be used without key
#[patch("/_count-download", guard = "admin_key_guard")]
pub async fn count_download(
    pool: web::Data<PgPool>,
    download_body: web::Json<DownloadBody>,
    download_queue: web::Data<Arc<DownloadQueue>>,
) -> Result<HttpResponse, ApiError> {
    let project_id: crate::database::models::ids::ProjectId =
        download_body.hash.into();

    let id_option = crate::models::ids::base62_impl::parse_base62(
        &download_body.version_name,
    )
    .ok()
    .map(|x| x as i64);

    let (version_id, project_id) = if let Some(version) = sqlx::query!(
        "SELECT id, mod_id FROM versions
         WHERE ((version_number = $1 OR id = $3) AND mod_id = $2)",
        download_body.version_name,
        project_id as crate::database::models::ids::ProjectId,
        id_option
    )
    .fetch_optional(pool.as_ref())
    .await?
    {
        (version.id, version.mod_id)
    } else if let Some(version) = sqlx::query!(
        "
        SELECT v.id id, v.mod_id project_id FROM files f
        INNER JOIN versions v ON v.id = f.version_id
        WHERE f.url = $1
        ",
        download_body.url,
    )
    .fetch_optional(pool.as_ref())
    .await?
    {
        (version.id, version.project_id)
    } else {
        return Err(ApiError::InvalidInput(
            "Specified version does not exist!".to_string(),
        ));
    };

    download_queue
        .add(
            crate::database::models::ProjectId(project_id),
            crate::database::models::VersionId(version_id),
        )
        .await;

    let client = reqwest::Client::new();

    client
        .post(format!("{}downloads", dotenv::var("ARIADNE_URL")?))
        .header("Modrinth-Admin", dotenv::var("ARIADNE_ADMIN_KEY")?)
        .json(&json!({
            "url": download_body.url,
            "project_id": download_body.hash
        }))
        .send()
        .await
        .ok();

    Ok(HttpResponse::Ok().body(""))
}
