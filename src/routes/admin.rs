use crate::routes::ApiError;
use crate::util::guards::admin_key_guard;
use actix_web::{patch, web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct DownloadBody {
    pub url: String,
}

// This is an internal route, cannot be used without key
#[patch("/_count-download", guard = "admin_key_guard")]
pub async fn version_count_patch(
    pool: web::Data<PgPool>,
    download_body: web::Json<DownloadBody>,
) -> Result<HttpResponse, ApiError> {
    let version = sqlx::query!(
        "
        SELECT v.id id, v.mod_id project_id FROM files f
        INNER JOIN versions v ON v.id = f.version_id
        WHERE f.url = $1
        ",
        download_body.url,
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput("Specified version does not exist!".to_string())
    })?;

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "UPDATE versions
         SET downloads = downloads + 1
         WHERE (id = $1)",
        version.id
    )
    .execute(&mut *transaction)
    .await?;

    sqlx::query!(
        "UPDATE mods
         SET downloads = downloads + 1
         WHERE (id = $1)",
        version.project_id
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::Ok().body(""))
}
