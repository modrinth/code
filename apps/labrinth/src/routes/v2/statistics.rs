use crate::routes::{
    ApiError, v2_reroute,
    v3::{self, statistics::V3Stats},
};
use actix_web::{HttpResponse, get, web};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_stats);
}

#[derive(serde::Serialize)]
pub struct V2Stats {
    pub projects: Option<i64>,
    pub versions: Option<i64>,
    pub authors: Option<i64>,
    pub files: Option<i64>,
}

#[get("statistics")]
pub async fn get_stats(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::statistics::get_stats(pool)
        .await
        .or_else(v2_reroute::flatten_404_error)?;

    match v2_reroute::extract_ok_json::<V3Stats>(response).await {
        Ok(stats) => {
            let stats = V2Stats {
                projects: stats.projects,
                versions: stats.versions,
                authors: stats.authors,
                files: stats.files,
            };
            Ok(HttpResponse::Ok().json(stats))
        }
        Err(response) => Ok(response),
    }
}
