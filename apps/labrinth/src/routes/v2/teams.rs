use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::models::ids::TeamId;
use crate::models::teams::{
    OrganizationPermissions, ProjectPermissions, TeamMember,
};
use crate::models::v2::teams::LegacyTeamMember;
use crate::queue::session::AuthQueue;
use crate::routes::{ApiError, v2_reroute, v3};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use ariadne::ids::UserId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(teams_get);
    cfg.service(
        utoipa_actix_web::scope("/team")
            .service(team_members_get)
            .service(edit_team_member)
            .service(transfer_ownership)
            .service(add_team_member)
            .service(join_team)
            .service(remove_team_member),
    );
}

// Returns all members of a project,
// including the team members of the project's team, but
// also the members of the organization's team if the project is associated with an organization
// (Unlike team_members_get_project, which only returns the members of the project's team)
// They can be differentiated by the "organization_permissions" field being null or not
/// Get a project's team members.
#[utoipa::path(
    get,
    operation_id = "getProjectTeamMembers",
    params(("id" = String, Path, description = "The ID or slug of the project")),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/{id}/members")]
pub async fn team_members_get_project(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::teams::team_members_get_project_internal(
        req,
        info,
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;
    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<TeamMember>>(response).await {
        Ok(members) => {
            let members = members
                .into_iter()
                .map(LegacyTeamMember::from)
                .collect::<Vec<_>>();
            Ok(HttpResponse::Ok().json(members))
        }
        Err(response) => Ok(response),
    }
}

// Returns all members of a team, but not necessarily those of a project-team's organization (unlike team_members_get_project)
/// Get a team's members.
#[utoipa::path(
    get,
    operation_id = "getTeamMembers",
    params(("id" = TeamId, Path, description = "The ID of the team")),
    responses((status = 200, description = "Expected response to a valid request")),
    security(("bearer_auth" = ["PROJECT_READ"]))
)]
#[get("/{id}/members")]
pub async fn team_members_get(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response =
        v3::teams::team_members_get(req, info, pool, redis, session_queue)
            .await
            .or_else(v2_reroute::flatten_404_error)?;
    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<TeamMember>>(response).await {
        Ok(members) => {
            let members = members
                .into_iter()
                .map(LegacyTeamMember::from)
                .collect::<Vec<_>>();
            Ok(HttpResponse::Ok().json(members))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TeamIds {
    pub ids: String,
}

/// Get the members of multiple teams.
#[utoipa::path(
    get,
    operation_id = "getTeams",
    params(("ids" = String, Query, description = "The JSON array of team IDs")),
    responses((status = 200, description = "Expected response to a valid request"))
)]
#[get("/teams")]
pub async fn teams_get(
    req: HttpRequest,
    web::Query(ids): web::Query<TeamIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::teams::teams_get(
        req,
        web::Query(v3::teams::TeamIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error);
    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Vec<TeamMember>>>(response?).await {
        Ok(members) => {
            let members = members
                .into_iter()
                .map(|members| {
                    members
                        .into_iter()
                        .map(LegacyTeamMember::from)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            Ok(HttpResponse::Ok().json(members))
        }
        Err(response) => Ok(response),
    }
}

/// Join a team with a pending invite.
#[utoipa::path(
    post,
    operation_id = "joinTeam",
    params(("id" = TeamId, Path, description = "The ID of the team")),
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
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[post("/{id}/join")]
pub async fn join_team(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert the response
    v3::teams::join_team(req, info, pool, redis, session_queue)
        .await
        .or_else(v2_reroute::flatten_404_error)
}

fn default_role() -> String {
    "Member".to_string()
}

fn default_ordering() -> i64 {
    0
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

/// Add a member to a team.
#[utoipa::path(
    post,
    operation_id = "addTeamMember",
    params(("id" = TeamId, Path, description = "The ID of the team")),
    request_body = NewTeamMember,
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
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[post("/{id}/members")]
pub async fn add_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    new_member: web::Json<NewTeamMember>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert the response
    v3::teams::add_team_member(
        req,
        info,
        pool,
        web::Json(v3::teams::NewTeamMember {
            user_id: new_member.user_id,
            role: new_member.role.clone(),
            permissions: new_member.permissions,
            organization_permissions: new_member.organization_permissions,
            payouts_split: new_member.payouts_split,
            ordering: new_member.ordering,
        }),
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EditTeamMember {
    pub permissions: Option<ProjectPermissions>,
    pub organization_permissions: Option<OrganizationPermissions>,
    pub role: Option<String>,
    pub payouts_split: Option<Decimal>,
    pub ordering: Option<i64>,
}

/// Modify a team member.
#[utoipa::path(
    patch,
    operation_id = "modifyTeamMember",
    params(
        ("id" = TeamId, Path, description = "The ID of the team"),
        (
            "user_id" = UserId,
            Path,
            description = "The ID of the user to modify"
        )
    ),
    request_body = EditTeamMember,
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
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[patch("/{id}/members/{user_id}")]
pub async fn edit_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId, String)>,
    pool: web::Data<PgPool>,
    edit_member: web::Json<EditTeamMember>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert the response
    v3::teams::edit_team_member(
        req,
        info,
        pool,
        web::Json(v3::teams::EditTeamMember {
            permissions: edit_member.permissions,
            organization_permissions: edit_member.organization_permissions,
            role: edit_member.role.clone(),
            payouts_split: edit_member.payouts_split,
            ordering: edit_member.ordering,
        }),
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TransferOwnership {
    pub user_id: UserId,
}

/// Transfer team ownership.
#[utoipa::path(
    patch,
    operation_id = "transferTeamOwnership",
    params(("id" = TeamId, Path, description = "The ID of the team")),
    request_body = TransferOwnership,
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
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[patch("/{id}/owner")]
pub async fn transfer_ownership(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    new_owner: web::Json<TransferOwnership>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert the response
    v3::teams::transfer_ownership(
        req,
        info,
        pool,
        web::Json(v3::teams::TransferOwnership {
            user_id: new_owner.user_id,
        }),
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Remove a member from a team.
#[utoipa::path(
    delete,
    operation_id = "deleteTeamMember",
    params(
        ("id" = TeamId, Path, description = "The ID of the team"),
        (
            "user_id" = UserId,
            Path,
            description = "The ID of the user to remove"
        )
    ),
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
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[delete("/{id}/members/{user_id}")]
pub async fn remove_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId, UserId)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert the response
    v3::teams::remove_team_member(req, info, pool, redis, session_queue)
        .await
        .or_else(v2_reroute::flatten_404_error)
}
