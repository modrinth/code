use crate::database::models::notification_item::{
    NotificationActionBuilder, NotificationBuilder,
};
use crate::database::models::TeamMember;
use crate::models::ids::ProjectId;
use crate::models::teams::{Permissions, TeamId};
use crate::models::users::UserId;
use crate::routes::ApiError;
use crate::util::auth::get_user_from_headers;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[get("{id}/members")]
pub async fn team_members_get_project(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;
    let project_data =
        crate::database::models::Project::get_from_slug_or_project_id(
            &string, &**pool,
        )
        .await?;

    if let Some(project) = project_data {
        let members_data =
            TeamMember::get_from_team_full(project.team_id, &**pool).await?;

        let current_user =
            get_user_from_headers(req.headers(), &**pool).await.ok();

        if let Some(user) = current_user {
            let team_member = TeamMember::get_from_user_id(
                project.team_id,
                user.id.into(),
                &**pool,
            )
            .await
            .map_err(ApiError::Database)?;

            if team_member.is_some() {
                let team_members: Vec<_> = members_data
                    .into_iter()
                    .map(|data| {
                        crate::models::teams::TeamMember::from(data, false)
                    })
                    .collect();

                return Ok(HttpResponse::Ok().json(team_members));
            }
        }

        let team_members: Vec<_> = members_data
            .into_iter()
            .filter(|x| x.accepted)
            .map(|data| crate::models::teams::TeamMember::from(data, true))
            .collect();

        Ok(HttpResponse::Ok().json(team_members))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("{id}/members")]
pub async fn team_members_get(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let members_data =
        TeamMember::get_from_team_full(id.into(), &**pool).await?;

    let current_user = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(user) = current_user {
        let team_member =
            TeamMember::get_from_user_id(id.into(), user.id.into(), &**pool)
                .await
                .map_err(ApiError::Database)?;

        if team_member.is_some() {
            let team_members: Vec<_> = members_data
                .into_iter()
                .map(|data| crate::models::teams::TeamMember::from(data, false))
                .collect();

            return Ok(HttpResponse::Ok().json(team_members));
        }
    }

    let team_members: Vec<_> = members_data
        .into_iter()
        .filter(|x| x.accepted)
        .map(|data| crate::models::teams::TeamMember::from(data, true))
        .collect();

    Ok(HttpResponse::Ok().json(team_members))
}

#[derive(Serialize, Deserialize)]
pub struct TeamIds {
    pub ids: String,
}

#[get("teams")]
pub async fn teams_get(
    req: HttpRequest,
    web::Query(ids): web::Query<TeamIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    use itertools::Itertools;

    let team_ids = serde_json::from_str::<Vec<TeamId>>(&ids.ids)?
        .into_iter()
        .map(|x| x.into())
        .collect::<Vec<crate::database::models::ids::TeamId>>();

    let teams_data =
        TeamMember::get_from_team_full_many(team_ids.clone(), &**pool).await?;

    let current_user = get_user_from_headers(req.headers(), &**pool).await.ok();
    let accepted = if let Some(user) = current_user {
        TeamMember::get_from_user_id_many(team_ids, user.id.into(), &**pool)
            .await?
            .into_iter()
            .map(|m| m.team_id.0)
            .collect()
    } else {
        std::collections::HashSet::new()
    };

    let teams_groups = teams_data.into_iter().group_by(|data| data.team_id.0);

    let mut teams: Vec<Vec<crate::models::teams::TeamMember>> = vec![];

    for (id, member_data) in &teams_groups {
        if accepted.contains(&id) {
            let team_members = member_data.map(|data| {
                crate::models::teams::TeamMember::from(data, false)
            });

            teams.push(team_members.collect());

            continue;
        }

        let team_members = member_data
            .filter(|x| x.accepted)
            .map(|data| crate::models::teams::TeamMember::from(data, true));

        teams.push(team_members.collect());
    }

    Ok(HttpResponse::Ok().json(teams))
}

#[post("{id}/join")]
pub async fn join_team(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let team_id = info.into_inner().0.into();
    let current_user = get_user_from_headers(req.headers(), &**pool).await?;

    let member = TeamMember::get_from_user_id_pending(
        team_id,
        current_user.id.into(),
        &**pool,
    )
    .await?;

    if let Some(member) = member {
        if member.accepted {
            return Err(ApiError::InvalidInput(
                "You are already a member of this team".to_string(),
            ));
        }
        let mut transaction = pool.begin().await?;

        // Edit Team Member to set Accepted to True
        TeamMember::edit_team_member(
            team_id,
            current_user.id.into(),
            None,
            None,
            Some(true),
            None,
            None,
            &mut transaction,
        )
        .await?;

        transaction.commit().await?;
    } else {
        return Err(ApiError::InvalidInput(
            "There is no pending request from this team".to_string(),
        ));
    }

    Ok(HttpResponse::NoContent().body(""))
}

fn default_role() -> String {
    "Member".to_string()
}

fn default_ordering() -> i64 {
    0
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewTeamMember {
    pub user_id: UserId,
    #[serde(default = "default_role")]
    pub role: String,
    #[serde(default = "Permissions::default")]
    pub permissions: Permissions,
    #[serde(default)]
    pub payouts_split: Decimal,
    #[serde(default = "default_ordering")]
    pub ordering: i64,
}

#[post("{id}/members")]
pub async fn add_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    new_member: web::Json<NewTeamMember>,
) -> Result<HttpResponse, ApiError> {
    let team_id = info.into_inner().0.into();

    let mut transaction = pool.begin().await?;

    let current_user = get_user_from_headers(req.headers(), &**pool).await?;
    let member =
        TeamMember::get_from_user_id(team_id, current_user.id.into(), &**pool)
            .await?
            .ok_or_else(|| {
                ApiError::CustomAuthentication(
                    "You don't have permission to edit members of this team"
                        .to_string(),
                )
            })?;

    if !member.permissions.contains(Permissions::MANAGE_INVITES) {
        return Err(ApiError::CustomAuthentication(
            "You don't have permission to invite users to this team"
                .to_string(),
        ));
    }
    if !member.permissions.contains(new_member.permissions) {
        return Err(ApiError::InvalidInput(
            "The new member has permissions that you don't have".to_string(),
        ));
    }

    if new_member.role == crate::models::teams::OWNER_ROLE {
        return Err(ApiError::InvalidInput(
            "The `Owner` role is restricted to one person".to_string(),
        ));
    }

    if new_member.payouts_split < Decimal::ZERO
        || new_member.payouts_split > Decimal::from(5000)
    {
        return Err(ApiError::InvalidInput(
            "Payouts split must be between 0 and 5000!".to_string(),
        ));
    }

    let request = crate::database::models::team_item::TeamMember::get_from_user_id_pending(
        team_id,
        new_member.user_id.into(),
        &**pool,
    )
    .await?;

    if let Some(req) = request {
        if req.accepted {
            return Err(ApiError::InvalidInput(
                "The user is already a member of that team".to_string(),
            ));
        } else {
            return Err(ApiError::InvalidInput(
                "There is already a pending member request for this user"
                    .to_string(),
            ));
        }
    }

    crate::database::models::User::get(member.user_id, &**pool)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("An invalid User ID specified".to_string())
        })?;

    let new_id =
        crate::database::models::ids::generate_team_member_id(&mut transaction)
            .await?;
    TeamMember {
        id: new_id,
        team_id,
        user_id: new_member.user_id.into(),
        role: new_member.role.clone(),
        permissions: new_member.permissions,
        accepted: false,
        payouts_split: new_member.payouts_split,
        ordering: new_member.ordering,
    }
    .insert(&mut transaction)
    .await?;

    let result = sqlx::query!(
        "
        SELECT m.title title, m.id id, pt.name project_type
        FROM mods m
        INNER JOIN project_types pt ON pt.id = m.project_type
        WHERE m.team_id = $1
        ",
        team_id as crate::database::models::ids::TeamId
    )
    .fetch_one(&**pool)
    .await?;

    let team: TeamId = team_id.into();
    NotificationBuilder {
        notification_type: Some("team_invite".to_string()),
        title: "You have been invited to join a team!".to_string(),
        text: format!(
            "Team invite from {} to join the team for project {}",
            current_user.username, result.title
        ),
        link: format!(
            "/{}/{}",
            result.project_type,
            ProjectId(result.id as u64)
        ),
        actions: vec![
            NotificationActionBuilder {
                title: "Accept".to_string(),
                action_route: ("POST".to_string(), format!("team/{team}/join")),
            },
            NotificationActionBuilder {
                title: "Deny".to_string(),
                action_route: (
                    "DELETE".to_string(),
                    format!("team/{team}/members/{}", new_member.user_id),
                ),
            },
        ],
    }
    .insert(new_member.user_id.into(), &mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditTeamMember {
    pub permissions: Option<Permissions>,
    pub role: Option<String>,
    pub payouts_split: Option<Decimal>,
    pub ordering: Option<i64>,
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
    let member =
        TeamMember::get_from_user_id(id, current_user.id.into(), &**pool)
            .await?
            .ok_or_else(|| {
                ApiError::CustomAuthentication(
                    "You don't have permission to edit members of this team"
                        .to_string(),
                )
            })?;
    let edit_member_db =
        TeamMember::get_from_user_id_pending(id, user_id, &**pool)
            .await?
            .ok_or_else(|| {
                ApiError::CustomAuthentication(
                    "You don't have permission to edit members of this team"
                        .to_string(),
                )
            })?;

    let mut transaction = pool.begin().await?;

    if &*edit_member_db.role == crate::models::teams::OWNER_ROLE
        && (edit_member.role.is_some() || edit_member.permissions.is_some())
    {
        return Err(ApiError::InvalidInput(
            "The owner's permission and role of a team cannot be edited"
                .to_string(),
        ));
    }

    if !member.permissions.contains(Permissions::EDIT_MEMBER) {
        return Err(ApiError::CustomAuthentication(
            "You don't have permission to edit members of this team"
                .to_string(),
        ));
    }

    if let Some(new_permissions) = edit_member.permissions {
        if !member.permissions.contains(new_permissions) {
            return Err(ApiError::InvalidInput(
                "The new permissions have permissions that you don't have"
                    .to_string(),
            ));
        }
    }

    if let Some(payouts_split) = edit_member.payouts_split {
        if payouts_split < Decimal::ZERO || payouts_split > Decimal::from(5000)
        {
            return Err(ApiError::InvalidInput(
                "Payouts split must be between 0 and 5000!".to_string(),
            ));
        }
    }

    if edit_member.role.as_deref() == Some(crate::models::teams::OWNER_ROLE) {
        return Err(ApiError::InvalidInput(
            "The `Owner` role is restricted to one person".to_string(),
        ));
    }

    TeamMember::edit_team_member(
        id,
        user_id,
        edit_member.permissions,
        edit_member.role.clone(),
        None,
        edit_member.payouts_split,
        edit_member.ordering,
        &mut transaction,
    )
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Deserialize)]
pub struct TransferOwnership {
    pub user_id: UserId,
}

#[patch("{id}/owner")]
pub async fn transfer_ownership(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    new_owner: web::Json<TransferOwnership>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;

    let current_user = get_user_from_headers(req.headers(), &**pool).await?;

    if !current_user.role.is_admin() {
        let member = TeamMember::get_from_user_id(
            id.into(),
            current_user.id.into(),
            &**pool,
        )
        .await?
        .ok_or_else(|| {
            ApiError::CustomAuthentication(
                "You don't have permission to edit members of this team"
                    .to_string(),
            )
        })?;

        if member.role != crate::models::teams::OWNER_ROLE {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit the ownership of this team"
                    .to_string(),
            ));
        }
    }

    let new_member = TeamMember::get_from_user_id(
        id.into(),
        new_owner.user_id.into(),
        &**pool,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "The new owner specified does not exist".to_string(),
        )
    })?;

    if !new_member.accepted {
        return Err(ApiError::InvalidInput(
            "You can only transfer ownership to members who are currently in your team".to_string(),
        ));
    }

    let mut transaction = pool.begin().await?;

    TeamMember::edit_team_member(
        id.into(),
        current_user.id.into(),
        None,
        Some(crate::models::teams::DEFAULT_ROLE.to_string()),
        None,
        None,
        None,
        &mut transaction,
    )
    .await?;

    TeamMember::edit_team_member(
        id.into(),
        new_owner.user_id.into(),
        Some(Permissions::ALL),
        Some(crate::models::teams::OWNER_ROLE.to_string()),
        None,
        None,
        None,
        &mut transaction,
    )
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
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
    let member =
        TeamMember::get_from_user_id(id, current_user.id.into(), &**pool)
            .await?
            .ok_or_else(|| {
                ApiError::CustomAuthentication(
                    "You don't have permission to edit members of this team"
                        .to_string(),
                )
            })?;

    let delete_member =
        TeamMember::get_from_user_id_pending(id, user_id, &**pool).await?;

    if let Some(delete_member) = delete_member {
        if delete_member.role == crate::models::teams::OWNER_ROLE {
            // The owner cannot be removed from a team
            return Err(ApiError::CustomAuthentication(
                "The owner can't be removed from a team".to_string(),
            ));
        }

        let mut transaction = pool.begin().await?;

        if delete_member.accepted {
            // Members other than the owner can either leave the team, or be
            // removed by a member with the REMOVE_MEMBER permission.
            if delete_member.user_id == member.user_id
                || (member.permissions.contains(Permissions::REMOVE_MEMBER)
                    && member.accepted)
            {
                TeamMember::delete(id, user_id, &mut transaction).await?;
            } else {
                return Err(ApiError::CustomAuthentication(
                    "You do not have permission to remove a member from this team".to_string(),
                ));
            }
        } else if delete_member.user_id == member.user_id
            || (member.permissions.contains(Permissions::MANAGE_INVITES)
                && member.accepted)
        {
            // This is a pending invite rather than a member, so the
            // user being invited or team members with the MANAGE_INVITES
            // permission can remove it.
            TeamMember::delete(id, user_id, &mut transaction).await?;
        } else {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to cancel a team invite"
                    .to_string(),
            ));
        }

        transaction.commit().await?;
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
