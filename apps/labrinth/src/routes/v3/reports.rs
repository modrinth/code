use crate::auth::{check_is_moderator_from_headers, get_user_from_headers};
use crate::database;
use crate::database::PgPool;
use crate::database::models::SharedInstanceId;
use crate::database::models::image_item;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::thread_item::{
    ThreadBuilder, ThreadMessageBuilder,
};
use crate::env::ENV;
use crate::models::ids::{ImageId, OrganizationId};
use crate::models::ids::{ProjectId, VersionId};
use crate::models::images::{Image, ImageContext};
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::reports::{ItemType, Report};
use crate::models::threads::{MessageBody, ThreadType};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::ApiContext as _;
use crate::util::error::Context;
use crate::util::http::HTTP_CLIENT;
use crate::util::img;
use crate::util::routes::read_typed_from_payload;
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use ariadne::ids::UserId;
use ariadne::ids::base62_impl::{parse_base62, to_base62};
use chrono::Utc;
use serde::Deserialize;
use validator::Validate;
use xredis::RedisPool;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(report_create_route)
        .service(reports_route)
        .service(reports_get_route)
        .service(report_get_route)
        .service(report_edit_route)
        .service(report_delete_route);
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateReport {
    pub report_type: String,
    pub item_id: String,
    pub item_type: ItemType,
    pub body: String,
    // Associations to uploaded images
    #[validate(length(max = 10))]
    #[serde(default)]
    pub uploaded_images: Vec<ImageId>,
}

#[utoipa::path(
	tag = "reports",
	request_body = CreateReport,
	responses((status = OK))
)]
#[post("/report")]
pub async fn report_create_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Payload,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    report_create(req, pool, body, redis, session_queue).await
}

pub async fn report_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    mut body: web::Payload,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REPORT_CREATE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let new_report: CreateReport = read_typed_from_payload(&mut body)
        .await
        .wrap_api_err("reading request payload")?;

    let id = crate::database::models::generate_report_id(&mut transaction)
        .await
        .wrap_internal_err("generating report ID")?;
    let report_type = crate::database::models::categories::ReportType::get_id(
        &new_report.report_type,
        &mut transaction,
    )
    .await
    .wrap_internal_err("generating report ID")?
    .wrap_request_err_with(|| {
        format!("invalid report type: `{}`", new_report.report_type)
    })?;

    let mut report = crate::database::models::report_item::DBReport {
        id,
        report_type_id: report_type,
        project_id: None,
        version_id: None,
        user_id: None,
        organization_id: None,
        shared_instance_id: None,
        shared_instance_version_id: None,
        body: new_report.body.clone(),
        reporter: current_user.id.into(),
        created: Utc::now(),
        closed: false,
    };

    match new_report.item_type {
        ItemType::Project => {
            let project_id = ProjectId(
                parse_base62(new_report.item_id.as_str())
                    .wrap_request_err("parsing reported project ID")?,
            );

            let result = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM mods WHERE id = $1)",
                project_id.0 as i64
            )
            .fetch_one(&mut transaction)
            .await
            .wrap_internal_err("querying database for `report_create`")?;

            if !result.exists.unwrap_or(false) {
                return Err(ApiError::Request(eyre::eyre!(format!(
                    "Project could not be found: {}",
                    new_report.item_id
                ))));
            }

            report.project_id = Some(project_id.into())
        }
        ItemType::Version => {
            let version_id = VersionId(
                parse_base62(new_report.item_id.as_str())
                    .wrap_request_err("parsing reported version ID")?,
            );

            let result = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM versions WHERE id = $1)",
                version_id.0 as i64
            )
            .fetch_one(&mut transaction)
            .await
            .wrap_internal_err("querying database for `report_create`")?;

            if !result.exists.unwrap_or(false) {
                return Err(ApiError::Request(eyre::eyre!(format!(
                    "Version could not be found: {}",
                    new_report.item_id
                ))));
            }

            report.version_id = Some(version_id.into())
        }
        ItemType::User => {
            let user_id = UserId(
                parse_base62(new_report.item_id.as_str())
                    .wrap_request_err("parsing reported user ID")?,
            );

            let result = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)",
                user_id.0 as i64
            )
            .fetch_one(&mut transaction)
            .await
            .wrap_internal_err("querying database for `report_create`")?;

            if !result.exists.unwrap_or(false) {
                return Err(ApiError::Request(eyre::eyre!(format!(
                    "User could not be found: {}",
                    new_report.item_id
                ))));
            }

            report.user_id = Some(user_id.into())
        }
        ItemType::Organization => {
            let organization_id = OrganizationId(
                parse_base62(new_report.item_id.as_str())
                    .wrap_request_err("parsing reported organization ID")?,
            );

            let result = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM organizations WHERE id = $1)",
                organization_id.0 as i64
            )
            .fetch_one(&mut transaction)
            .await
            .wrap_internal_err("querying database for `report_create`")?;

            if !result.exists.unwrap_or(false) {
                return Err(ApiError::Request(eyre::eyre!(
                    "Organization could not be found: {}",
                    new_report.item_id
                )));
            }

            report.organization_id = Some(organization_id.into())
        }
        ItemType::SharedInstance => {
            // parsing
            let (instance_part, version_part) = new_report
                .item_id
                .split_once('/')
                .wrap_request_err_with(|| "shared instance reports must format the item ID as `instance_id/version_id`"
                            .to_string())?;

            let shared_instance_id = SharedInstanceId(
                parse_base62(instance_part)
                    .wrap_request_err("parsing reported shared instance ID")?
                    as i64,
            );
            let shared_instance_version_id: i32 =
                version_part.parse().wrap_request_err(format!(
                    "shared instance version is not a number: `{version_part}`"
                ))?;

            // validation
            let url = format!(
                "{}/v1/instances/{}",
                ENV.SHARED_INSTANCES_URL,
                to_base62(shared_instance_id.0 as u64)
            );
            let instance_response = HTTP_CLIENT
                .get(&url)
                .bearer_auth(&ENV.SHARED_INSTANCES_KEY)
                .send()
                .await
                .wrap_internal_err(
                    "failed to reach the shared instance service (instance lookup)",
                )?;

            if !instance_response.status().is_success() {
                return Err(ApiError::Request(eyre::eyre!(format!(
                    "Shared instance could not be found: {instance_part}"
                ))));
            }

            let version_response = HTTP_CLIENT
                .get(format!("{url}/versions/{version_part}"))
                .bearer_auth(&ENV.SHARED_INSTANCES_KEY)
                .send()
                .await
                .wrap_internal_err("failed to reach the shared instance service (version lookup)")?;

            if !version_response.status().is_success() {
                return Err(ApiError::Request(eyre::eyre!(format!(
                    "Shared instance version could not be found: {instance_part}/{version_part}"
                ))));
            }

            report.shared_instance_id = Some(shared_instance_id);
            report.shared_instance_version_id = Some(shared_instance_version_id)
        }
        ItemType::Unknown => {
            return Err(ApiError::Request(eyre::eyre!(format!(
                "Invalid report item type: {}",
                new_report.item_type.as_str()
            ))));
        }
    }

    report
        .insert(&mut transaction)
        .await
        .wrap_internal_err("inserting database records for `report_create`")?;

    for image_id in new_report.uploaded_images {
        if let Some(db_image) =
            image_item::DBImage::get(image_id.into(), &mut transaction, &redis)
                .await
                .wrap_internal_err("fetching image from database")?
        {
            let image: Image = db_image.into();
            if !matches!(image.context, ImageContext::Report { .. })
                || image.context.inner_id().is_some()
            {
                return Err(ApiError::Request(eyre::eyre!(format!(
                    "Image {image_id} is not unused and in the 'report' context"
                ))));
            }

            sqlx::query!(
                "
                UPDATE uploaded_images
                SET report_id = $1
                WHERE id = $2
                ",
                id.0 as i64,
                image_id.0 as i64
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err("querying database for `report_create`")?;

            image_item::DBImage::clear_cache(image.id.into(), &redis)
                .await
                .wrap_internal_err("clearing cached data from Redis")?;
        } else {
            return Err(ApiError::Request(eyre::eyre!(format!(
                "Image {image_id} could not be found"
            ))));
        }
    }

    let thread_id = ThreadBuilder {
        type_: ThreadType::Report,
        members: vec![],
        project_id: None,
        report_id: Some(report.id),
    }
    .insert(&mut transaction)
    .await
    .wrap_internal_err("inserting database records for `report_create`")?;

    // Notify the reporter that the report has been submitted
    NotificationBuilder {
        body: NotificationBody::ReportSubmitted {
            report_id: id.into(),
        },
    }
    .insert(current_user.id.into(), &mut transaction, &redis)
    .await
    .wrap_internal_err("inserting database records for `report_create`")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    Ok(HttpResponse::Ok().json(Report {
        id: id.into(),
        report_type: new_report.report_type.clone(),
        item_id: match report.shared_instance_id {
            Some(shared_instance_id) => to_base62(shared_instance_id.0 as u64),
            None => new_report.item_id.clone(),
        },
        item_type: new_report.item_type.clone(),
        shared_instance_version_id: report.shared_instance_version_id,
        reporter: current_user.id,
        body: new_report.body.clone(),
        created: Utc::now(),
        closed: false,
        thread_id: thread_id.into(),
    }))
}

#[derive(Deserialize)]
pub struct ReportsRequestOptions {
    #[serde(default = "default_count")]
    pub count: u16,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub all: bool,
}

fn default_count() -> u16 {
    100
}

#[utoipa::path(
	tag = "reports",
	params(
		("count" = Option<u16>, Query),
		("offset" = Option<u32>, Query),
		("all" = Option<bool>, Query)
	),
	responses((status = OK))
)]
#[get("/report")]
pub async fn reports_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ReportsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    reports(req, pool, redis, request_opts, session_queue).await
}

pub async fn reports(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ReportsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REPORT_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    use futures::stream::TryStreamExt;

    let report_ids = if user.role.is_mod() && request_opts.all {
        sqlx::query!(
            "
            SELECT id FROM reports
            WHERE closed = FALSE
            ORDER BY created ASC
            OFFSET $2
            LIMIT $1
            ",
            request_opts.count as i64,
            request_opts.offset as i64
        )
        .fetch(&**pool)
        .map_ok(|m| crate::database::models::ids::DBReportId(m.id))
        .try_collect::<Vec<crate::database::models::ids::DBReportId>>()
        .await
        .wrap_internal_err("fetching report IDs from database")?
    } else {
        sqlx::query!(
            "
            SELECT id FROM reports
            WHERE closed = FALSE AND reporter = $1
            ORDER BY created ASC
            OFFSET $3
            LIMIT $2
            ",
            user.id.0 as i64,
            request_opts.count as i64,
            request_opts.offset as i64
        )
        .fetch(&**pool)
        .map_ok(|m| crate::database::models::ids::DBReportId(m.id))
        .try_collect::<Vec<crate::database::models::ids::DBReportId>>()
        .await
        .wrap_internal_err("querying database for `reports`")?
    };

    let query_reports =
        crate::database::models::report_item::DBReport::get_many(
            &report_ids,
            &**pool,
        )
        .await
        .wrap_internal_err("fetching reports from database")?;

    let mut reports: Vec<Report> = Vec::new();

    for x in query_reports {
        reports.push(x.into());
    }

    Ok(HttpResponse::Ok().json(reports))
}

#[derive(Deserialize)]
pub struct ReportIds {
    pub ids: String,
}

#[utoipa::path(
	tag = "reports",
	params(("ids" = String, Query)),
	responses((status = OK))
)]
#[get("/reports")]
pub async fn reports_get_route(
    req: HttpRequest,
    ids: web::Query<ReportIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    reports_get(req, ids, pool, redis, session_queue).await
}

pub async fn reports_get(
    req: HttpRequest,
    web::Query(ids): web::Query<ReportIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let report_ids: Vec<crate::database::models::ids::DBReportId> =
        serde_json::from_str::<Vec<crate::models::ids::ReportId>>(&ids.ids)
            .wrap_request_err("deserializing JSON data")?
            .into_iter()
            .map(|x| x.into())
            .collect();

    let reports_data =
        crate::database::models::report_item::DBReport::get_many(
            &report_ids,
            &**pool,
        )
        .await
        .wrap_internal_err("fetching reports from database")?;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REPORT_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let all_reports = reports_data
        .into_iter()
        .filter(|x| user.role.is_mod() || x.reporter == user.id.into())
        .map(|x| x.into())
        .collect::<Vec<Report>>();

    Ok(HttpResponse::Ok().json(all_reports))
}

#[utoipa::path(tag = "reports", responses((status = OK)))]
#[get("/report/{id}")]
pub async fn report_get_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    info: web::Path<(crate::models::ids::ReportId,)>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    report_get(req, pool, redis, info, session_queue).await
}

pub async fn report_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    info: web::Path<(crate::models::ids::ReportId,)>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REPORT_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let id = info.into_inner().0.into();

    let report =
        crate::database::models::report_item::DBReport::get(id, &**pool)
            .await
            .wrap_internal_err("fetching report from database")?;

    if let Some(report) = report {
        if !user.role.is_mod() && report.reporter != user.id.into() {
            return Err(ApiError::NotFound(eyre::eyre!("resource not found")));
        }

        let report: Report = report.into();
        Ok(HttpResponse::Ok().json(report))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct EditReport {
    #[validate(length(max = 65536))]
    pub body: Option<String>,
    pub closed: Option<bool>,
}

#[utoipa::path(tag = "reports", responses((status = NO_CONTENT)))]
#[patch("/report/{id}")]
pub async fn report_edit_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    info: web::Path<(crate::models::ids::ReportId,)>,
    session_queue: web::Data<AuthQueue>,
    edit_report: web::Json<EditReport>,
) -> Result<HttpResponse, ApiError> {
    report_edit(req, pool, redis, info, session_queue, edit_report).await
}

pub async fn report_edit(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    info: web::Path<(crate::models::ids::ReportId,)>,
    session_queue: web::Data<AuthQueue>,
    edit_report: web::Json<EditReport>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REPORT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let id = info.into_inner().0.into();

    let report =
        crate::database::models::report_item::DBReport::get(id, &**pool)
            .await
            .wrap_internal_err("fetching report from database")?;

    if let Some(report) = report {
        if !user.role.is_mod() && report.reporter != user.id.into() {
            return Err(ApiError::NotFound(eyre::eyre!("resource not found")));
        }

        let mut transaction = pool
            .begin()
            .await
            .wrap_internal_err("starting database transaction")?;

        if let Some(edit_body) = &edit_report.body {
            sqlx::query!(
                "
                UPDATE reports
                SET body = $1
                WHERE (id = $2)
                ",
                edit_body,
                id as crate::database::models::ids::DBReportId,
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err("updating edit body in database")?;
        }

        if let Some(edit_closed) = edit_report.closed {
            if !user.role.is_mod() {
                return Err(ApiError::Request(eyre::eyre!(
                    "You cannot reopen a report!",
                )));
            }

            ThreadMessageBuilder {
                author_id: Some(user.id.into()),
                body: if !edit_closed && report.closed {
                    MessageBody::ThreadReopen
                } else {
                    MessageBody::ThreadClosure
                },
                thread_id: report.thread_id,
                hide_identity: user.role.is_mod(),
            }
            .insert(&mut transaction)
            .await
            .wrap_internal_err(
                "inserting database records for `report_edit`",
            )?;

            NotificationBuilder {
                body: NotificationBody::ReportStatusUpdated {
                    report_id: id.into(),
                },
            }
            .insert(report.reporter, &mut transaction, &redis)
            .await
            .wrap_internal_err(
                "inserting database records for `report_edit`",
            )?;

            sqlx::query!(
                "
                UPDATE reports
                SET closed = $1
                WHERE (id = $2)
                ",
                edit_closed,
                id as crate::database::models::ids::DBReportId,
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err("updating database records for `report_edit`")?;
        }

        // delete any images no longer in the body
        let checkable_strings: Vec<&str> = vec![&edit_report.body]
            .into_iter()
            .filter_map(|x: &Option<String>| x.as_ref().map(|y| y.as_str()))
            .collect();
        let image_context = ImageContext::Report {
            report_id: Some(id.into()),
        };
        img::delete_unused_images(
            image_context,
            checkable_strings,
            &mut transaction,
            &redis,
        )
        .await
        .wrap_api_err("deleting unused images")?;

        transaction
            .commit()
            .await
            .wrap_internal_err("committing database transaction")?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[utoipa::path(tag = "reports", responses((status = NO_CONTENT)))]
#[delete("/report/{id}")]
pub async fn report_delete_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    info: web::Path<(crate::models::ids::ReportId,)>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    report_delete(req, pool, info, redis, session_queue).await
}

pub async fn report_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    info: web::Path<(crate::models::ids::ReportId,)>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REPORT_DELETE,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let id = info.into_inner().0;
    let context = ImageContext::Report {
        report_id: Some(id),
    };
    let uploaded_images = database::models::DBImage::get_many_contexted(
        context,
        &mut transaction,
    )
    .await
    .wrap_internal_err("fetching images from database")?;
    for image in uploaded_images {
        image_item::DBImage::remove(image.id, &mut transaction, &redis)
            .await
            .wrap_internal_err("deleting image from database")?;
    }

    let result = crate::database::models::report_item::DBReport::remove_full(
        id.into(),
        &mut transaction,
    )
    .await
    .wrap_internal_err("deleting report from database")?;
    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}
