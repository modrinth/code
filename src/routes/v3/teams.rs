use crate::auth::{get_user_from_headers, is_authorized};
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::team_item::TeamAssociationId;
use crate::database::models::{Organization, Team, TeamMember, User};
use crate::database::redis::RedisPool;
use crate::database::Project;
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::teams::{OrganizationPermissions, ProjectPermissions, TeamId};
use crate::models::users::UserId;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{web, HttpRequest, HttpResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("teams", web::get().to(teams_get));

    cfg.service(
        web::scope("team")
            .route("{id}/members", web::get().to(team_members_get))
            .route("{id}/members/{user_id}", web::patch().to(edit_team_member))
            .route(
                "{id}/members/{user_id}",
                web::delete().to(remove_team_member),
            )
            .route("{id}/members", web::post().to(add_team_member))
            .route("{id}/join", web::post().to(join_team))
            .route("{id}/owner", web::patch().to(transfer_ownership)),
    );
}

// Returns all members of a project,
// including the team members of the project's team, but
// also the members of the organization's team if the project is associated with an organization
// (Unlike team_members_get_project, which only returns the members of the project's team)
pub async fn team_members_get_project(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;
    let project_data = crate::database::models::Project::get(&string, &**pool, &redis).await?;

    if let Some(project) = project_data {
        let current_user = get_user_from_headers(
            &req,
            &**pool,
            &redis,
            &session_queue,
            Some(&[Scopes::PROJECT_READ]),
        )
        .await
        .map(|x| x.1)
        .ok();

        if !is_authorized(&project.inner, &current_user, &pool).await? {
            return Ok(HttpResponse::NotFound().body(""));
        }
        let mut members_data =
            TeamMember::get_from_team_full(project.inner.team_id, &**pool, &redis).await?;
        let mut member_user_ids = members_data.iter().map(|x| x.user_id).collect::<Vec<_>>();

        // Adds the organization's team members to the list of members, if the project is associated with an organization
        if let Some(oid) = project.inner.organization_id {
            let organization_data = Organization::get_id(oid, &**pool, &redis).await?;
            if let Some(organization_data) = organization_data {
                let org_team =
                    TeamMember::get_from_team_full(organization_data.team_id, &**pool, &redis)
                        .await?;
                for member in org_team {
                    if !member_user_ids.contains(&member.user_id) {
                        member_user_ids.push(member.user_id);
                        members_data.push(member);
                    }
                }
            }
        }

        let users =
            crate::database::models::User::get_many_ids(&member_user_ids, &**pool, &redis).await?;

        let user_id = current_user.as_ref().map(|x| x.id.into());

        let logged_in = current_user
            .and_then(|user| {
                members_data
                    .iter()
                    .find(|x| x.user_id == user.id.into() && x.accepted)
            })
            .is_some();
        let team_members: Vec<_> = members_data
            .into_iter()
            .filter(|x| {
                logged_in
                    || x.accepted
                    || user_id
                        .map(|y: crate::database::models::UserId| y == x.user_id)
                        .unwrap_or(false)
            })
            .flat_map(|data| {
                users.iter().find(|x| x.id == data.user_id).map(|user| {
                    crate::models::teams::TeamMember::from(data, user.clone(), !logged_in)
                })
            })
            .collect();
        Ok(HttpResponse::Ok().json(team_members))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

pub async fn team_members_get_organization(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;
    let organization_data =
        crate::database::models::Organization::get(&string, &**pool, &redis).await?;

    if let Some(organization) = organization_data {
        let current_user = get_user_from_headers(
            &req,
            &**pool,
            &redis,
            &session_queue,
            Some(&[Scopes::ORGANIZATION_READ]),
        )
        .await
        .map(|x| x.1)
        .ok();

        let members_data =
            TeamMember::get_from_team_full(organization.team_id, &**pool, &redis).await?;
        let users = crate::database::models::User::get_many_ids(
            &members_data.iter().map(|x| x.user_id).collect::<Vec<_>>(),
            &**pool,
            &redis,
        )
        .await?;

        let user_id = current_user.as_ref().map(|x| x.id.into());

        let logged_in = current_user
            .and_then(|user| {
                members_data
                    .iter()
                    .find(|x| x.user_id == user.id.into() && x.accepted)
            })
            .is_some();

        let team_members: Vec<_> = members_data
            .into_iter()
            .filter(|x| {
                logged_in
                    || x.accepted
                    || user_id
                        .map(|y: crate::database::models::UserId| y == x.user_id)
                        .unwrap_or(false)
            })
            .flat_map(|data| {
                users.iter().find(|x| x.id == data.user_id).map(|user| {
                    crate::models::teams::TeamMember::from(data, user.clone(), !logged_in)
                })
            })
            .collect();

        Ok(HttpResponse::Ok().json(team_members))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

// Returns all members of a team, but not necessarily those of a project-team's organization (unlike team_members_get_project)
pub async fn team_members_get(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let members_data = TeamMember::get_from_team_full(id.into(), &**pool, &redis).await?;
    let users = crate::database::models::User::get_many_ids(
        &members_data.iter().map(|x| x.user_id).collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await?;

    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();
    let user_id = current_user.as_ref().map(|x| x.id.into());

    let logged_in = current_user
        .and_then(|user| {
            members_data
                .iter()
                .find(|x| x.user_id == user.id.into() && x.accepted)
        })
        .is_some();

    let team_members: Vec<_> = members_data
        .into_iter()
        .filter(|x| {
            logged_in
                || x.accepted
                || user_id
                    .map(|y: crate::database::models::UserId| y == x.user_id)
                    .unwrap_or(false)
        })
        .flat_map(|data| {
            users
                .iter()
                .find(|x| x.id == data.user_id)
                .map(|user| crate::models::teams::TeamMember::from(data, user.clone(), !logged_in))
        })
        .collect();

    Ok(HttpResponse::Ok().json(team_members))
}

#[derive(Serialize, Deserialize)]
pub struct TeamIds {
    pub ids: String,
}

pub async fn teams_get(
    req: HttpRequest,
    web::Query(ids): web::Query<TeamIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    use itertools::Itertools;

    let team_ids = serde_json::from_str::<Vec<TeamId>>(&ids.ids)?
        .into_iter()
        .map(|x| x.into())
        .collect::<Vec<crate::database::models::ids::TeamId>>();

    let teams_data = TeamMember::get_from_team_full_many(&team_ids, &**pool, &redis).await?;
    let users = crate::database::models::User::get_many_ids(
        &teams_data.iter().map(|x| x.user_id).collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await?;

    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let teams_groups = teams_data.into_iter().group_by(|data| data.team_id.0);

    let mut teams: Vec<Vec<crate::models::teams::TeamMember>> = vec![];

    for (_, member_data) in &teams_groups {
        let members = member_data.collect::<Vec<_>>();

        let logged_in = current_user
            .as_ref()
            .and_then(|user| {
                members
                    .iter()
                    .find(|x| x.user_id == user.id.into() && x.accepted)
            })
            .is_some();

        let team_members = members
            .into_iter()
            .filter(|x| logged_in || x.accepted)
            .flat_map(|data| {
                users.iter().find(|x| x.id == data.user_id).map(|user| {
                    crate::models::teams::TeamMember::from(data, user.clone(), !logged_in)
                })
            });

        teams.push(team_members.collect());
    }

    Ok(HttpResponse::Ok().json(teams))
}

pub async fn join_team(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let team_id = info.into_inner().0.into();
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;

    let member =
        TeamMember::get_from_user_id_pending(team_id, current_user.id.into(), &**pool).await?;

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
            None,
            Some(true),
            None,
            None,
            &mut transaction,
        )
        .await?;

        User::clear_project_cache(&[current_user.id.into()], &redis).await?;
        TeamMember::clear_cache(team_id, &redis).await?;

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
    #[serde(default)]
    pub permissions: ProjectPermissions,
    #[serde(default)]
    pub organization_permissions: Option<OrganizationPermissions>,
    #[serde(default)]
    #[serde(with = "rust_decimal::serde::float")]
    pub payouts_split: Decimal,
    #[serde(default = "default_ordering")]
    pub ordering: i64,
}

pub async fn add_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    new_member: web::Json<NewTeamMember>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let team_id = info.into_inner().0.into();

    let mut transaction = pool.begin().await?;

    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;
    let team_association = Team::get_association(team_id, &**pool)
        .await?
        .ok_or_else(|| ApiError::InvalidInput("The team specified does not exist".to_string()))?;
    let member = TeamMember::get_from_user_id(team_id, current_user.id.into(), &**pool).await?;
    match team_association {
        // If team is associated with a project, check if they have permissions to invite users to that project
        TeamAssociationId::Project(pid) => {
            let organization =
                Organization::get_associated_organization_project_id(pid, &**pool).await?;
            let organization_team_member = if let Some(organization) = &organization {
                TeamMember::get_from_user_id(organization.team_id, current_user.id.into(), &**pool)
                    .await?
            } else {
                None
            };
            let permissions = ProjectPermissions::get_permissions_by_role(
                &current_user.role,
                &member,
                &organization_team_member,
            )
            .unwrap_or_default();

            if !permissions.contains(ProjectPermissions::MANAGE_INVITES) {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to invite users to this team".to_string(),
                ));
            }
            if !permissions.contains(new_member.permissions) {
                return Err(ApiError::InvalidInput(
                    "The new member has permissions that you don't have".to_string(),
                ));
            }

            if new_member.organization_permissions.is_some() {
                return Err(ApiError::InvalidInput(
                    "The organization permissions of a project team member cannot be set"
                        .to_string(),
                ));
            }
        }
        // If team is associated with an organization, check if they have permissions to invite users to that organization
        TeamAssociationId::Organization(_) => {
            let organization_permissions =
                OrganizationPermissions::get_permissions_by_role(&current_user.role, &member)
                    .unwrap_or_default();
            if !organization_permissions.contains(OrganizationPermissions::MANAGE_INVITES) {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to invite users to this organization".to_string(),
                ));
            }
            if !organization_permissions
                .contains(new_member.organization_permissions.unwrap_or_default())
            {
                return Err(ApiError::InvalidInput(
                    "The new member has organization permissions that you don't have".to_string(),
                ));
            }
            if !organization_permissions
                .contains(OrganizationPermissions::EDIT_MEMBER_DEFAULT_PERMISSIONS)
                && !new_member.permissions.is_empty()
            {
                return Err(ApiError::CustomAuthentication(
                    "You do not have permission to give this user default project permissions. Ensure 'permissions' is set if it is not, and empty (0)."
                        .to_string(),
                ));
            }
        }
    }

    if new_member.role == crate::models::teams::OWNER_ROLE {
        return Err(ApiError::InvalidInput(
            "The `Owner` role is restricted to one person".to_string(),
        ));
    }

    if new_member.payouts_split < Decimal::ZERO || new_member.payouts_split > Decimal::from(5000) {
        return Err(ApiError::InvalidInput(
            "Payouts split must be between 0 and 5000!".to_string(),
        ));
    }

    let request =
        TeamMember::get_from_user_id_pending(team_id, new_member.user_id.into(), &**pool).await?;

    if let Some(req) = request {
        if req.accepted {
            return Err(ApiError::InvalidInput(
                "The user is already a member of that team".to_string(),
            ));
        } else {
            return Err(ApiError::InvalidInput(
                "There is already a pending member request for this user".to_string(),
            ));
        }
    }
    crate::database::models::User::get_id(new_member.user_id.into(), &**pool, &redis)
        .await?
        .ok_or_else(|| ApiError::InvalidInput("An invalid User ID specified".to_string()))?;

    let new_id = crate::database::models::ids::generate_team_member_id(&mut transaction).await?;
    TeamMember {
        id: new_id,
        team_id,
        user_id: new_member.user_id.into(),
        role: new_member.role.clone(),
        permissions: new_member.permissions,
        organization_permissions: new_member.organization_permissions,
        accepted: false,
        payouts_split: new_member.payouts_split,
        ordering: new_member.ordering,
    }
    .insert(&mut transaction)
    .await?;

    match team_association {
        TeamAssociationId::Project(pid) => {
            NotificationBuilder {
                body: NotificationBody::TeamInvite {
                    project_id: pid.into(),
                    team_id: team_id.into(),
                    invited_by: current_user.id,
                    role: new_member.role.clone(),
                },
            }
            .insert(new_member.user_id.into(), &mut transaction, &redis)
            .await?;
        }
        TeamAssociationId::Organization(oid) => {
            NotificationBuilder {
                body: NotificationBody::OrganizationInvite {
                    organization_id: oid.into(),
                    team_id: team_id.into(),
                    invited_by: current_user.id,
                    role: new_member.role.clone(),
                },
            }
            .insert(new_member.user_id.into(), &mut transaction, &redis)
            .await?;
        }
    }

    TeamMember::clear_cache(team_id, &redis).await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditTeamMember {
    pub permissions: Option<ProjectPermissions>,
    pub organization_permissions: Option<OrganizationPermissions>,
    pub role: Option<String>,
    pub payouts_split: Option<Decimal>,
    pub ordering: Option<i64>,
}

pub async fn edit_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId, UserId)>,
    pool: web::Data<PgPool>,
    edit_member: web::Json<EditTeamMember>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids = info.into_inner();
    let id = ids.0.into();
    let user_id = ids.1.into();

    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;

    let team_association = Team::get_association(id, &**pool)
        .await?
        .ok_or_else(|| ApiError::InvalidInput("The team specified does not exist".to_string()))?;
    let member = TeamMember::get_from_user_id(id, current_user.id.into(), &**pool).await?;
    let edit_member_db = TeamMember::get_from_user_id_pending(id, user_id, &**pool)
        .await?
        .ok_or_else(|| {
            ApiError::CustomAuthentication(
                "You don't have permission to edit members of this team".to_string(),
            )
        })?;

    let mut transaction = pool.begin().await?;

    if &*edit_member_db.role == crate::models::teams::OWNER_ROLE
        && (edit_member.role.is_some() || edit_member.permissions.is_some())
    {
        return Err(ApiError::InvalidInput(
            "The owner's permission and role of a team cannot be edited".to_string(),
        ));
    }

    match team_association {
        TeamAssociationId::Project(project_id) => {
            let organization =
                Organization::get_associated_organization_project_id(project_id, &**pool).await?;
            let organization_team_member = if let Some(organization) = &organization {
                TeamMember::get_from_user_id(organization.team_id, current_user.id.into(), &**pool)
                    .await?
            } else {
                None
            };
            let permissions = ProjectPermissions::get_permissions_by_role(
                &current_user.role,
                &member.clone(),
                &organization_team_member,
            )
            .unwrap_or_default();
            if !permissions.contains(ProjectPermissions::EDIT_MEMBER) {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to edit members of this team".to_string(),
                ));
            }

            if let Some(new_permissions) = edit_member.permissions {
                if !permissions.contains(new_permissions) {
                    return Err(ApiError::InvalidInput(
                        "The new permissions have permissions that you don't have".to_string(),
                    ));
                }
            }

            if edit_member.organization_permissions.is_some() {
                return Err(ApiError::InvalidInput(
                    "The organization permissions of a project team member cannot be edited"
                        .to_string(),
                ));
            }
        }
        TeamAssociationId::Organization(_) => {
            let organization_permissions =
                OrganizationPermissions::get_permissions_by_role(&current_user.role, &member)
                    .unwrap_or_default();

            if !organization_permissions.contains(OrganizationPermissions::EDIT_MEMBER) {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to edit members of this team".to_string(),
                ));
            }

            if let Some(new_permissions) = edit_member.organization_permissions {
                if !organization_permissions.contains(new_permissions) {
                    return Err(ApiError::InvalidInput(
                        "The new organization permissions have permissions that you don't have"
                            .to_string(),
                    ));
                }
            }

            if edit_member.permissions.is_some()
                && !organization_permissions
                    .contains(OrganizationPermissions::EDIT_MEMBER_DEFAULT_PERMISSIONS)
            {
                return Err(ApiError::CustomAuthentication(
                    "You do not have permission to give this user default project permissions."
                        .to_string(),
                ));
            }
        }
    }

    if let Some(payouts_split) = edit_member.payouts_split {
        if payouts_split < Decimal::ZERO || payouts_split > Decimal::from(5000) {
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
        edit_member.organization_permissions,
        edit_member.role.clone(),
        None,
        edit_member.payouts_split,
        edit_member.ordering,
        &mut transaction,
    )
    .await?;

    TeamMember::clear_cache(id, &redis).await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Deserialize)]
pub struct TransferOwnership {
    pub user_id: UserId,
}

pub async fn transfer_ownership(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    new_owner: web::Json<TransferOwnership>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;

    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;

    // Forbid transferring ownership of a project team that is owned by an organization
    // These are owned by the organization owner, and must be removed from the organization first
    let pid = Team::get_association(id.into(), &**pool).await?;
    if let Some(TeamAssociationId::Project(pid)) = pid {
        let result = Project::get_id(pid, &**pool, &redis).await?;
        if let Some(project_item) = result {
            if project_item.inner.organization_id.is_some() {
                return Err(ApiError::InvalidInput(
                    "You cannot transfer ownership of a project team that is owend by an organization"
                        .to_string(),
                ));
            }
        }
    }

    if !current_user.role.is_admin() {
        let member = TeamMember::get_from_user_id(id.into(), current_user.id.into(), &**pool)
            .await?
            .ok_or_else(|| {
                ApiError::CustomAuthentication(
                    "You don't have permission to edit members of this team".to_string(),
                )
            })?;

        if member.role != crate::models::teams::OWNER_ROLE {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit the ownership of this team".to_string(),
            ));
        }
    }

    let new_member = TeamMember::get_from_user_id(id.into(), new_owner.user_id.into(), &**pool)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("The new owner specified does not exist".to_string())
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
        Some(ProjectPermissions::all()),
        Some(OrganizationPermissions::all()),
        Some(crate::models::teams::OWNER_ROLE.to_string()),
        None,
        None,
        None,
        &mut transaction,
    )
    .await?;

    TeamMember::clear_cache(id.into(), &redis).await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

pub async fn remove_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId, UserId)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids = info.into_inner();
    let id = ids.0.into();
    let user_id = ids.1.into();

    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_WRITE]),
    )
    .await?
    .1;

    let team_association = Team::get_association(id, &**pool)
        .await?
        .ok_or_else(|| ApiError::InvalidInput("The team specified does not exist".to_string()))?;
    let member = TeamMember::get_from_user_id(id, current_user.id.into(), &**pool).await?;

    let delete_member = TeamMember::get_from_user_id_pending(id, user_id, &**pool).await?;

    if let Some(delete_member) = delete_member {
        if delete_member.role == crate::models::teams::OWNER_ROLE {
            // The owner cannot be removed from a team
            return Err(ApiError::CustomAuthentication(
                "The owner can't be removed from a team".to_string(),
            ));
        }

        let mut transaction = pool.begin().await?;

        // Organization attached to a project this team is attached to
        match team_association {
            TeamAssociationId::Project(pid) => {
                let organization =
                    Organization::get_associated_organization_project_id(pid, &**pool).await?;
                let organization_team_member = if let Some(organization) = &organization {
                    TeamMember::get_from_user_id(
                        organization.team_id,
                        current_user.id.into(),
                        &**pool,
                    )
                    .await?
                } else {
                    None
                };
                let permissions = ProjectPermissions::get_permissions_by_role(
                    &current_user.role,
                    &member,
                    &organization_team_member,
                )
                .unwrap_or_default();

                if delete_member.accepted {
                    // Members other than the owner can either leave the team, or be
                    // removed by a member with the REMOVE_MEMBER permission.
                    if Some(delete_member.user_id) == member.as_ref().map(|m| m.user_id)
                        || permissions.contains(ProjectPermissions::REMOVE_MEMBER)
                    // true as if the permission exists, but the member does not, they are part of an org
                    {
                        TeamMember::delete(id, user_id, &mut transaction).await?;
                    } else {
                        return Err(ApiError::CustomAuthentication(
                            "You do not have permission to remove a member from this team"
                                .to_string(),
                        ));
                    }
                } else if Some(delete_member.user_id) == member.as_ref().map(|m| m.user_id)
                    || permissions.contains(ProjectPermissions::MANAGE_INVITES)
                // true as if the permission exists, but the member does not, they are part of an org
                {
                    // This is a pending invite rather than a member, so the
                    // user being invited or team members with the MANAGE_INVITES
                    // permission can remove it.
                    TeamMember::delete(id, user_id, &mut transaction).await?;
                } else {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have permission to cancel a team invite".to_string(),
                    ));
                }
            }
            TeamAssociationId::Organization(_) => {
                let organization_permissions =
                    OrganizationPermissions::get_permissions_by_role(&current_user.role, &member)
                        .unwrap_or_default();
                // Organization teams requires a TeamMember, so we can 'unwrap'
                if delete_member.accepted {
                    // Members other than the owner can either leave the team, or be
                    // removed by a member with the REMOVE_MEMBER permission.
                    if Some(delete_member.user_id) == member.map(|m| m.user_id)
                        || organization_permissions.contains(OrganizationPermissions::REMOVE_MEMBER)
                    {
                        TeamMember::delete(id, user_id, &mut transaction).await?;
                    } else {
                        return Err(ApiError::CustomAuthentication(
                            "You do not have permission to remove a member from this organization"
                                .to_string(),
                        ));
                    }
                } else if Some(delete_member.user_id) == member.map(|m| m.user_id)
                    || organization_permissions.contains(OrganizationPermissions::MANAGE_INVITES)
                {
                    // This is a pending invite rather than a member, so the
                    // user being invited or team members with the MANAGE_INVITES
                    // permission can remove it.
                    TeamMember::delete(id, user_id, &mut transaction).await?;
                } else {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have permission to cancel an organization invite".to_string(),
                    ));
                }
            }
        }

        TeamMember::clear_cache(id, &redis).await?;
        User::clear_project_cache(&[delete_member.user_id], &redis).await?;

        transaction.commit().await?;
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
