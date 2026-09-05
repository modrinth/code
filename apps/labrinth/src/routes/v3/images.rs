use super::threads::is_authorized_thread;
use crate::auth::checks::{is_team_member_project, is_team_member_version};
use crate::auth::get_user_from_headers;
use crate::database;
use crate::database::PgPool;
use crate::database::models::{
    project_item, report_item, thread_item, version_item,
};
use crate::file_hosting::{FileHost, FileHostPublicity};
use crate::models::ids::{ReportId, ThreadMessageId, VersionId};
use crate::models::images::{Image, ImageContext};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::ApiContext as _;
use crate::util::error::Context as _;
use crate::util::img::upload_image_optimized;
use crate::util::routes::read_limited_from_payload;
use actix_web::{HttpRequest, HttpResponse, post, web};
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(images_add);
}

#[derive(Serialize, Deserialize)]
pub struct ImageUpload {
    pub ext: String,

    // Context must be an allowed context
    // currently: project, version, thread_message, report
    pub context: String,

    // Optional context id to associate with
    pub project_id: Option<String>, // allow slug or id
    pub version_id: Option<VersionId>,
    pub thread_message_id: Option<ThreadMessageId>,
    pub report_id: Option<ReportId>,
}

#[utoipa::path(
	tag = "images",
	params(
		("ext" = String, Query),
		("context" = String, Query),
		("project_id" = Option<String>, Query),
		("version_id" = Option<VersionId>, Query),
		("thread_message_id" = Option<ThreadMessageId>, Query),
		("report_id" = Option<ReportId>, Query)
	),
	request_body(content = Vec<u8>, content_type = "application/octet-stream"),
	responses((status = OK))
)]
#[post("/image")]
pub async fn images_add(
    req: HttpRequest,
    web::Query(data): web::Query<ImageUpload>,
    file_host: web::Data<dyn FileHost>,
    mut payload: web::Payload,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let mut context = ImageContext::from_str(&data.context, None);

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        context.relevant_scope(),
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    // Attempt to associated a supplied id with the context
    // If the context cannot be found, or the user is not authorized to upload images for the context, return an error
    match &mut context {
        ImageContext::Project { project_id } => {
            if let Some(id) = data.project_id {
                let project =
                    project_item::DBProject::get(&id, &**pool, &redis)
                        .await
                        .wrap_api_err("fetching project from database")?;
                if let Some(project) = project {
                    if is_team_member_project(
                        &project.inner,
                        &Some(user.clone()),
                        &pool,
                    )
                    .await
                    .wrap_api_err("checking team member project")?
                    {
                        *project_id = Some(project.inner.id.into());
                    } else {
                        return Err(ApiError::Auth(eyre::eyre!(
                            "You are not authorized to upload images for this project",
                        )));
                    }
                } else {
                    return Err(ApiError::Request(eyre::eyre!(
                        "The project could not be found.",
                    )));
                }
            }
        }
        ImageContext::Version { version_id } => {
            if let Some(id) = data.version_id {
                let version =
                    version_item::DBVersion::get(id.into(), &**pool, &redis)
                        .await
                        .wrap_internal_err("fetching version from database")?;
                if let Some(version) = version {
                    if is_team_member_version(
                        &version.inner,
                        &Some(user.clone()),
                        &pool,
                        &redis,
                    )
                    .await
                    .wrap_api_err("checking team member version")?
                    {
                        *version_id = Some(version.inner.id.into());
                    } else {
                        return Err(ApiError::Auth(eyre::eyre!(
                            "You are not authorized to upload images for this version",
                        )));
                    }
                } else {
                    return Err(ApiError::Request(eyre::eyre!(
                        "The version could not be found.",
                    )));
                }
            }
        }
        ImageContext::ThreadMessage { thread_message_id } => {
            if let Some(id) = data.thread_message_id {
                let thread_message =
                    thread_item::DBThreadMessage::get(id.into(), &**pool)
                        .await
                        .wrap_internal_err(
                            "fetching thread message from database",
                        )?
                        .wrap_request_err_with(|| {
                            "the thread message could not found.".to_string()
                        })?;
                let thread = thread_item::DBThread::get(thread_message.thread_id, &**pool)
                    .await.wrap_internal_err("fetching thread from database")?
                    .wrap_request_err_with(|| "the thread associated with the thread message could not be found"
                                .to_string())?;
                if is_authorized_thread(&thread, &user, &pool)
                    .await
                    .wrap_api_err("checking thread authorization")?
                {
                    *thread_message_id = Some(thread_message.id.into());
                } else {
                    return Err(ApiError::Auth(eyre::eyre!(
                        "You are not authorized to upload images for this thread message",
                    )));
                }
            }
        }
        ImageContext::Report { report_id } => {
            if let Some(id) = data.report_id {
                let report = report_item::DBReport::get(id.into(), &**pool)
                    .await
                    .wrap_internal_err("fetching report from database")?
                    .wrap_request_err_with(|| {
                        "the report could not be found.".to_string()
                    })?;
                let thread = thread_item::DBThread::get(
                    report.thread_id,
                    &**pool,
                )
                .await
                .wrap_internal_err("fetching thread from database")?
                .wrap_request_err_with(|| {
                    "the thread associated with the report could not be found."
                        .to_string()
                })?;
                if is_authorized_thread(&thread, &user, &pool)
                    .await
                    .wrap_api_err("checking thread authorization")?
                {
                    *report_id = Some(report.id.into());
                } else {
                    return Err(ApiError::Auth(eyre::eyre!(
                        "You are not authorized to upload images for this report",
                    )));
                }
            }
        }
        ImageContext::Unknown => {
            return Err(ApiError::Request(eyre::eyre!(
                "Context must be one of: project, version, thread_message, report",
            )));
        }
    }

    // Upload the image to the file host
    let bytes = read_limited_from_payload(
        &mut payload,
        1_048_576,
        "Icons must be smaller than 1MiB",
    )
    .await
    .wrap_api_err("executing `read_limited_from_payload`")?;

    let content_length = bytes.len();
    let upload_result = upload_image_optimized(
        "data/cached_images",
        FileHostPublicity::Public, // FIXME: Maybe use private images for threads
        bytes.freeze(),
        &data.ext,
        None,
        None,
        &**file_host,
    )
    .await
    .wrap_api_err("uploading image")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let db_image: database::models::DBImage = database::models::DBImage {
        id: database::models::generate_image_id(&mut transaction)
            .await
            .wrap_internal_err("generating image ID")?,
        url: upload_result.url,
        raw_url: upload_result.raw_url,
        size: content_length as u64,
        created: chrono::Utc::now(),
        owner_id: database::models::DBUserId::from(user.id),
        context: context.context_as_str().to_string(),
        project_id: if let ImageContext::Project {
            project_id: Some(id),
        } = context
        {
            Some(crate::database::models::DBProjectId::from(id))
        } else {
            None
        },
        version_id: if let ImageContext::Version {
            version_id: Some(id),
        } = context
        {
            Some(database::models::DBVersionId::from(id))
        } else {
            None
        },
        thread_message_id: if let ImageContext::ThreadMessage {
            thread_message_id: Some(id),
        } = context
        {
            Some(database::models::DBThreadMessageId::from(id))
        } else {
            None
        },
        report_id: if let ImageContext::Report {
            report_id: Some(id),
        } = context
        {
            Some(database::models::DBReportId::from(id))
        } else {
            None
        },
    };

    // Insert
    db_image
        .insert(&mut transaction)
        .await
        .wrap_internal_err("inserting database records for `images_add`")?;

    let image = Image {
        id: db_image.id.into(),
        url: db_image.url,
        size: db_image.size,
        created: db_image.created,
        owner_id: db_image.owner_id.into(),
        context,
    };

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    Ok(HttpResponse::Ok().json(image))
}
