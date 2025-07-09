use crate::database::redis::RedisPool;
use crate::models::reports::Report;
use crate::models::v2::reports::LegacyReport;
use crate::queue::session::AuthQueue;
use crate::routes::{ApiError, v2_reroute, v3};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(reports_get);
    cfg.service(reports);
    cfg.service(report_create);
    cfg.service(report_edit);
    cfg.service(report_delete);
    cfg.service(report_get);
}

#[post("report")]
pub async fn report_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Payload,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response =
        v3::reports::report_create(req, pool, body, redis, session_queue)
            .await
            .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Report>(response).await {
        Ok(report) => {
            let report = LegacyReport::from(report);
            Ok(HttpResponse::Ok().json(report))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Deserialize)]
pub struct ReportsRequestOptions {
    #[serde(default = "default_count")]
    count: i16,
    #[serde(default = "default_all")]
    all: bool,
}

fn default_count() -> i16 {
    100
}
fn default_all() -> bool {
    true
}

#[get("report")]
pub async fn reports(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    count: web::Query<ReportsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::reports::reports(
        req,
        pool,
        redis,
        web::Query(v3::reports::ReportsRequestOptions {
            count: count.count,
            all: count.all,
        }),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Report>>(response).await {
        Ok(reports) => {
            let reports: Vec<_> =
                reports.into_iter().map(LegacyReport::from).collect();
            Ok(HttpResponse::Ok().json(reports))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Deserialize)]
pub struct ReportIds {
    pub ids: String,
}

#[get("reports")]
pub async fn reports_get(
    req: HttpRequest,
    web::Query(ids): web::Query<ReportIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::reports::reports_get(
        req,
        web::Query(v3::reports::ReportIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Report>>(response).await {
        Ok(report_list) => {
            let report_list: Vec<_> =
                report_list.into_iter().map(LegacyReport::from).collect();
            Ok(HttpResponse::Ok().json(report_list))
        }
        Err(response) => Ok(response),
    }
}

#[get("report/{id}")]
pub async fn report_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    info: web::Path<(crate::models::ids::ReportId,)>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response =
        v3::reports::report_get(req, pool, redis, info, session_queue)
            .await
            .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Report>(response).await {
        Ok(report) => {
            let report = LegacyReport::from(report);
            Ok(HttpResponse::Ok().json(report))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Deserialize, Validate)]
pub struct EditReport {
    #[validate(length(max = 65536))]
    pub body: Option<String>,
    pub closed: Option<bool>,
}

#[patch("report/{id}")]
pub async fn report_edit(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    info: web::Path<(crate::models::ids::ReportId,)>,
    session_queue: web::Data<AuthQueue>,
    edit_report: web::Json<EditReport>,
) -> Result<HttpResponse, ApiError> {
    let edit_report = edit_report.into_inner();
    // Returns NoContent, so no need to convert
    v3::reports::report_edit(
        req,
        pool,
        redis,
        info,
        session_queue,
        web::Json(v3::reports::EditReport {
            body: edit_report.body,
            closed: edit_report.closed,
        }),
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[delete("report/{id}")]
pub async fn report_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    info: web::Path<(crate::models::ids::ReportId,)>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::reports::report_delete(req, pool, info, redis, session_queue)
        .await
        .or_else(v2_reroute::flatten_404_error)
}
