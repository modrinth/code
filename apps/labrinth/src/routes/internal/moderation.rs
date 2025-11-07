use super::ApiError;
use crate::database;
use crate::database::models::{DBOrganization, DBTeamId, DBTeamMember, DBUser};
use crate::database::redis::RedisPool;
use crate::models::ids::{OrganizationId, TeamId};
use crate::models::projects::{Project, ProjectStatus};
use crate::queue::moderation::{ApprovalType, IdentifiedFile, MissingMetadata};
use crate::queue::session::AuthQueue;
use crate::util::error::Context;
use crate::{auth::check_is_moderator_from_headers, models::pats::Scopes};
use actix_web::{HttpRequest, get, post, web};
use ariadne::ids::{UserId, random_base62};
use eyre::eyre;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(get_projects)
        .service(get_project_meta)
        .service(set_project_meta);
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ProjectsRequestOptions {
    /// How many projects to fetch.
    #[serde(default = "default_count")]
    pub count: u16,
    /// How many projects to skip.
    #[serde(default)]
    pub offset: u32,
}

fn default_count() -> u16 {
    100
}

/// Project with extra information fetched from the database, to avoid having
/// clients make more round trips.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FetchedProject {
    /// Project info.
    #[serde(flatten)]
    pub project: Project,
    /// Who owns the project.
    pub ownership: Ownership,
}

/// Fetched information on who owns a project.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Ownership {
    /// Project is owned by a team, and this is the team owner.
    User {
        /// ID of the team owner.
        id: UserId,
        /// Name of the team owner.
        name: String,
        /// URL of the team owner's icon.
        icon_url: Option<String>,
    },
    /// Project is owned by an organization.
    Organization {
        /// ID of the organization.
        id: OrganizationId,
        /// Name of the organization.
        name: String,
        /// URL of the organization's icon.
        icon_url: Option<String>,
    },
}

/// Fetch all projects which are in the moderation queue.
#[utoipa::path(
    responses((status = OK, body = inline(Vec<FetchedProject>)))
)]
#[get("/projects")]
async fn get_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ProjectsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<FetchedProject>>, ApiError> {
    get_projects_internal(req, pool, redis, request_opts, session_queue).await
}

pub async fn get_projects_internal(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ProjectsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<FetchedProject>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    use futures::stream::TryStreamExt;

    let project_ids = sqlx::query!(
        "
        SELECT id FROM mods
        WHERE status = $1
        ORDER BY queued ASC
        OFFSET $3
        LIMIT $2
        ",
        ProjectStatus::Processing.as_str(),
        request_opts.count as i64,
        request_opts.offset as i64
    )
    .fetch(&**pool)
    .map_ok(|m| database::models::DBProjectId(m.id))
    .try_collect::<Vec<database::models::DBProjectId>>()
    .await
    .wrap_internal_err("failed to fetch projects awaiting review")?;

    let projects =
        database::DBProject::get_many_ids(&project_ids, &**pool, &redis)
            .await
            .wrap_internal_err("failed to fetch projects")?
            .into_iter()
            .map(crate::models::projects::Project::from)
            .collect::<Vec<_>>();

    let team_ids = projects
        .iter()
        .map(|project| project.team_id)
        .map(DBTeamId::from)
        .collect::<Vec<_>>();
    let org_ids = projects
        .iter()
        .filter_map(|project| project.organization)
        .collect::<Vec<_>>();

    let team_members =
        DBTeamMember::get_from_team_full_many(&team_ids, &**pool, &redis)
            .await
            .wrap_internal_err("failed to fetch team members")?;
    let users = DBUser::get_many_ids(
        &team_members
            .iter()
            .map(|member| member.user_id)
            .collect::<Vec<_>>(),
        &**pool,
        &redis,
    )
    .await
    .wrap_internal_err("failed to fetch user data of team members")?;
    let orgs = DBOrganization::get_many(&org_ids, &**pool, &redis)
        .await
        .wrap_internal_err("failed to fetch organizations")?;

    let map_project = |project: Project| -> Result<FetchedProject, ApiError> {
        let project_id = project.id;
        let ownership = if let Some(org_id) = project.organization {
            let org = orgs
                    .iter()
                    .find(|org| OrganizationId::from(org.id) == org_id)
                    .wrap_internal_err_with(|| {
                        eyre!(
                            "project {project_id} is owned by an invalid organization {org_id}"
                        )
                    })?;

            Ownership::Organization {
                id: OrganizationId::from(org.id),
                name: org.name.clone(),
                icon_url: org.icon_url.clone(),
            }
        } else {
            let team_id = project.team_id;
            let team_owner = team_members.iter().find(|member| TeamId::from(member.team_id) == team_id && member.is_owner)
                .wrap_internal_err_with(|| eyre!("project {project_id} is owned by a team {team_id} which has no valid owner"))?;
            let team_owner_id = team_owner.user_id;
            let user = users.iter().find(|user| user.id == team_owner_id)
                .wrap_internal_err_with(|| eyre!("project {project_id} is owned by a team {team_id} which has owner {} which does not exist", UserId::from(team_owner_id)))?;

            Ownership::User {
                id: UserId::from(user.id),
                name: user.username.clone(),
                icon_url: user.avatar_url.clone(),
            }
        };

        Ok(FetchedProject { ownership, project })
    };

    let projects = projects
        .into_iter()
        .map(map_project)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(web::Json(projects))
}

/// Fetch moderation metadata for a specific project.
#[utoipa::path(
    responses((status = OK, body = inline(Vec<Project>)))
)]
#[get("/project/{id}")]
async fn get_project_meta(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    info: web::Path<(String,)>,
) -> Result<web::Json<MissingMetadata>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let project_id = info.into_inner().0;
    let project =
        database::models::DBProject::get(&project_id, &**pool, &redis).await?;

    if let Some(project) = project {
        let rows = sqlx::query!(
            "
            SELECT
            f.metadata, v.id version_id
            FROM versions v
            INNER JOIN files f ON f.version_id = v.id
            WHERE v.mod_id = $1
            ",
            project.inner.id.0
        )
        .fetch_all(&**pool)
        .await?;

        let mut merged = MissingMetadata {
            identified: HashMap::new(),
            flame_files: HashMap::new(),
            unknown_files: HashMap::new(),
        };

        let mut check_hashes = Vec::new();
        let mut check_flames = Vec::new();

        for row in rows {
            if let Some(metadata) = row
                .metadata
                .and_then(|x| serde_json::from_value::<MissingMetadata>(x).ok())
            {
                merged.identified.extend(metadata.identified);
                merged.flame_files.extend(metadata.flame_files);
                merged.unknown_files.extend(metadata.unknown_files);

                check_hashes.extend(merged.flame_files.keys().cloned());
                check_hashes.extend(merged.unknown_files.keys().cloned());
                check_flames
                    .extend(merged.flame_files.values().map(|x| x.id as i32));
            }
        }

        let rows = sqlx::query!(
            "
            SELECT encode(mef.sha1, 'escape') sha1, mel.status status
            FROM moderation_external_files mef
            INNER JOIN moderation_external_licenses mel ON mef.external_license_id = mel.id
            WHERE mef.sha1 = ANY($1)
            ",
            &check_hashes
                .iter()
                .map(|x| x.as_bytes().to_vec())
                .collect::<Vec<_>>()
        )
        .fetch_all(&**pool)
        .await?;

        for row in rows {
            if let Some(sha1) = row.sha1 {
                if let Some(val) = merged.flame_files.remove(&sha1) {
                    merged.identified.insert(
                        sha1,
                        IdentifiedFile {
                            file_name: val.file_name,
                            status: ApprovalType::from_string(&row.status)
                                .unwrap_or(ApprovalType::Unidentified),
                        },
                    );
                } else if let Some(val) = merged.unknown_files.remove(&sha1) {
                    merged.identified.insert(
                        sha1,
                        IdentifiedFile {
                            file_name: val,
                            status: ApprovalType::from_string(&row.status)
                                .unwrap_or(ApprovalType::Unidentified),
                        },
                    );
                }
            }
        }

        let rows = sqlx::query!(
            "
            SELECT mel.id, mel.flame_project_id, mel.status status
            FROM moderation_external_licenses mel
            WHERE mel.flame_project_id = ANY($1)
            ",
            &check_flames,
        )
        .fetch_all(&**pool)
        .await?;

        for row in rows {
            if let Some(sha1) = merged
                .flame_files
                .iter()
                .find(|x| Some(x.1.id as i32) == row.flame_project_id)
                .map(|x| x.0.clone())
                && let Some(val) = merged.flame_files.remove(&sha1)
            {
                merged.identified.insert(
                    sha1,
                    IdentifiedFile {
                        file_name: val.file_name.clone(),
                        status: ApprovalType::from_string(&row.status)
                            .unwrap_or(ApprovalType::Unidentified),
                    },
                );
            }
        }

        Ok(web::Json(merged))
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Judgement {
    Flame {
        id: i32,
        status: ApprovalType,
        link: String,
        title: String,
    },
    Unknown {
        status: ApprovalType,
        proof: Option<String>,
        link: Option<String>,
        title: Option<String>,
    },
}

/// Update moderation judgements for projects in the review queue.
#[utoipa::path]
#[post("/project")]
async fn set_project_meta(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    judgements: web::Json<HashMap<String, Judgement>>,
) -> Result<(), ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    let mut ids = Vec::new();
    let mut titles = Vec::new();
    let mut statuses = Vec::new();
    let mut links = Vec::new();
    let mut proofs = Vec::new();
    let mut flame_ids = Vec::new();

    let mut file_hashes = Vec::new();

    for (hash, judgement) in judgements.0 {
        let id = random_base62(8);

        let (title, status, link, proof, flame_id) = match judgement {
            Judgement::Flame {
                id,
                status,
                link,
                title,
            } => (
                Some(title),
                status,
                Some(link),
                Some("See Flame page/license for permission".to_string()),
                Some(id),
            ),
            Judgement::Unknown {
                status,
                proof,
                link,
                title,
            } => (title, status, link, proof, None),
        };

        ids.push(id as i64);
        titles.push(title);
        statuses.push(status.as_str());
        links.push(link);
        proofs.push(proof);
        flame_ids.push(flame_id);
        file_hashes.push(hash);
    }

    sqlx::query(
    "
        INSERT INTO moderation_external_licenses (id, title, status, link, proof, flame_project_id)
        SELECT * FROM UNNEST ($1::bigint[], $2::varchar[], $3::varchar[], $4::varchar[], $5::varchar[], $6::integer[])
        "
    )
        .bind(&ids[..])
        .bind(&titles[..])
        .bind(&statuses[..])
        .bind(&links[..])
        .bind(&proofs[..])
        .bind(&flame_ids[..])
        .execute(&mut *transaction)
        .await?;

    sqlx::query(
        "
        INSERT INTO moderation_external_files (sha1, external_license_id)
        SELECT * FROM UNNEST ($1::bytea[], $2::bigint[])
        ON CONFLICT (sha1)
        DO NOTHING
        ",
    )
    .bind(&file_hashes[..])
    .bind(&ids[..])
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}
