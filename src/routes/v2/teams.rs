use crate::database::redis::RedisPool;
use crate::models::teams::{OrganizationPermissions, ProjectPermissions, TeamId};
use crate::models::users::UserId;
use crate::queue::session::AuthQueue;
use crate::routes::{v3, ApiError};
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(teams_get);

    cfg.service(
        web::scope("team")
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
#[get("{id}/members")]
pub async fn team_members_get_project(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::teams::team_members_get_project(req, info, pool, redis, session_queue).await
}

#[get("{id}/members")]
pub async fn team_members_get_organization(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::teams::team_members_get_organization(req, info, pool, redis, session_queue).await
}

// Returns all members of a team, but not necessarily those of a project-team's organization (unlike team_members_get_project)
#[get("{id}/members")]
pub async fn team_members_get(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::teams::team_members_get(req, info, pool, redis, session_queue).await
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
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::teams::teams_get(
        req,
        web::Query(v3::teams::TeamIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
}

#[post("{id}/join")]
pub async fn join_team(
    req: HttpRequest,
    info: web::Path<(TeamId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::teams::join_team(req, info, pool, redis, session_queue).await
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
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditTeamMember {
    pub permissions: Option<ProjectPermissions>,
    pub organization_permissions: Option<OrganizationPermissions>,
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
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
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
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
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
}

#[delete("{id}/members/{user_id}")]
pub async fn remove_team_member(
    req: HttpRequest,
    info: web::Path<(TeamId, UserId)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::teams::remove_team_member(req, info, pool, redis, session_queue).await
}
