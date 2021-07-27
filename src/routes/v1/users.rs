use crate::database::models::User;
use crate::models::ids::UserId;
use crate::models::projects::{ProjectId, ProjectStatus};
use crate::routes::ApiError;
use crate::util::auth::get_user_from_headers;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};
use sqlx::PgPool;

#[get("{user_id}/mods")]
pub async fn mods_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await.ok();

    let id_option =
        crate::database::models::User::get_id_from_username_or_id(&*info.into_inner().0, &**pool)
            .await?;

    if let Some(id) = id_option {
        let user_id: UserId = id.into();

        let project_data = if let Some(current_user) = user {
            if current_user.role.is_mod() || current_user.id == user_id {
                User::get_projects_private(id, &**pool).await?
            } else {
                User::get_projects(id, ProjectStatus::Approved.as_str(), &**pool).await?
            }
        } else {
            User::get_projects(id, ProjectStatus::Approved.as_str(), &**pool).await?
        };

        let response = project_data
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<crate::models::ids::ProjectId>>();

        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("{id}/follows")]
pub async fn user_follows(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let id_option =
        crate::database::models::User::get_id_from_username_or_id(&*info.into_inner().0, &**pool)
            .await?;

    if let Some(id) = id_option {
        if !user.role.is_mod() && user.id != id.into() {
            return Err(ApiError::CustomAuthenticationError(
                "You do not have permission to see the projects this user follows!".to_string(),
            ));
        }

        use futures::TryStreamExt;

        let projects: Vec<ProjectId> = sqlx::query!(
            "
            SELECT mf.mod_id FROM mod_follows mf
            WHERE mf.follower_id = $1
            ",
            id as crate::database::models::ids::UserId,
        )
        .fetch_many(&**pool)
        .try_filter_map(|e| async { Ok(e.right().map(|m| ProjectId(m.mod_id as u64))) })
        .try_collect::<Vec<ProjectId>>()
        .await?;

        Ok(HttpResponse::Ok().json(projects))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
