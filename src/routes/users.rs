use crate::auth::{check_is_moderator_from_headers, get_user_from_headers};
use crate::models::users::{Role, UserId};
use crate::routes::ApiError;
use actix_web::{delete, get, web, HttpRequest, HttpResponse};
use sqlx::PgPool;

#[get("user")]
pub async fn user_auth_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(
        get_user_from_headers(
            req.headers(),
            &mut *pool
                .acquire()
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?,
        )
        .await
        .map_err(|_| ApiError::AuthenticationError)?,
    ))
}

#[get("{id}")]
pub async fn user_get(
    info: web::Path<(UserId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.0;
    let user_data = crate::database::models::User::get(id.into(), &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if let Some(data) = user_data {
        let response = crate::models::users::User {
            id: data.id.into(),
            github_id: data.github_id.map(|i| i as u64),
            username: data.username,
            name: data.name,
            email: None,
            avatar_url: data.avatar_url,
            bio: data.bio,
            created: data.created,
            role: Role::from_string(&*data.role),
        };
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

// TODO: Make this actually do stuff
#[delete("{id}")]
pub async fn user_delete(
    req: HttpRequest,
    info: web::Path<(UserId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(
        req.headers(),
        &mut *pool
            .acquire()
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?,
    )
    .await
    .map_err(|_| ApiError::AuthenticationError)?;

    let _id = info.0;
    let result = Some(());

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
