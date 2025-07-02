use std::collections::HashMap;
use std::sync::Arc;

use super::ApiError;
use crate::auth::{filter_visible_projects, get_user_from_headers};
use crate::database::models::team_item::DBTeamMember;
use crate::database::models::{
    DBOrganization, generate_organization_id, team_item,
};
use crate::database::redis::RedisPool;
use crate::file_hosting::{FileHost, FileHostPublicity};
use crate::models::ids::OrganizationId;
use crate::models::pats::Scopes;
use crate::models::teams::{OrganizationPermissions, ProjectPermissions};
use crate::queue::session::AuthQueue;
use crate::routes::v3::project_creation::CreateError;
use crate::util::img::delete_old_images;
use crate::util::routes::read_limited_from_payload;
use crate::util::validate::validation_errors_to_string;
use crate::{database, models};
use actix_web::{HttpRequest, HttpResponse, web};
use ariadne::ids::UserId;
use ariadne::ids::base62_impl::parse_base62;
use futures::TryStreamExt;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("organizations", web::get().to(organizations_get));
    cfg.service(
        web::scope("organization")
            .route("", web::post().to(organization_create))
            .route("{id}/projects", web::get().to(organization_projects_get))
            .route("{id}", web::get().to(organization_get))
            .route("{id}", web::patch().to(organizations_edit))
            .route("{id}", web::delete().to(organization_delete))
            .route("{id}/projects", web::post().to(organization_projects_add))
            .route(
                "{id}/projects/{project_id}",
                web::delete().to(organization_projects_remove),
            )
            .route("{id}/icon", web::patch().to(organization_icon_edit))
            .route("{id}/icon", web::delete().to(delete_organization_icon))
            .route(
                "{id}/members",
                web::get().to(super::teams::team_members_get_organization),
            ),
    );
}

pub async fn organization_projects_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ORGANIZATION_READ | Scopes::PROJECT_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let organization_data = DBOrganization::get(&id, &**pool, &redis).await?;
    if let Some(organization) = organization_data {
        let project_ids = sqlx::query!(
            "
            SELECT m.id FROM organizations o
            INNER JOIN mods m ON m.organization_id = o.id
            WHERE o.id = $1
            ",
            organization.id as database::models::ids::DBOrganizationId
        )
        .fetch(&**pool)
        .map_ok(|m| database::models::DBProjectId(m.id))
        .try_collect::<Vec<_>>()
        .await?;

        let projects_data = crate::database::models::DBProject::get_many_ids(
            &project_ids,
            &**pool,
            &redis,
        )
        .await?;

        let projects =
            filter_visible_projects(projects_data, &current_user, &pool, true)
                .await?;

        Ok(HttpResponse::Ok().json(projects))
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Deserialize, Validate)]
pub struct NewOrganization {
    #[validate(
        length(min = 3, max = 64),
        regex(path = *crate::util::validate::RE_URL_SAFE)
    )]
    pub slug: String,
    // Title of the organization
    #[validate(length(min = 3, max = 64))]
    pub name: String,
    #[validate(length(min = 3, max = 256))]
    pub description: String,
}

pub async fn organization_create(
    req: HttpRequest,
    new_organization: web::Json<NewOrganization>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, CreateError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ORGANIZATION_CREATE,
    )
    .await?
    .1;

    new_organization.validate().map_err(|err| {
        CreateError::ValidationError(validation_errors_to_string(err, None))
    })?;

    let mut transaction = pool.begin().await?;

    // Try title
    let name_organization_id_option: Option<OrganizationId> =
        serde_json::from_str(&format!("\"{}\"", new_organization.slug)).ok();
    let mut organization_strings = vec![];
    if let Some(name_organization_id) = name_organization_id_option {
        organization_strings.push(name_organization_id.to_string());
    }
    organization_strings.push(new_organization.slug.clone());
    let results = DBOrganization::get_many(
        &organization_strings,
        &mut *transaction,
        &redis,
    )
    .await?;
    if !results.is_empty() {
        return Err(CreateError::SlugCollision);
    }

    let organization_id = generate_organization_id(&mut transaction).await?;

    // Create organization managerial team
    let team = team_item::TeamBuilder {
        members: vec![team_item::TeamMemberBuilder {
            user_id: current_user.id.into(),
            role: crate::models::teams::DEFAULT_ROLE.to_owned(),
            is_owner: true,
            permissions: ProjectPermissions::all(),
            organization_permissions: Some(OrganizationPermissions::all()),
            accepted: true,
            payouts_split: Decimal::ONE_HUNDRED,
            ordering: 0,
        }],
    };
    let team_id = team.insert(&mut transaction).await?;

    // Create organization
    let organization = DBOrganization {
        id: organization_id,
        slug: new_organization.slug.clone(),
        name: new_organization.name.clone(),
        description: new_organization.description.clone(),
        team_id,
        icon_url: None,
        raw_icon_url: None,
        color: None,
    };
    organization.clone().insert(&mut transaction).await?;
    transaction.commit().await?;

    // Only member is the owner, the logged in one
    let member_data =
        DBTeamMember::get_from_team_full(team_id, &**pool, &redis)
            .await?
            .into_iter()
            .next();
    let members_data = if let Some(member_data) = member_data {
        vec![crate::models::teams::TeamMember::from_model(
            member_data,
            current_user.clone(),
            false,
        )]
    } else {
        return Err(CreateError::InvalidInput(
            "Failed to get created team.".to_owned(), // should never happen
        ));
    };

    let organization =
        models::organizations::Organization::from(organization, members_data);

    Ok(HttpResponse::Ok().json(organization))
}

pub async fn organization_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ORGANIZATION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();
    let user_id = current_user.as_ref().map(|x| x.id.into());

    let organization_data = DBOrganization::get(&id, &**pool, &redis).await?;
    if let Some(data) = organization_data {
        let members_data =
            DBTeamMember::get_from_team_full(data.team_id, &**pool, &redis)
                .await?;

        let users = crate::database::models::DBUser::get_many_ids(
            &members_data.iter().map(|x| x.user_id).collect::<Vec<_>>(),
            &**pool,
            &redis,
        )
        .await?;
        let logged_in = current_user
            .as_ref()
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
                    || user_id.is_some_and(
                        |y: crate::database::models::DBUserId| y == x.user_id,
                    )
            })
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

        let organization =
            models::organizations::Organization::from(data, team_members);
        return Ok(HttpResponse::Ok().json(organization));
    }
    Err(ApiError::NotFound)
}

#[derive(Deserialize)]
pub struct OrganizationIds {
    pub ids: String,
}

pub async fn organizations_get(
    req: HttpRequest,
    web::Query(ids): web::Query<OrganizationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids = serde_json::from_str::<Vec<&str>>(&ids.ids)?;
    let organizations_data =
        DBOrganization::get_many(&ids, &**pool, &redis).await?;
    let team_ids = organizations_data
        .iter()
        .map(|x| x.team_id)
        .collect::<Vec<_>>();

    let teams_data =
        DBTeamMember::get_from_team_full_many(&team_ids, &**pool, &redis)
            .await?;
    let users = crate::database::models::DBUser::get_many_ids(
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
        Scopes::ORGANIZATION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();
    let user_id = current_user.as_ref().map(|x| x.id.into());

    let mut organizations = vec![];

    let mut team_groups = HashMap::new();
    for item in teams_data {
        team_groups.entry(item.team_id).or_insert(vec![]).push(item);
    }

    for data in organizations_data {
        let members_data = team_groups.remove(&data.team_id).unwrap_or(vec![]);
        let logged_in = current_user
            .as_ref()
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
                    || user_id.is_some_and(
                        |y: crate::database::models::DBUserId| y == x.user_id,
                    )
            })
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

        let organization =
            models::organizations::Organization::from(data, team_members);
        organizations.push(organization);
    }

    Ok(HttpResponse::Ok().json(organizations))
}

#[derive(Serialize, Deserialize, Validate)]
pub struct OrganizationEdit {
    #[validate(length(min = 3, max = 256))]
    pub description: Option<String>,
    #[validate(
        length(min = 3, max = 64),
        regex(path = *crate::util::validate::RE_URL_SAFE)
    )]
    pub slug: Option<String>,
    #[validate(length(min = 3, max = 64))]
    pub name: Option<String>,
}

pub async fn organizations_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    new_organization: web::Json<OrganizationEdit>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ORGANIZATION_WRITE,
    )
    .await?
    .1;

    new_organization.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let string = info.into_inner().0;
    let result =
        database::models::DBOrganization::get(&string, &**pool, &redis).await?;
    if let Some(organization_item) = result {
        let id = organization_item.id;

        let team_member = database::models::DBTeamMember::get_from_user_id(
            organization_item.team_id,
            user.id.into(),
            &**pool,
        )
        .await?;

        let permissions = OrganizationPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
        );

        if let Some(perms) = permissions {
            let mut transaction = pool.begin().await?;
            if let Some(description) = &new_organization.description {
                if !perms.contains(OrganizationPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the description of this organization!"
                            .to_string(),
                    ));
                }
                sqlx::query!(
                    "
                    UPDATE organizations
                    SET description = $1
                    WHERE (id = $2)
                    ",
                    description,
                    id as database::models::ids::DBOrganizationId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(name) = &new_organization.name {
                if !perms.contains(OrganizationPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the name of this organization!"
                            .to_string(),
                    ));
                }
                sqlx::query!(
                    "
                    UPDATE organizations
                    SET name = $1
                    WHERE (id = $2)
                    ",
                    name,
                    id as database::models::ids::DBOrganizationId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(slug) = &new_organization.slug {
                if !perms.contains(OrganizationPermissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the slug of this organization!"
                            .to_string(),
                    ));
                }

                let name_organization_id_option: Option<u64> =
                    parse_base62(slug).ok();
                if let Some(name_organization_id) = name_organization_id_option
                {
                    let results = sqlx::query!(
                        "
                        SELECT EXISTS(SELECT 1 FROM organizations WHERE id=$1)
                        ",
                        name_organization_id as i64
                    )
                    .fetch_one(&mut *transaction)
                    .await?;

                    if results.exists.unwrap_or(true) {
                        return Err(ApiError::InvalidInput(
                            "slug collides with other organization's id!"
                                .to_string(),
                        ));
                    }
                }

                // Make sure the new name is different from the old one
                // We are able to unwrap here because the name is always set
                if !slug.eq(&organization_item.slug.clone()) {
                    let results = sqlx::query!(
                        "
                        SELECT EXISTS(SELECT 1 FROM organizations WHERE LOWER(slug) = LOWER($1))
                        ",
                        slug
                    )
                    .fetch_one(&mut *transaction)
                    .await?;

                    if results.exists.unwrap_or(true) {
                        return Err(ApiError::InvalidInput(
                            "slug collides with other organization's id!"
                                .to_string(),
                        ));
                    }
                }

                sqlx::query!(
                    "
                    UPDATE organizations
                    SET slug = $1
                    WHERE (id = $2)
                    ",
                    Some(slug),
                    id as database::models::ids::DBOrganizationId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            transaction.commit().await?;
            database::models::DBOrganization::clear_cache(
                organization_item.id,
                Some(organization_item.slug),
                &redis,
            )
            .await?;

            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthentication(
                "You do not have permission to edit this organization!"
                    .to_string(),
            ))
        }
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn organization_delete(
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
        Scopes::ORGANIZATION_DELETE,
    )
    .await?
    .1;
    let string = info.into_inner().0;

    let organization =
        database::models::DBOrganization::get(&string, &**pool, &redis)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified organization does not exist!".to_string(),
                )
            })?;

    if !user.role.is_admin() {
        let team_member =
            database::models::DBTeamMember::get_from_user_id_organization(
                organization.id,
                user.id.into(),
                false,
                &**pool,
            )
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified organization does not exist!".to_string(),
                )
            })?;

        let permissions = OrganizationPermissions::get_permissions_by_role(
            &user.role,
            &Some(team_member),
        )
        .unwrap_or_default();

        if !permissions.contains(OrganizationPermissions::DELETE_ORGANIZATION) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to delete this organization!"
                    .to_string(),
            ));
        }
    }

    let owner_id = sqlx::query!(
        "
        SELECT user_id FROM team_members
        WHERE team_id = $1 AND is_owner = TRUE
        ",
        organization.team_id as database::models::ids::DBTeamId
    )
    .fetch_one(&**pool)
    .await?
    .user_id;
    let owner_id = database::models::ids::DBUserId(owner_id);

    let mut transaction = pool.begin().await?;

    // Handle projects- every project that is in this organization needs to have its owner changed the organization owner
    // Now, no project should have an owner if it is in an organization, and also
    // the owner of an organization should not be a team member in any project
    let organization_project_teams = sqlx::query!(
        "
        SELECT t.id FROM organizations o
        INNER JOIN mods m ON m.organization_id = o.id
        INNER JOIN teams t ON t.id = m.team_id
        WHERE o.id = $1 AND $1 IS NOT NULL
        ",
        organization.id as database::models::ids::DBOrganizationId
    )
    .fetch(&mut *transaction)
    .map_ok(|c| database::models::DBTeamId(c.id))
    .try_collect::<Vec<_>>()
    .await?;

    for organization_project_team in &organization_project_teams {
        let new_id = crate::database::models::ids::generate_team_member_id(
            &mut transaction,
        )
        .await?;
        let member = DBTeamMember {
            id: new_id,
            team_id: *organization_project_team,
            user_id: owner_id,
            role: "Inherited Owner".to_string(),
            is_owner: true,
            permissions: ProjectPermissions::all(),
            organization_permissions: None,
            accepted: true,
            payouts_split: Decimal::ZERO,
            ordering: 0,
        };
        member.insert(&mut transaction).await?;
    }
    // Safely remove the organization
    let result = database::models::DBOrganization::remove(
        organization.id,
        &mut transaction,
        &redis,
    )
    .await?;

    transaction.commit().await?;

    database::models::DBOrganization::clear_cache(
        organization.id,
        Some(organization.slug),
        &redis,
    )
    .await?;

    for team_id in &organization_project_teams {
        database::models::DBTeamMember::clear_cache(*team_id, &redis).await?;
    }

    if !organization_project_teams.is_empty() {
        database::models::DBUser::clear_project_cache(&[owner_id], &redis)
            .await?;
    }

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Deserialize)]
pub struct OrganizationProjectAdd {
    pub project_id: String, // Also allow name/slug
}
pub async fn organization_projects_add(
    req: HttpRequest,
    info: web::Path<(String,)>,
    project_info: web::Json<OrganizationProjectAdd>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let info = info.into_inner().0;
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE | Scopes::ORGANIZATION_WRITE,
    )
    .await?
    .1;

    let organization =
        database::models::DBOrganization::get(&info, &**pool, &redis)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified organization does not exist!".to_string(),
                )
            })?;

    let project_item = database::models::DBProject::get(
        &project_info.project_id,
        &**pool,
        &redis,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "The specified project does not exist!".to_string(),
        )
    })?;
    if project_item.inner.organization_id.is_some() {
        return Err(ApiError::InvalidInput(
            "The specified project is already owned by an organization!"
                .to_string(),
        ));
    }

    let project_team_member =
        database::models::DBTeamMember::get_from_user_id_project(
            project_item.inner.id,
            current_user.id.into(),
            false,
            &**pool,
        )
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "You are not a member of this project!".to_string(),
            )
        })?;
    let organization_team_member =
        database::models::DBTeamMember::get_from_user_id_organization(
            organization.id,
            current_user.id.into(),
            false,
            &**pool,
        )
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "You are not a member of this organization!".to_string(),
            )
        })?;

    // Require ownership of a project to add it to an organization
    if !current_user.role.is_admin() && !project_team_member.is_owner {
        return Err(ApiError::CustomAuthentication(
            "You need to be an owner of a project to add it to an organization!".to_string(),
        ));
    }

    let permissions = OrganizationPermissions::get_permissions_by_role(
        &current_user.role,
        &Some(organization_team_member),
    )
    .unwrap_or_default();
    if permissions.contains(OrganizationPermissions::ADD_PROJECT) {
        let mut transaction = pool.begin().await?;
        sqlx::query!(
            "
            UPDATE mods
            SET organization_id = $1
            WHERE (id = $2)
            ",
            organization.id as database::models::DBOrganizationId,
            project_item.inner.id as database::models::ids::DBProjectId
        )
        .execute(&mut *transaction)
        .await?;

        // The former owner is no longer an owner (as it is now 'owned' by the organization, 'given' to them)
        // The former owner is still a member of the project, but not an owner
        // When later removed from the organization, the project will  be owned by whoever is specified as the new owner there

        let organization_owner_user_id = sqlx::query!(
            "
            SELECT u.id
            FROM team_members
            INNER JOIN users u ON u.id = team_members.user_id
            WHERE team_id = $1 AND is_owner = TRUE
            ",
            organization.team_id as database::models::ids::DBTeamId
        )
        .fetch_one(&mut *transaction)
        .await?;
        let organization_owner_user_id =
            database::models::ids::DBUserId(organization_owner_user_id.id);

        sqlx::query!(
            "
            DELETE FROM team_members
            WHERE team_id = $1 AND (is_owner = TRUE OR user_id = $2)
            ",
            project_item.inner.team_id as database::models::ids::DBTeamId,
            organization_owner_user_id as database::models::ids::DBUserId,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        database::models::DBUser::clear_project_cache(
            &[current_user.id.into()],
            &redis,
        )
        .await?;
        database::models::DBTeamMember::clear_cache(
            project_item.inner.team_id,
            &redis,
        )
        .await?;
        database::models::DBProject::clear_cache(
            project_item.inner.id,
            project_item.inner.slug,
            None,
            &redis,
        )
        .await?;
    } else {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to add projects to this organization!"
                .to_string(),
        ));
    }
    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct OrganizationProjectRemoval {
    // A new owner must be supplied for the project.
    // That user must be a member of the organization, but not necessarily a member of the project.
    pub new_owner: UserId,
}

pub async fn organization_projects_remove(
    req: HttpRequest,
    info: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    data: web::Json<OrganizationProjectRemoval>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (organization_id, project_id) = info.into_inner();
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE | Scopes::ORGANIZATION_WRITE,
    )
    .await?
    .1;

    let organization = database::models::DBOrganization::get(
        &organization_id,
        &**pool,
        &redis,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "The specified organization does not exist!".to_string(),
        )
    })?;

    let project_item =
        database::models::DBProject::get(&project_id, &**pool, &redis)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified project does not exist!".to_string(),
                )
            })?;

    if !project_item
        .inner
        .organization_id
        .eq(&Some(organization.id))
    {
        return Err(ApiError::InvalidInput(
            "The specified project is not owned by this organization!"
                .to_string(),
        ));
    }

    let organization_team_member =
        database::models::DBTeamMember::get_from_user_id_organization(
            organization.id,
            current_user.id.into(),
            false,
            &**pool,
        )
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "You are not a member of this organization!".to_string(),
            )
        })?;

    let permissions = OrganizationPermissions::get_permissions_by_role(
        &current_user.role,
        &Some(organization_team_member),
    )
    .unwrap_or_default();
    if permissions.contains(OrganizationPermissions::REMOVE_PROJECT) {
        // Now that permissions are confirmed, we confirm the veracity of the new user as an org member
        database::models::DBTeamMember::get_from_user_id_organization(
            organization.id,
            data.new_owner.into(),
            false,
            &**pool,
        )
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified user is not a member of this organization!"
                    .to_string(),
            )
        })?;

        // Then, we get the team member of the project and that user (if it exists)
        // We use the team member get directly
        let new_owner =
            database::models::DBTeamMember::get_from_user_id_project(
                project_item.inner.id,
                data.new_owner.into(),
                true,
                &**pool,
            )
            .await?;

        let mut transaction = pool.begin().await?;

        // If the user is not a member of the project, we add them
        let new_owner = match new_owner {
            Some(new_owner) => new_owner,
            None => {
                let new_id =
                    crate::database::models::ids::generate_team_member_id(
                        &mut transaction,
                    )
                    .await?;
                let member = DBTeamMember {
                    id: new_id,
                    team_id: project_item.inner.team_id,
                    user_id: data.new_owner.into(),
                    role: "Inherited Owner".to_string(),
                    is_owner: false,
                    permissions: ProjectPermissions::all(),
                    organization_permissions: None,
                    accepted: true,
                    payouts_split: Decimal::ZERO,
                    ordering: 0,
                };
                member.insert(&mut transaction).await?;
                member
            }
        };

        // Set the new owner to fit owner
        sqlx::query!(
            "
            UPDATE team_members
            SET
                is_owner = TRUE,
                accepted = TRUE,
                permissions = $2,
                organization_permissions = NULL,
                role = 'Inherited Owner'
            WHERE (id = $1)
            ",
            new_owner.id as database::models::ids::DBTeamMemberId,
            ProjectPermissions::all().bits() as i64
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            UPDATE mods
            SET organization_id = NULL
            WHERE (id = $1)
            ",
            project_item.inner.id as database::models::ids::DBProjectId
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        database::models::DBUser::clear_project_cache(
            &[current_user.id.into()],
            &redis,
        )
        .await?;
        database::models::DBTeamMember::clear_cache(
            project_item.inner.team_id,
            &redis,
        )
        .await?;
        database::models::DBProject::clear_cache(
            project_item.inner.id,
            project_item.inner.slug,
            None,
            &redis,
        )
        .await?;
    } else {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to add projects to this organization!"
                .to_string(),
        ));
    }
    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn organization_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ORGANIZATION_WRITE,
    )
    .await?
    .1;
    let string = info.into_inner().0;

    let organization_item =
        database::models::DBOrganization::get(&string, &**pool, &redis)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified organization does not exist!".to_string(),
                )
            })?;

    if !user.role.is_mod() {
        let team_member = database::models::DBTeamMember::get_from_user_id(
            organization_item.team_id,
            user.id.into(),
            &**pool,
        )
        .await
        .map_err(ApiError::Database)?;

        let permissions = OrganizationPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(OrganizationPermissions::EDIT_DETAILS) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit this organization's icon."
                    .to_string(),
            ));
        }
    }

    delete_old_images(
        organization_item.icon_url,
        organization_item.raw_icon_url,
        FileHostPublicity::Public,
        &***file_host,
    )
    .await?;

    let bytes = read_limited_from_payload(
        &mut payload,
        262144,
        "Icons must be smaller than 256KiB",
    )
    .await?;

    let organization_id: OrganizationId = organization_item.id.into();
    let upload_result = crate::util::img::upload_image_optimized(
        &format!("data/{organization_id}"),
        FileHostPublicity::Public,
        bytes.freeze(),
        &ext.ext,
        Some(96),
        Some(1.0),
        &***file_host,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        UPDATE organizations
        SET icon_url = $1, raw_icon_url = $2, color = $3
        WHERE (id = $4)
        ",
        upload_result.url,
        upload_result.raw_url,
        upload_result.color.map(|x| x as i32),
        organization_item.id as database::models::ids::DBOrganizationId,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    database::models::DBOrganization::clear_cache(
        organization_item.id,
        Some(organization_item.slug),
        &redis,
    )
    .await?;

    Ok(HttpResponse::NoContent().body(""))
}

pub async fn delete_organization_icon(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ORGANIZATION_WRITE,
    )
    .await?
    .1;
    let string = info.into_inner().0;

    let organization_item =
        database::models::DBOrganization::get(&string, &**pool, &redis)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified organization does not exist!".to_string(),
                )
            })?;

    if !user.role.is_mod() {
        let team_member = database::models::DBTeamMember::get_from_user_id(
            organization_item.team_id,
            user.id.into(),
            &**pool,
        )
        .await
        .map_err(ApiError::Database)?;

        let permissions = OrganizationPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(OrganizationPermissions::EDIT_DETAILS) {
            return Err(ApiError::CustomAuthentication(
                "You don't have permission to edit this organization's icon."
                    .to_string(),
            ));
        }
    }

    delete_old_images(
        organization_item.icon_url,
        organization_item.raw_icon_url,
        FileHostPublicity::Public,
        &***file_host,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        UPDATE organizations
        SET icon_url = NULL, raw_icon_url = NULL, color = NULL
        WHERE (id = $1)
        ",
        organization_item.id as database::models::ids::DBOrganizationId,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    database::models::DBOrganization::clear_cache(
        organization_item.id,
        Some(organization_item.slug),
        &redis,
    )
    .await?;

    Ok(HttpResponse::NoContent().body(""))
}
