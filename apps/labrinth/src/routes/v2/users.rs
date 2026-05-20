use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::notifications::Notification;
use crate::models::projects::Project;
use crate::models::users::{Badges, Role, User};
use crate::models::v2::notifications::LegacyNotification;
use crate::models::v2::projects::LegacyProject;
use crate::models::v2::user::LegacyUser;
use crate::queue::session::AuthQueue;
use crate::routes::{ApiError, v2_reroute, v3};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(user_auth_get);
    cfg.service(users_get);
    cfg.service(
        utoipa_actix_web::scope("/user")
            .service(user_get)
            .service(projects_list)
            .service(user_delete)
            .service(user_edit)
            .service(user_icon_edit)
            .service(user_icon_delete)
            .service(user_notifications)
            .service(user_follows),
    );
}

/// Get the current user from the authorization header.
#[utoipa::path(
    get,
    operation_id = "getUserFromAuth",
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["USER_READ"]))
)]
#[get("/user")]
pub async fn user_auth_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::users::user_auth_get(req, pool, redis, session_queue)
        .await
        .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<User>(response).await {
        Ok(user) => {
            let user = LegacyUser::from(user);
            Ok(HttpResponse::Ok().json(user))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserIds {
    pub ids: String,
}

/// Get multiple users by ID.
#[utoipa::path(
    get,
    operation_id = "getUsers",
    params(("ids" = String, Query, description = "The JSON array of user IDs")),
    responses((status = 200, description = "Expected response to a valid request"))
)]
#[get("/users")]
pub async fn users_get(
    req: HttpRequest,
    web::Query(ids): web::Query<UserIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::users::users_get(
        req,
        web::Query(v3::users::UserIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<User>>(response).await {
        Ok(users) => {
            let legacy_users: Vec<LegacyUser> =
                users.into_iter().map(LegacyUser::from).collect();
            Ok(HttpResponse::Ok().json(legacy_users))
        }
        Err(response) => Ok(response),
    }
}

/// Get a user by ID or username.
#[utoipa::path(
    get,
    operation_id = "getUser",
    params(("id" = String, Path, description = "The ID or username of the user")),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/{id}")]
pub async fn user_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::users::user_get(req, info, pool, redis, session_queue)
        .await
        .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<User>(response).await {
        Ok(user) => {
            let user = LegacyUser::from(user);
            Ok(HttpResponse::Ok().json(user))
        }
        Err(response) => Ok(response),
    }
}

/// Get a user's projects.
#[utoipa::path(
    get,
    operation_id = "getUserProjects",
    params(("user_id" = String, Path, description = "The ID or username of the user")),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/{user_id}/projects")]
pub async fn projects_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::users::projects_list(
        req,
        info,
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert to V2 projects
    match v2_reroute::extract_ok_json::<Vec<Project>>(response).await {
        Ok(project) => {
            let legacy_projects =
                LegacyProject::from_many(project, &**pool, &redis).await?;
            Ok(HttpResponse::Ok().json(legacy_projects))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct EditUser {
    #[validate(length(min = 1, max = 39), regex(path = *crate::util::validate::RE_USERNAME))]
    pub username: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 64), regex(path = *crate::util::validate::RE_USERNAME))]
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 160))]
    pub bio: Option<Option<String>>,
    pub role: Option<Role>,
    pub badges: Option<Badges>,
    pub allow_friend_requests: Option<bool>,
}

/// Modify a user.
#[utoipa::path(
    patch,
    operation_id = "modifyUser",
    params(("id" = String, Path, description = "The ID or username of the user")),
    request_body = EditUser,
    responses(
        (status = 204, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["USER_WRITE"]))
)]
#[patch("/{id}")]
pub async fn user_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    new_user: web::Json<EditUser>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let new_user = new_user.into_inner();
    // Returns NoContent, so we don't need to convert to V2
    v3::users::user_edit(
        req,
        info,
        web::Json(v3::users::EditUser {
            username: new_user.username,
            bio: new_user.bio,
            role: new_user.role,
            badges: new_user.badges,
            venmo_handle: None,
            allow_friend_requests: new_user.allow_friend_requests,
        }),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct Extension {
    pub ext: String,
}

/// Change a user's avatar.
#[utoipa::path(
    patch,
    operation_id = "changeUserIcon",
    params(
        ("id" = String, Path, description = "The ID or username of the user"),
        (
            "ext" = String,
            Query,
            description = "Image extension (png, jpg, jpeg, bmp, gif, webp, svg, svgz, rgb)"
        )
    ),
    request_body(
        content(
            ("image/png"),
            ("image/jpeg"),
            ("image/bmp"),
            ("image/gif"),
            ("image/webp"),
            ("image/svg+xml")
        )
    ),
    responses(
        (status = 204, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["USER_WRITE"]))
)]
#[patch("/{id}/icon")]
#[allow(clippy::too_many_arguments)]
pub async fn user_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert to V2
    v3::users::user_icon_edit(
        web::Query(v3::users::Extension { ext: ext.ext }),
        req,
        info,
        pool,
        redis,
        file_host,
        payload,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Remove a user's avatar.
#[utoipa::path(
    delete,
    operation_id = "deleteUserIcon",
    params(("id" = String, Path, description = "The ID or username of the user")),
    responses(
        (status = 204, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["USER_WRITE"]))
)]
#[delete("/{id}/icon")]
pub async fn user_icon_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert to V2
    v3::users::user_icon_delete(
        req,
        info,
        pool,
        redis,
        file_host,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Delete a user by ID or username.
#[utoipa::path(
    delete,
    operation_id = "deleteUser",
    params(("id" = String, Path, description = "The ID or username of the user")),
    responses(
        (status = 204, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["USER_DELETE"]))
)]
#[delete("/{id}")]
pub async fn user_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert to V2
    v3::users::user_delete(req, info, pool, redis, session_queue)
        .await
        .map(|()| HttpResponse::NoContent().body(""))
        .or_else(v2_reroute::flatten_404_error)
}

/// Get projects followed by a user.
#[utoipa::path(
    get,
    operation_id = "getFollowedProjects",
    params(("id" = String, Path, description = "The ID or username of the user")),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["USER_READ"]))
)]
#[get("/{id}/follows")]
pub async fn user_follows(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::users::user_follows(
        req,
        info,
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert to V2 projects
    match v2_reroute::extract_ok_json::<Vec<Project>>(response).await {
        Ok(project) => {
            let legacy_projects =
                LegacyProject::from_many(project, &**pool, &redis).await?;
            Ok(HttpResponse::Ok().json(legacy_projects))
        }
        Err(response) => Ok(response),
    }
}

/// Get notifications for a user.
#[utoipa::path(
    get,
    operation_id = "getUserNotifications",
    params(("id" = String, Path, description = "The ID or username of the user")),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["NOTIFICATION_READ"]))
)]
#[get("/{id}/notifications")]
pub async fn user_notifications(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response =
        v3::users::user_notifications(req, info, pool, redis, session_queue)
            .await
            .or_else(v2_reroute::flatten_404_error)?;
    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Notification>>(response).await {
        Ok(notifications) => {
            let legacy_notifications: Vec<LegacyNotification> = notifications
                .into_iter()
                .map(LegacyNotification::from)
                .collect();
            Ok(HttpResponse::Ok().json(legacy_notifications))
        }
        Err(response) => Ok(response),
    }
}
