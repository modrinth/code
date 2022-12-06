use crate::database::models::User;
use crate::models::ids::UserId;
use crate::models::projects::ProjectId;
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

    let id_option = crate::database::models::User::get_id_from_username_or_id(
        &info.into_inner().0,
        &**pool,
    )
    .await?;

    if let Some(id) = id_option {
        let user_id: UserId = id.into();

        let can_view_private = user
            .map(|y| y.role.is_mod() || y.id == user_id)
            .unwrap_or(false);

        let project_data = User::get_projects(id, &**pool).await?;

        let response: Vec<_> =
            crate::database::Project::get_many(project_data, &**pool)
                .await?
                .into_iter()
                .filter(|x| can_view_private || x.status.is_approved())
                .map(|x| x.id.into())
                .collect::<Vec<ProjectId>>();

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
    let id_option = crate::database::models::User::get_id_from_username_or_id(
        &info.into_inner().0,
        &**pool,
    )
    .await?;

    if let Some(id) = id_option {
        if !user.role.is_admin() && user.id != id.into() {
            return Err(ApiError::CustomAuthentication(
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
        .try_filter_map(|e| async {
            Ok(e.right().map(|m| ProjectId(m.mod_id as u64)))
        })
        .try_collect::<Vec<ProjectId>>()
        .await?;

        Ok(HttpResponse::Ok().json(projects))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
