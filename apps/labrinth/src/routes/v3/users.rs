use crate::util::error::ApiContext as _;
use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};
use xredis::RedisPool;

use super::{ApiError, oauth_clients::get_user_clients};
use crate::database::PgPool;
use crate::util::error::Context;
use crate::{
    auth::{
        check_is_moderator_from_headers, checks::is_visible_organization,
        filter_visible_collections, filter_visible_projects,
        get_user_from_headers,
    },
    database::models::{DBModerationNote, DBOrganization, DBProjectId, DBUser},
    file_hosting::{FileHost, FileHostPublicity},
    models::{
        ids::OrganizationId,
        notifications::Notification,
        organizations::Organization,
        pats::Scopes,
        projects::Project,
        users::{Badges, Role},
    },
    queue::session::AuthQueue,
    util::{img::delete_old_images, routes::read_limited_from_payload},
};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, web};
use ariadne::ids::UserId;
use eyre::eyre;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::database::models::user_preferences_item::DBUserPreferences;
use crate::models::v3::preferences::{PartialUserPreferences, UserPreferences};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(user_auth_get_route)
        .service(users_get_route)
        .service(users_search)
        .service(admin_user_email)
        .service(all_projects)
        .service(projects_list_route)
        .service(user_notes_edit)
        .service(user_get_route)
        .service(collections_list)
        .service(orgs_list)
        .service(user_edit_route)
        .service(user_icon_edit_route)
        .service(user_icon_delete_route)
        .service(user_delete_route)
        .service(user_follows_route)
        .service(user_notifications_route)
        .service(get_user_preferences)
        .service(edit_user_preferences)
        .service(get_user_clients);
}

#[derive(Serialize)]
pub struct AllProjectsResponse {
    pub projects: Vec<Project>,
    pub organizations: HashMap<OrganizationId, Organization>,
}

#[derive(Deserialize)]
pub struct UserEmailQuery {
    pub email: String,
}

#[utoipa::path(tag = "users", responses((status = OK)))]
#[get("/user/{user_id}/all-projects")]
pub async fn all_projects(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<AllProjectsResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .map(|x| x.1)
    .ok();
    let target_user = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("resource not found")?;

    let user_project_ids =
        DBUser::get_projects(target_user.id, &**pool, &redis)
            .await
            .wrap_internal_err("fetching users from database")?;
    let organization_ids = DBUser::get_organizations(target_user.id, &**pool)
        .await
        .wrap_internal_err("fetching users from database")?;
    let organizations_data =
        DBOrganization::get_many_ids(&organization_ids, &**pool, &redis)
            .await
            .wrap_internal_err("fetching organizations from database")?;

    let team_ids = organizations_data
        .iter()
        .map(|organization| organization.team_id)
        .collect::<Vec<_>>();
    let teams_data =
        crate::database::models::DBTeamMember::get_from_team_full_many(
            &team_ids, &**pool, &redis,
        )
        .await
        .wrap_internal_err("fetching team members from database")?;
    let users = DBUser::get_many_ids(
        &teams_data
            .iter()
            .map(|member| member.user_id)
            .collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching users from database")?;

    let mut team_groups = HashMap::new();
    for member in teams_data {
        team_groups
            .entry(member.team_id)
            .or_insert(vec![])
            .push(member);
    }

    let mut organizations = HashMap::new();
    let mut visible_organization_ids = Vec::new();
    for data in organizations_data {
        if !is_visible_organization(&data, &user, &pool, &redis)
            .await
            .wrap_api_err("checking organization visibility")?
        {
            continue;
        }

        visible_organization_ids.push(data.id);
        let members_data = team_groups.remove(&data.team_id).unwrap_or(vec![]);
        let logged_in = user
            .as_ref()
            .and_then(|user| {
                members_data
                    .iter()
                    .find(|x| x.user_id == user.id.into() && x.accepted)
            })
            .is_some();
        let team_members = members_data
            .into_iter()
            .filter(|x| logged_in || x.accepted || target_user.id == x.user_id)
            .filter_map(|data| {
                users.iter().find(|x| x.id == data.user_id).map(|member| {
                    crate::models::teams::TeamMember::from(
                        data,
                        member.clone(),
                        !logged_in,
                    )
                })
            })
            .collect();

        organizations.insert(
            OrganizationId::from(data.id),
            Organization::from(data, team_members),
        );
    }

    let organization_id_values = visible_organization_ids
        .iter()
        .map(|id| id.0)
        .collect::<Vec<_>>();
    let organization_project_ids = sqlx::query!(
        "
        SELECT m.id
        FROM mods m
        WHERE m.organization_id = ANY($1)
        ",
        &organization_id_values,
    )
    .fetch_all(&**pool)
    .await
    .wrap_internal_err("fetching organization project IDs from database")?
    .into_iter()
    .map(|row| DBProjectId(row.id))
    .collect::<Vec<_>>();

    let project_ids = user_project_ids
        .into_iter()
        .chain(organization_project_ids)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let projects_data =
        crate::database::DBProject::get_many_ids(&project_ids, &**pool, &redis)
            .await
            .wrap_api_err("fetching user and organization projects")?;
    let projects = filter_visible_projects(projects_data, &user, &pool, true)
        .await
        .wrap_api_err("filtering visible projects")?;

    Ok(web::Json(AllProjectsResponse {
        projects,
        organizations,
    }))
}

#[utoipa::path(
	tag = "users",
	params(("email" = String, Query)),
	responses((status = OK))
)]
#[get("/user_email")]
pub async fn admin_user_email(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    email: web::Query<UserEmailQuery>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .map(|x| x.1)
    .wrap_auth_err("authenticating API request")?;

    if !user.role.is_admin() {
        return Err(ApiError::Auth(eyre::eyre!(
            "You do not have permission to get a user from their email!",
        )));
    }

    let user_id = sqlx::query!(
        "
        SELECT id FROM users
        WHERE LOWER(email) = LOWER($1)
        ",
        email.email
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("fetching user ID from database")?
    .map(|x| x.id)
    .wrap_request_err_with(|| {
        "the email provided is not associated with a user!".to_string()
    })?;

    let user = DBUser::get_id(
        crate::database::models::DBUserId(user_id),
        &**pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching user from database")?;

    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[utoipa::path(tag = "users", responses((status = OK)))]
#[get("/user/{user_id}/projects")]
pub async fn projects_list_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    projects_list(req, info, pool, redis, session_queue).await
}

pub async fn projects_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let id_option = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(id) = id_option.map(|x| x.id) {
        let project_data = DBUser::get_projects(id, &**pool, &redis)
            .await
            .wrap_internal_err("fetching user from database")?;

        let projects: Vec<_> = crate::database::DBProject::get_many_ids(
            &project_data,
            &**pool,
            &redis,
        )
        .await
        .wrap_api_err("fetching organization projects")?;
        let projects = filter_visible_projects(projects, &user, &pool, true)
            .await
            .wrap_api_err("filtering visible projects")?;
        Ok(HttpResponse::Ok().json(projects))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[utoipa::path(tag = "users", responses((status = OK)))]
#[get("/user")]
pub async fn user_auth_get_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    user_auth_get(req, pool, redis, session_queue).await
}

pub async fn user_auth_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (scopes, mut user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    if !scopes.contains(Scopes::USER_READ_EMAIL) {
        user.email = None;
    }

    if !scopes.contains(Scopes::PAYOUTS_READ) {
        user.payout_data = None;
    }

    if user.role.is_mod() {
        let note = DBModerationNote::get_user(user.id.into(), &**pool, &redis)
            .await
            .wrap_internal_err("fetching moderation note from database")?;
        user.moderation_notes = Some(note.map(Into::into));
    }

    Ok(HttpResponse::Ok().json(user))
}

#[utoipa::path(tag = "users", responses((status = OK, body = UserPreferences)))]
#[get("/user/{id}/preferences")]
pub async fn get_user_preferences(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<UserPreferences>, ApiError> {
    let (_, requester) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let target = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("resource not found")?;

    let can_access =
        requester.id == target.id.into() || requester.role.is_mod();
    if !can_access {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to access this user's preferences"
        )));
    }

    let preference_overrides = DBUserPreferences::get(target.id, &**pool)
        .await
        .wrap_internal_err("failed to fetch user preferences")?;

    Ok(web::Json(UserPreferences::resolve(preference_overrides)))
}

#[utoipa::path(
	tag = "users",
	request_body = PartialUserPreferences,
	responses((status = OK, body = UserPreferences))
)]
#[patch("/user/{id}/preferences")]
pub async fn edit_user_preferences(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<PartialUserPreferences>,
) -> Result<web::Json<UserPreferences>, ApiError> {
    let (_, requester) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let target = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("resource not found")?;

    let can_access =
        requester.id == target.id.into() || requester.role.is_mod();
    if !can_access {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to access this user's preferences"
        )));
    }

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let stored = DBUserPreferences::get_for_update(target.id, &mut txn)
        .await
        .wrap_internal_err("failed to fetch user preferences")?;

    let mut preferences = UserPreferences::resolve(stored);
    body.into_inner().apply_to(&mut preferences);

    let overrides = preferences.into_diff_from(&UserPreferences::default());
    DBUserPreferences::upsert(target.id, &overrides, &mut txn)
        .await
        .wrap_internal_err("failed to update user preferences")?;

    txn.commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    let preferences = UserPreferences::resolve(Some(overrides));

    Ok(web::Json(preferences))
}

#[derive(Serialize, Deserialize)]
pub struct UserIds {
    pub ids: String,
}

#[derive(Deserialize)]
pub struct UserSearchQuery {
    pub query: String,
}

#[utoipa::path(
	tag = "users",
	params(("query" = String, Query)),
	responses((status = OK))
)]
#[get("/users/search")]
pub async fn users_search(
    web::Query(query): web::Query<UserSearchQuery>,
    pool: web::Data<PgPool>,
) -> Result<web::Json<Vec<crate::models::users::SearchUser>>, ApiError> {
    let query = query.query.trim();
    let users = DBUser::search(query, &**pool)
        .await
        .wrap_internal_err("failed to search users")?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(web::Json(users))
}

#[utoipa::path(
	tag = "users",
	params(("ids" = String, Query)),
	responses((status = OK))
)]
#[get("/users")]
pub async fn users_get_route(
    req: HttpRequest,
    ids: web::Query<UserIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    users_get(req, ids, pool, redis, session_queue).await
}

pub async fn users_get(
    req: HttpRequest,
    web::Query(ids): web::Query<UserIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_ids = serde_json::from_str::<Vec<String>>(&ids.ids)
        .wrap_request_err("deserializing JSON data")?;

    let users_data = DBUser::get_many(&user_ids, &**pool, &redis)
        .await
        .wrap_internal_err("fetching users from database")?;

    let auth_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .map(|x| x.1)
    .ok();

    let notes = if auth_user.as_ref().is_some_and(|x| x.role.is_mod()) {
        DBModerationNote::get_many_users(
            &users_data.iter().map(|x| x.id).collect::<Vec<_>>(),
            &**pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching moderation notes from database")?
    } else {
        HashMap::new()
    };

    let users: Vec<crate::models::users::User> = users_data
        .into_iter()
        .map(|data| {
            let mut user = crate::models::users::User::from(data.clone());
            if auth_user.as_ref().is_some_and(|x| x.role.is_mod()) {
                user.moderation_notes =
                    Some(notes.get(&data.id).cloned().map(Into::into));
            }
            user
        })
        .collect();

    Ok(HttpResponse::Ok().json(users))
}

#[utoipa::path(tag = "users", responses((status = OK)))]
#[get("/user/{id}")]
pub async fn user_get_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    user_get(req, info, pool, redis, session_queue).await
}

pub async fn user_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user_data = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(data) = user_data {
        let auth_user = get_user_from_headers(
            &req,
            &**pool,
            &redis,
            &session_queue,
            Scopes::SESSION_ACCESS,
        )
        .await
        .map(|x| x.1)
        .ok();

        let is_admin = auth_user.as_ref().is_some_and(|x| x.role.is_admin());
        let is_mod = auth_user.as_ref().is_some_and(|x| x.role.is_mod());
        let user_id = data.id;

        let mut response: crate::models::users::User = if is_admin {
            let github_id =
                data.github_id.and_then(|id| u64::try_from(id).ok());
            let discord_id = data.discord_id.map(|id| id.to_string());
            let steam_id = data.steam_id.map(|id| id.to_string());
            let mut user = crate::models::users::User::from_full(data);
            user.github_id = github_id;
            user.discord_id = discord_id;
            user.steam_id = steam_id;
            user
        } else {
            data.into()
        };

        if is_mod {
            let note = DBModerationNote::get_user(user_id, &**pool, &redis)
                .await
                .wrap_internal_err("fetching moderation note from database")?;
            response.moderation_notes = Some(note.map(Into::into));
        }

        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[utoipa::path(tag = "users", responses((status = NO_CONTENT)))]
#[patch("/user/{id}/notes")]
pub async fn user_notes_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    new_note: web::Json<crate::models::moderation_notes::PatchModerationNote>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    new_note
        .validate_not_empty()
        .wrap_api_err("validating not empty")?;
    let expected_version =
        crate::models::moderation_notes::parse_if_match_header(&req)
            .wrap_api_err(
                "executing `moderation_notes::parse_if_match_header`",
            )?;

    let user_data = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .wrap_not_found_err("resource not found")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;
    if let Some(expected) = expected_version {
        let updated = DBModerationNote::update(
            Some(user_data.id),
            None,
            user.id.into(),
            expected,
            new_note.notes.as_deref(),
            new_note.user_rating,
            &mut transaction,
        )
        .await
        .wrap_internal_err("updating moderation note in database")?;

        if updated.is_none() {
            return Err(ApiError::PreconditionFailed(eyre::eyre!(
                "moderation note version does not match",
            )));
        }
    } else {
        let updated = DBModerationNote::insert(
            Some(user_data.id),
            None,
            user.id.into(),
            new_note.notes.as_deref(),
            new_note.user_rating,
            &mut transaction,
        )
        .await
        .wrap_internal_err("inserting moderation note into database")?;

        if updated.is_none() {
            return Err(ApiError::PreconditionRequired(eyre::eyre!(
                "moderation note version does not match",
            )));
        }
    };

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;
    DBModerationNote::clear_user_cache(user_data.id, &redis)
        .await
        .wrap_internal_err("clearing cached moderation note in Redis")?;

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(tag = "users", responses((status = OK)))]
#[get("/user/{user_id}/collections")]
pub async fn collections_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::COLLECTION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let id_option = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(id) = id_option.map(|x| x.id) {
        let collection_data = DBUser::get_collections(id, &**pool)
            .await
            .wrap_internal_err("fetching user from database")?;

        let response: Vec<_> = crate::database::models::DBCollection::get_many(
            &collection_data,
            &**pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching collections from database")?;

        let collections = filter_visible_collections(response, &user, true)
            .await
            .wrap_api_err("filtering visible collections")?;

        Ok(HttpResponse::Ok().json(collections))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[utoipa::path(tag = "users", responses((status = OK)))]
#[get("/user/{user_id}/organizations")]
pub async fn orgs_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let id_option = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(id) = id_option.map(|x| x.id) {
        let org_data = DBUser::get_organizations(id, &**pool)
            .await
            .wrap_internal_err("fetching user from database")?;

        let organizations_data =
            crate::database::models::organization_item::DBOrganization::get_many_ids(
                &org_data, &**pool, &redis,
            )
            .await.wrap_internal_err("fetching organizations from database")?;

        let team_ids = organizations_data
            .iter()
            .map(|x| x.team_id)
            .collect::<Vec<_>>();

        let teams_data =
            crate::database::models::DBTeamMember::get_from_team_full_many(
                &team_ids, &**pool, &redis,
            )
            .await
            .wrap_internal_err("fetching team members from database")?;
        let users = DBUser::get_many_ids(
            &teams_data.iter().map(|x| x.user_id).collect::<Vec<_>>(),
            &**pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching users from database")?;

        let mut organizations = vec![];
        let mut team_groups = HashMap::new();
        for item in teams_data {
            team_groups.entry(item.team_id).or_insert(vec![]).push(item);
        }

        for data in organizations_data {
            if !is_visible_organization(&data, &user, &pool, &redis)
                .await
                .wrap_api_err("checking organization visibility")?
            {
                continue;
            }

            let members_data =
                team_groups.remove(&data.team_id).unwrap_or(vec![]);
            let logged_in = user
                .as_ref()
                .and_then(|user| {
                    members_data
                        .iter()
                        .find(|x| x.user_id == user.id.into() && x.accepted)
                })
                .is_some();

            let team_members: Vec<_> = members_data
                .into_iter()
                .filter(|x| logged_in || x.accepted || id == x.user_id)
                .filter_map(|data| {
                    users.iter().find(|x| x.id == data.user_id).map(|user| {
                        crate::models::teams::TeamMember::from(
                            data,
                            user.clone(),
                            !logged_in,
                        )
                    })
                })
                .collect();

            let organization = crate::models::organizations::Organization::from(
                data,
                team_members,
            );
            organizations.push(organization);
        }

        Ok(HttpResponse::Ok().json(organizations))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct EditUser {
    #[validate(length(min = 1, max = 39), regex(path = *crate::util::validate::RE_URL_SAFE))]
    pub username: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 160))]
    pub bio: Option<Option<String>>,
    pub role: Option<Role>,
    pub badges: Option<Badges>,
    #[validate(length(max = 160))]
    pub venmo_handle: Option<String>,
    pub allow_friend_requests: Option<bool>,
}

#[utoipa::path(tag = "users", responses((status = NO_CONTENT)))]
#[patch("/user/{id}")]
pub async fn user_edit_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    new_user: web::Json<EditUser>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    user_edit(req, info, new_user, pool, redis, session_queue).await
}

pub async fn user_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    new_user: web::Json<EditUser>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (scopes, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    new_user
        .validate()
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("validating request")?;

    let id_option = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(actual_user) = id_option {
        let id = actual_user.id;
        let user_id: UserId = id.into();

        if user.id == user_id || user.role.is_mod() {
            let mut transaction = pool
                .begin()
                .await
                .wrap_internal_err("starting database transaction")?;

            if let Some(username) = &new_user.username {
                let existing_user_id_option =
                    DBUser::get(username, &**pool, &redis)
                        .await
                        .wrap_internal_err("fetching user from database")?;

                if existing_user_id_option
                    .map(|x| UserId::from(x.id))
                    .is_none_or(|id| id == user.id)
                {
                    sqlx::query!(
                        "
                        UPDATE users
                        SET username = $1
                        WHERE (id = $2)
                        ",
                        username,
                        id as crate::database::models::ids::DBUserId,
                    )
                    .execute(&mut transaction)
                    .await
                    .wrap_internal_err("querying database for `user_edit`")?;
                } else {
                    return Err(ApiError::Request(eyre::eyre!(format!(
                        "Username {username} is taken!"
                    ))));
                }
            }

            if let Some(bio) = &new_user.bio {
                sqlx::query!(
                    "
                    UPDATE users
                    SET bio = $1
                    WHERE (id = $2)
                    ",
                    bio.as_deref(),
                    id as crate::database::models::ids::DBUserId,
                )
                .execute(&mut transaction)
                .await
                .wrap_internal_err("fetching bio from database")?;
            }

            if let Some(role) = &new_user.role {
                if !user.role.is_admin() {
                    return Err(ApiError::Auth(eyre::eyre!(
                        "You do not have the permissions to edit the role of this user!",
                    )));
                }

                let role = role.to_string();

                sqlx::query!(
                    "
                    UPDATE users
                    SET role = $1
                    WHERE (id = $2)
                    ",
                    role,
                    id as crate::database::models::ids::DBUserId,
                )
                .execute(&mut transaction)
                .await
                .wrap_internal_err("querying database for `user_edit`")?;
            }

            if let Some(badges) = &new_user.badges {
                if !user.role.is_admin() {
                    return Err(ApiError::Auth(eyre::eyre!(
                        "You do not have the permissions to edit the badges of this user!",
                    )));
                }

                sqlx::query!(
                    "
                    UPDATE users
                    SET badges = $1
                    WHERE (id = $2)
                    ",
                    badges.bits() as i64,
                    id as crate::database::models::ids::DBUserId,
                )
                .execute(&mut transaction)
                .await
                .wrap_internal_err("querying database for `user_edit`")?;
            }

            if let Some(venmo_handle) = &new_user.venmo_handle {
                if !scopes.contains(Scopes::PAYOUTS_WRITE) {
                    return Err(ApiError::Auth(eyre::eyre!(
                        "You do not have the permissions to edit the venmo handle of this user!",
                    )));
                }

                sqlx::query!(
                    "
                    UPDATE users
                    SET venmo_handle = $1
                    WHERE (id = $2)
                    ",
                    venmo_handle,
                    id as crate::database::models::ids::DBUserId,
                )
                .execute(&mut transaction)
                .await
                .wrap_internal_err("querying database for `user_edit`")?;
            }

            if let Some(allow_friend_requests) = &user.allow_friend_requests {
                sqlx::query!(
                    "
                    UPDATE users
                    SET allow_friend_requests = $1
                    WHERE (id = $2)
                    ",
                    allow_friend_requests,
                    id as crate::database::models::ids::DBUserId,
                )
                .execute(&mut transaction)
                .await
                .wrap_internal_err(
                    "fetching allow friend requests from database",
                )?;
            }

            transaction
                .commit()
                .await
                .wrap_internal_err("committing database transaction")?;
            DBUser::clear_caches(&[(id, Some(actual_user.username))], &redis)
                .await
                .wrap_internal_err("clearing cached data from Redis")?;
            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::Auth(eyre::eyre!(
                "You do not have permission to edit this user!",
            )))
        }
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[allow(clippy::too_many_arguments)]
#[utoipa::path(
	tag = "users",
	params(("ext" = String, Query)),
	request_body(content = Vec<u8>, content_type = "application/octet-stream"),
	responses((status = NO_CONTENT))
)]
#[patch("/user/{id}/icon")]
pub async fn user_icon_edit_route(
    ext: web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    user_icon_edit(
        ext,
        req,
        info,
        pool,
        redis,
        file_host,
        payload,
        session_queue,
    )
    .await
}

pub async fn user_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let id_option = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(actual_user) = id_option {
        if user.id != actual_user.id.into() && !user.role.is_mod() {
            return Err(ApiError::Auth(eyre::eyre!(
                "You don't have permission to edit this user's icon.",
            )));
        }

        delete_old_images(
            actual_user.avatar_url,
            actual_user.raw_avatar_url,
            FileHostPublicity::Public,
            &**file_host,
        )
        .await
        .wrap_api_err("deleting old images")?;

        let bytes = read_limited_from_payload(
            &mut payload,
            262144,
            "Icons must be smaller than 256KiB",
        )
        .await
        .wrap_api_err("executing `read_limited_from_payload`")?;

        let user_id: UserId = actual_user.id.into();
        let upload_result = crate::util::img::upload_image_optimized(
            &format!("data/{user_id}"),
            FileHostPublicity::Public,
            bytes.freeze(),
            &ext.ext,
            Some(96),
            Some(1.0),
            &**file_host,
        )
        .await
        .wrap_api_err("uploading image")?;

        sqlx::query!(
            "
            UPDATE users
            SET avatar_url = $1, raw_avatar_url = $2
            WHERE (id = $3)
            ",
            upload_result.url,
            upload_result.raw_url,
            actual_user.id as crate::database::models::ids::DBUserId,
        )
        .execute(&**pool)
        .await
        .wrap_internal_err("querying database for `user_icon_edit`")?;
        DBUser::clear_caches(&[(actual_user.id, None)], &redis)
            .await
            .wrap_internal_err("clearing cached data from Redis")?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[utoipa::path(tag = "users", responses((status = NO_CONTENT)))]
#[delete("/user/{id}/icon")]
pub async fn user_icon_delete_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    user_icon_delete(req, info, pool, redis, file_host, session_queue).await
}

pub async fn user_icon_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let id_option = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(actual_user) = id_option {
        if user.id != actual_user.id.into() && !user.role.is_mod() {
            return Err(ApiError::Auth(eyre::eyre!(
                "You don't have permission to edit this user's icon.",
            )));
        }

        delete_old_images(
            actual_user.avatar_url,
            actual_user.raw_avatar_url,
            FileHostPublicity::Public,
            &**file_host,
        )
        .await
        .wrap_api_err("deleting old images")?;

        sqlx::query!(
            "
            UPDATE users
            SET avatar_url = NULL, raw_avatar_url = NULL
            WHERE (id = $1)
            ",
            actual_user.id as crate::database::models::ids::DBUserId,
        )
        .execute(&**pool)
        .await
        .wrap_internal_err("querying database for `user_icon_delete`")?;

        DBUser::clear_caches(&[(actual_user.id, None)], &redis)
            .await
            .wrap_internal_err("clearing cached data from Redis")?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[utoipa::path(tag = "users", responses((status = NO_CONTENT)))]
#[delete("/user/{id}")]
pub async fn user_delete_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    user_delete(req, info, pool, redis, session_queue).await
}

pub async fn user_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_DELETE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let id_option = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("failed to get user")?;

    let id = id_option
        .map(|x| x.id)
        .wrap_not_found_err("resource not found")?;
    if !user.role.is_admin() && user.id != id.into() {
        return Err(ApiError::Auth(eyre::eyre!(
            "You do not have permission to delete this user!",
        )));
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    let result = DBUser::remove(id, &mut transaction, &redis)
        .await
        .wrap_internal_err("failed to remove user")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    if result.is_some() {
        Ok(())
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[utoipa::path(tag = "users", responses((status = OK)))]
#[get("/user/{id}/follows")]
pub async fn user_follows_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    user_follows(req, info, pool, redis, session_queue).await
}

pub async fn user_follows(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let id_option = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(id) = id_option.map(|x| x.id) {
        if !user.role.is_admin() && user.id != id.into() {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have permission to see the projects this user follows!",
            )));
        }

        let project_ids = DBUser::get_follows(id, &**pool)
            .await
            .wrap_internal_err("fetching users from database")?;
        let projects: Vec<_> = crate::database::DBProject::get_many_ids(
            &project_ids,
            &**pool,
            &redis,
        )
        .await
        .wrap_api_err("fetching followed projects")?
        .into_iter()
        .map(Project::from)
        .collect();

        Ok(HttpResponse::Ok().json(projects))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[utoipa::path(tag = "users", responses((status = OK)))]
#[get("/user/{id}/notifications")]
pub async fn user_notifications_route(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    user_notifications(req, info, pool, redis, session_queue).await
}

pub async fn user_notifications(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::NOTIFICATION_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let id_option = DBUser::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(id) = id_option.map(|x| x.id) {
        if !user.role.is_admin() && user.id != id.into() {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have permission to see the notifications of this user!",
            )));
        }

        let mut notifications: Vec<Notification> =
            crate::database::models::notification_item::DBNotification::get_many_user_exposed_on_site(
                id, &**pool, &redis,
            )
            .await.wrap_internal_err("fetching notifications from database")?
            .into_iter()
            .map(Into::into)
            .collect();

        notifications.sort_by_key(|b| Reverse(b.created));
        Ok(HttpResponse::Ok().json(notifications))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}
