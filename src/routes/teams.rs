use crate::database::models::TeamMember;
use crate::models::teams::TeamId;
use crate::routes::ApiError;
use actix_web::{get, web, HttpResponse};
use sqlx::PgPool;

#[get("{id}/members")]
pub async fn team_members_get(
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let members_data = TeamMember::get_from_team(id.into(), &**pool).await?;

    let team_members: Vec<crate::models::teams::TeamMember> = members_data
        .into_iter()
        .map(|data| crate::models::teams::TeamMember {
            user_id: data.user_id.into(),
            name: data.name,
            role: data.role,
        })
        .collect();

    Ok(HttpResponse::Ok().json(team_members))
}
