use crate::auth::get_user_from_headers;
use crate::database::models::shared_instance_item::{
    DBSharedInstance, DBSharedInstanceUser, DBSharedInstanceVersion,
};
use crate::database::models::{
    DBSharedInstanceId, DBSharedInstanceVersionId,
    generate_shared_instance_version_id,
};
use crate::database::redis::RedisPool;
use crate::file_hosting::{FileHost, FileHostPublicity};
use crate::models::ids::{SharedInstanceId, SharedInstanceVersionId};
use crate::models::pats::Scopes;
use crate::models::shared_instances::{
    SharedInstanceUserPermissions, SharedInstanceVersion,
};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::routes::v3::project_creation::UploadedFile;
use crate::util::ext::MRPACK_MIME_TYPE;
use actix_web::http::header::ContentLength;
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse, web};
use bytes::BytesMut;
use chrono::Utc;
use futures_util::StreamExt;
use hex::FromHex;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;

const MAX_FILE_SIZE: usize = 500 * 1024 * 1024;
const MAX_FILE_SIZE_TEXT: &str = "500 MB";

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "shared-instance/{id}/version",
        web::post().to(shared_instance_version_create),
    );
}

#[allow(clippy::too_many_arguments)]
pub async fn shared_instance_version_create(
    req: HttpRequest,
    pool: Data<PgPool>,
    payload: web::Payload,
    web::Header(ContentLength(content_length)): web::Header<ContentLength>,
    redis: Data<RedisPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    info: web::Path<(SharedInstanceId,)>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    if content_length > MAX_FILE_SIZE {
        return Err(ApiError::InvalidInput(format!(
            "File size exceeds the maximum limit of {MAX_FILE_SIZE_TEXT}"
        )));
    }

    let mut transaction = pool.begin().await?;
    let mut uploaded_files = vec![];

    let result = shared_instance_version_create_inner(
        req,
        &pool,
        payload,
        content_length,
        &redis,
        &***file_host,
        info.into_inner().0.into(),
        &session_queue,
        &mut transaction,
        &mut uploaded_files,
    )
    .await;

    if result.is_err() {
        let undo_result = super::project_creation::undo_uploads(
            &***file_host,
            &uploaded_files,
        )
        .await;
        let rollback_result = transaction.rollback().await;

        undo_result?;
        if let Err(e) = rollback_result {
            return Err(e.into());
        }
    } else {
        transaction.commit().await?;
    }

    result
}

#[allow(clippy::too_many_arguments)]
async fn shared_instance_version_create_inner(
    req: HttpRequest,
    pool: &PgPool,
    mut payload: web::Payload,
    content_length: usize,
    redis: &RedisPool,
    file_host: &dyn FileHost,
    instance_id: DBSharedInstanceId,
    session_queue: &AuthQueue,
    transaction: &mut Transaction<'_, Postgres>,
    uploaded_files: &mut Vec<UploadedFile>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        pool,
        redis,
        session_queue,
        Scopes::SHARED_INSTANCE_VERSION_CREATE,
    )
    .await?
    .1;

    let Some(instance) = DBSharedInstance::get(instance_id, pool).await? else {
        return Err(ApiError::NotFound);
    };
    if !user.role.is_mod() && instance.owner_id != user.id.into() {
        let permissions = DBSharedInstanceUser::get_user_permissions(
            instance_id,
            user.id.into(),
            pool,
        )
        .await?;
        if let Some(permissions) = permissions {
            if !permissions
                .contains(SharedInstanceUserPermissions::UPLOAD_VERSION)
            {
                return Err(ApiError::CustomAuthentication(
                    "You do not have permission to upload a version for this shared instance.".to_string()
                ));
            }
        } else {
            return Err(ApiError::NotFound);
        }
    }

    let version_id =
        generate_shared_instance_version_id(&mut *transaction).await?;

    let mut file_data = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|_| {
            ApiError::InvalidInput(
                "Unable to parse bytes in payload sent!".to_string(),
            )
        })?;

        if file_data.len() + chunk.len() <= MAX_FILE_SIZE {
            file_data.extend_from_slice(&chunk);
        } else {
            file_data
                .extend_from_slice(&chunk[..MAX_FILE_SIZE - file_data.len()]);
            break;
        }
    }

    let file_data = file_data.freeze();
    let file_path = format!(
        "shared_instance/{}.mrpack",
        SharedInstanceVersionId::from(version_id),
    );

    let upload_data = file_host
        .upload_file(
            MRPACK_MIME_TYPE,
            &file_path,
            FileHostPublicity::Private,
            file_data,
        )
        .await?;

    uploaded_files.push(UploadedFile {
        name: file_path,
        publicity: upload_data.file_publicity,
    });

    let sha512 = Vec::<u8>::from_hex(upload_data.content_sha512).unwrap();

    let new_version = DBSharedInstanceVersion {
        id: version_id,
        shared_instance_id: instance_id,
        size: content_length as u64,
        sha512,
        created: Utc::now(),
    };
    new_version.insert(transaction).await?;

    sqlx::query!(
        "UPDATE shared_instances SET current_version_id = $1 WHERE id = $2",
        new_version.id as DBSharedInstanceVersionId,
        instance_id as DBSharedInstanceId,
    )
    .execute(&mut **transaction)
    .await?;

    let version: SharedInstanceVersion = new_version.into();
    Ok(HttpResponse::Created().json(version))
}
