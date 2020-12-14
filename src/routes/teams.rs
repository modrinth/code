use crate::auth::get_user_from_headers;
use crate::database::models::TeamMember;
use crate::models::teams::{Permissions, TeamId};
use crate::models::users::UserId;
use crate::routes::ApiError;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[get("{id}/members")]
pub async fn team_members_get(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let members_data = TeamMember::get_from_team(id.into(), &**pool).await?;

    let current_user = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(user) = current_user {
        let team_member = TeamMember::get_from_user_id(id.into(), user.id.into(), &**pool)
            .await
            .map_err(ApiError::DatabaseError)?;

        if team_member.is_some() {
            let team_members: Vec<crate::models::teams::TeamMember> = members_data
                .into_iter()
                .map(|data| crate::models::teams::TeamMember {
                    user_id: data.user_id.into(),
                    role: data.role,
                    permissions: Some(data.permissions),
                })
                .collect();

            return Ok(HttpResponse::Ok().json(team_members));
        }
    }

    let team_members: Vec<crate::models::teams::TeamMember> = members_data
        .into_iter()
        .map(|data| crate::models::teams::TeamMember {
            user_id: data.user_id.into(),
            role: data.role,
            permissions: None,
        })
        .collect();

    Ok(HttpResponse::Ok().json(team_members))
}

#[post("{id}/join")]
pub async fn join_team(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let team_id = info.into_inner().0.into();
    let current_user = get_user_from_headers(req.headers(), &**pool).await?;

    let member =
        TeamMember::get_from_user_id_pending(team_id, current_user.id.into(), &**pool).await?;

    if let Some(member) = member {
        if member.accepted {
            return Err(ApiError::InvalidInputError(
                "You are already a member of this team".to_string(),
            ));
        }
        let mut transaction = pool
            .begin()
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;

        // Edit Team Member to set Accepted to True
        TeamMember::edit_team_member(
            team_id,
            current_user.id.into(),
            None,
            None,
            Some(true),
            &mut transaction,
        )
        .await?;

        transaction
            .commit()
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
    } else {
        return Err(ApiError::InvalidInputError(
            "There is no pending request from this team".to_string(),
        ));
    }

    Ok(HttpResponse::Ok().body(""))
}

fn default_role() -> String {
    "Member".to_string()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewTeamMember {
    pub user_id: UserId,
    #[serde(default = "default_role")]
    pub role: String,
    #[serde(default = "Permissions::default")]
    pub permissions: Permissions,
}

#[post("{id}/members")]
pub async fn add_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    new_member: web::Json<NewTeamMember>,
) -> Result<HttpResponse, ApiError> {
    let team_id = info.into_inner().0.into();

    let mut transaction = pool
        .begin()
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    let current_user = get_user_from_headers(req.headers(), &**pool).await?;
    let team_member =
        TeamMember::get_from_user_id(team_id, current_user.id.into(), &**pool).await?;

    let member = match team_member {
        Some(m) => m,
        None => {
            return Err(ApiError::CustomAuthenticationError(
                "You don't have permission to invite users to this team".to_string(),
            ))
        }
    };

    if !member.permissions.contains(Permissions::MANAGE_INVITES) {
        return Err(ApiError::CustomAuthenticationError(
            "You don't have permission to invite users to this team".to_string(),
        ));
    }
    if !member.permissions.contains(new_member.permissions) {
        return Err(ApiError::InvalidInputError(
            "The new member has permissions that you don't have".to_string(),
        ));
    }

    if new_member.role == crate::models::teams::OWNER_ROLE {
        return Err(ApiError::InvalidInputError(
            "The `Owner` role is restricted to one person".to_string(),
        ));
    }
    let request = crate::database::models::team_item::TeamMember::get_from_user_id_pending(
        team_id,
        member.user_id,
        &**pool,
    )
    .await?;

    if let Some(req) = request {
        if req.accepted {
            return Err(ApiError::InvalidInputError(
                "The user is already a member of that team".to_string(),
            ));
        } else {
            return Err(ApiError::InvalidInputError(
                "There is already a pending member request for this user".to_string(),
            ));
        }
    }

    crate::database::models::User::get(member.user_id, &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?
        .ok_or_else(|| ApiError::InvalidInputError("An invalid User ID specified".to_string()))?;

    let new_id = crate::database::models::ids::generate_team_member_id(&mut transaction).await?;
    TeamMember {
        id: new_id,
        team_id,
        user_id: new_member.user_id.into(),
        role: new_member.role.clone(),
        permissions: new_member.permissions,
        accepted: false,
    }
    .insert(&mut transaction)
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?;

    transaction
        .commit()
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    Ok(HttpResponse::Ok().body(""))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditTeamMember {
    pub permissions: Option<Permissions>,
    pub role: Option<String>,
}

#[patch("{id}/members/{user_id}")]
pub async fn edit_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId, UserId)>,
    pool: web::Data<PgPool>,
    edit_member: web::Json<EditTeamMember>,
) -> Result<HttpResponse, ApiError> {
    let ids = info.into_inner();
    let id = ids.0.into();
    let user_id = ids.1.into();

    let current_user = get_user_from_headers(req.headers(), &**pool).await?;
    let team_member = TeamMember::get_from_user_id(id, current_user.id.into(), &**pool).await?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    let member = match team_member {
        Some(m) => m,
        None => {
            return Err(ApiError::CustomAuthenticationError(
                "You don't have permission to edit members of this team".to_string(),
            ))
        }
    };

    if !member.permissions.contains(Permissions::EDIT_MEMBER) {
        return Err(ApiError::CustomAuthenticationError(
            "You don't have permission to edit members of this team".to_string(),
        ));
    }

    if let Some(new_permissions) = edit_member.permissions {
        if !member.permissions.contains(new_permissions) {
            return Err(ApiError::InvalidInputError(
                "The new permissions have permissions that you don't have".to_string(),
            ));
        }
    }

    if edit_member.role.as_deref() == Some(crate::models::teams::OWNER_ROLE) {
        return Err(ApiError::InvalidInputError(
            "The `Owner` role is restricted to one person".to_string(),
        ));
    }

    TeamMember::edit_team_member(
        id,
        user_id,
        edit_member.permissions,
        edit_member.role.clone(),
        None,
        &mut transaction,
    )
    .await?;

    transaction
        .commit()
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    Ok(HttpResponse::Ok().body(""))
}

#[delete("{id}/members/{user_id}")]
pub async fn remove_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId, UserId)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let ids = info.into_inner();
    let id = ids.0.into();
    let user_id = ids.1.into();

    let current_user = get_user_from_headers(req.headers(), &**pool).await?;
    let team_member = TeamMember::get_from_user_id(id, current_user.id.into(), &**pool).await?;

    let member = match team_member {
        Some(m) => m,
        None => {
            return Err(ApiError::CustomAuthenticationError(
                "You don't have permission to remove members from this team".to_string(),
            ))
        }
    };

    let delete_member = TeamMember::get_from_user_id(id, user_id, &**pool).await?;

    if let Some(delete_member) = delete_member {
        if delete_member.role == crate::models::teams::OWNER_ROLE {
            // The owner cannot be removed from a team
            return Err(ApiError::CustomAuthenticationError(
                "The owner can't be removed from a team".to_string(),
            ));
        }

        if delete_member.accepted {
            // Members other than the owner can either leave the team, or be
            // removed by a member with the REMOVE_MEMBER permission.
            if delete_member.user_id == member.user_id
                || member.permissions.contains(Permissions::REMOVE_MEMBER)
            {
                TeamMember::delete(id, user_id, &**pool).await?;
            } else {
                return Err(ApiError::CustomAuthenticationError(
                    "You do not have permission to remove a member from this team".to_string(),
                ));
            }
        } else if delete_member.user_id == member.user_id
            || member.permissions.contains(Permissions::MANAGE_INVITES)
        {
            // This is a pending invite rather than a member, so the
            // user being invited or team members with the MANAGE_INVITES
            // permission can remove it.
            TeamMember::delete(id, user_id, &**pool).await?;
        } else {
            return Err(ApiError::CustomAuthenticationError(
                "You do not have permission to cancel a team invite".to_string(),
            ));
        }
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
