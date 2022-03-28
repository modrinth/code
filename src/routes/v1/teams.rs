use crate::models::teams::{Permissions, TeamId};
use crate::models::users::UserId;
use crate::routes::ApiError;
use crate::util::auth::get_user_from_headers;
use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// A member of a team
#[derive(Serialize, Deserialize, Clone)]
pub struct TeamMember {
    /// The ID of the team this team member is a member of
    pub team_id: TeamId,
    /// The ID of the user associated with the member
    pub user_id: UserId,
    /// The role of the user in the team
    pub role: String,
    /// A bitset containing the user's permissions in this team
    pub permissions: Option<Permissions>,
    /// Whether the user has joined the team or is just invited to it
    pub accepted: bool,
}

#[get("{id}/members")]
pub async fn team_members_get(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let members_data =
        crate::database::models::TeamMember::get_from_team(id.into(), &**pool)
            .await?;

    let current_user = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(user) = current_user {
        let team_member =
            crate::database::models::TeamMember::get_from_user_id(
                id.into(),
                user.id.into(),
                &**pool,
            )
            .await
            .map_err(ApiError::Database)?;

        if team_member.is_some() {
            let team_members: Vec<TeamMember> = members_data
                .into_iter()
                .map(|data| TeamMember {
                    team_id: id,
                    user_id: data.user_id.into(),
                    role: data.role,
                    permissions: Some(data.permissions),
                    accepted: data.accepted,
                })
                .collect();

            return Ok(HttpResponse::Ok().json(team_members));
        }
    }

    let mut team_members: Vec<TeamMember> = Vec::new();

    for team_member in members_data {
        if team_member.accepted {
            team_members.push(TeamMember {
                team_id: id,
                user_id: team_member.user_id.into(),
                role: team_member.role,
                permissions: None,
                accepted: team_member.accepted,
            })
        }
    }

    Ok(HttpResponse::Ok().json(team_members))
}
