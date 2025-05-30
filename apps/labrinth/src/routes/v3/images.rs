use std::sync::Arc;

use super::threads::is_authorized_thread;
use crate::auth::checks::{is_team_member_project, is_team_member_version};
use crate::auth::get_user_from_headers;
use crate::database;
use crate::database::models::{
    project_item, report_item, thread_item, version_item,
};
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::ids::{ReportId, ThreadMessageId, VersionId};
use crate::models::images::{Image, ImageContext};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::img::upload_image_optimized;
use crate::util::routes::read_from_payload;
use actix_web::{HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("image", web::post().to(images_add));
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

pub async fn images_add(
    req: HttpRequest,
    web::Query(data): web::Query<ImageUpload>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let mut context = ImageContext::from_str(&data.context, None);

    let scopes = vec![context.relevant_scope()];

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&scopes),
    )
    .await?
    .1;

    // Attempt to associated a supplied id with the context
    // If the context cannot be found, or the user is not authorized to upload images for the context, return an error
    match &mut context {
        ImageContext::Project { project_id } => {
            if let Some(id) = data.project_id {
                let project =
                    project_item::DBProject::get(&id, &**pool, &redis).await?;
                if let Some(project) = project {
                    if is_team_member_project(
                        &project.inner,
                        &Some(user.clone()),
                        &pool,
                    )
                    .await?
                    {
                        *project_id = Some(project.inner.id.into());
                    } else {
                        return Err(ApiError::CustomAuthentication(
                            "You are not authorized to upload images for this project".to_string(),
                        ));
                    }
                } else {
                    return Err(ApiError::InvalidInput(
                        "The project could not be found.".to_string(),
                    ));
                }
            }
        }
        ImageContext::Version { version_id } => {
            if let Some(id) = data.version_id {
                let version =
                    version_item::DBVersion::get(id.into(), &**pool, &redis)
                        .await?;
                if let Some(version) = version {
                    if is_team_member_version(
                        &version.inner,
                        &Some(user.clone()),
                        &pool,
                        &redis,
                    )
                    .await?
                    {
                        *version_id = Some(version.inner.id.into());
                    } else {
                        return Err(ApiError::CustomAuthentication(
                            "You are not authorized to upload images for this version".to_string(),
                        ));
                    }
                } else {
                    return Err(ApiError::InvalidInput(
                        "The version could not be found.".to_string(),
                    ));
                }
            }
        }
        ImageContext::ThreadMessage { thread_message_id } => {
            if let Some(id) = data.thread_message_id {
                let thread_message =
                    thread_item::DBThreadMessage::get(id.into(), &**pool)
                        .await?
                        .ok_or_else(|| {
                            ApiError::InvalidInput(
                                "The thread message could not found."
                                    .to_string(),
                            )
                        })?;
                let thread = thread_item::DBThread::get(thread_message.thread_id, &**pool)
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInput(
                            "The thread associated with the thread message could not be found"
                                .to_string(),
                        )
                    })?;
                if is_authorized_thread(&thread, &user, &pool).await? {
                    *thread_message_id = Some(thread_message.id.into());
                } else {
                    return Err(ApiError::CustomAuthentication(
                        "You are not authorized to upload images for this thread message"
                            .to_string(),
                    ));
                }
            }
        }
        ImageContext::Report { report_id } => {
            if let Some(id) = data.report_id {
                let report = report_item::DBReport::get(id.into(), &**pool)
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInput(
                            "The report could not be found.".to_string(),
                        )
                    })?;
                let thread = thread_item::DBThread::get(report.thread_id, &**pool)
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInput(
                            "The thread associated with the report could not be found.".to_string(),
                        )
                    })?;
                if is_authorized_thread(&thread, &user, &pool).await? {
                    *report_id = Some(report.id.into());
                } else {
                    return Err(ApiError::CustomAuthentication(
                        "You are not authorized to upload images for this report".to_string(),
                    ));
                }
            }
        }
        ImageContext::Unknown => {
            return Err(ApiError::InvalidInput(
                "Context must be one of: project, version, thread_message, report".to_string(),
            ));
        }
    }

    // Upload the image to the file host
    let bytes = read_from_payload(
        &mut payload,
        1_048_576,
        "Icons must be smaller than 1MiB",
    )
    .await?;

    let content_length = bytes.len();
    let upload_result = upload_image_optimized(
        "data/cached_images",
        bytes.freeze(),
        &data.ext,
        None,
        None,
        &***file_host,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    let db_image: database::models::DBImage = database::models::DBImage {
        id: database::models::generate_image_id(&mut transaction).await?,
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
    db_image.insert(&mut transaction).await?;

    let image = Image {
        id: db_image.id.into(),
        url: db_image.url,
        size: db_image.size,
        created: db_image.created,
        owner_id: db_image.owner_id.into(),
        context,
    };

    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(image))
}
