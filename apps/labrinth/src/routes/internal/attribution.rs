use actix_web::{HttpRequest, get, patch, post, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::ids::{
    DBAttributionGroupId, DBProjectId, generate_attribution_group_id,
};
use crate::database::redis::RedisPool;
use crate::models::ids::{ProjectId, VersionId};
use crate::models::pats::Scopes;
use crate::models::projects::{
    AttributionModerationStatusKind, AttributionResolution,
    AttributionResolutionKind,
};
use crate::models::users::User;
use crate::queue::moderation::ApprovalType;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(
        utoipa_actix_web::scope("/attribution")
            .service(list)
            .service(update_group)
            .service(assign)
            .service(split),
    );
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FlameProject {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub icon_url: String,
}

#[derive(Serialize)]
struct AttributionGroupResponse {
    id: crate::models::ids::AttributionGroupId,
    flame_project: Option<FlameProject>,
    attribution: Option<crate::models::projects::AttributionResolution>,
    attributed_at: Option<chrono::DateTime<chrono::Utc>>,
    attributed_by: Option<ariadne::ids::UserId>,
    files: Vec<AttributionFileResponse>,
    versions: Vec<VersionInfo>,
}

#[derive(Clone, Serialize)]
struct VersionInfo {
    id: VersionId,
    name: String,
    version_number: String,
    date_created: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct AttributionFileResponse {
    name: String,
    sha1: String,
    versions: Vec<VersionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    moderation_external_license_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    moderation_external_license: Option<ModerationExternalLicenseResponse>,
}

#[derive(Clone, Serialize)]
struct ModerationExternalLicenseResponse {
    id: i64,
    title: Option<String>,
    status: ApprovalType,
    link: Option<String>,
    exceptions: Option<String>,
    proof: Option<String>,
    flame_project_id: Option<i32>,
    inserted_at: Option<DateTime<Utc>>,
    inserted_by: Option<i64>,
    updated_at: Option<DateTime<Utc>>,
    updated_by: Option<i64>,
}

#[utoipa::path]
#[get("{project_id}")]
async fn list(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<ProjectId>,
) -> Result<web::Json<Vec<AttributionGroupResponse>>, ApiError> {
    let project_id: DBProjectId = path.into_inner().into();
    let show_moderation_external_license_ids = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .ok()
    .is_some_and(|(_, user)| user.role.is_mod());

    let groups = sqlx::query!(
        r#"
		select
			g.id as "id: DBAttributionGroupId",
			g.flame_project,
			g.attribution,
			g.attributed_at,
			g.attributed_by as "attributed_by: i64"
		from project_attribution_groups g
		where g.project_id = $1
		"#,
        project_id as DBProjectId,
    )
    .fetch_all(pool.as_ref())
    .await?;

    let group_ids: Vec<i64> = groups.iter().map(|g| g.id.0).collect();

    let files = if group_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query(
            "
			select paf.group_id, paf.name, convert_from(paf.sha1, 'UTF8') as sha1, paf.moderation_external_license_id,
				coalesce(array_agg(distinct f.version_id) filter (where f.version_id is not null), '{}') as version_ids
			from project_attribution_files paf
			left join override_file_sources ofs on ofs.sha1 = paf.sha1
			left join files f on f.id = ofs.file_id
			where paf.group_id = ANY($1)
			group by paf.group_id, paf.name, paf.sha1, paf.moderation_external_license_id
			",
        )
        .bind(&group_ids)
        .fetch_all(pool.as_ref())
        .await?
    };

    let moderation_external_licenses = if show_moderation_external_license_ids {
        let mut ids: Vec<i64> = files
            .iter()
            .filter_map(|f| f.get("moderation_external_license_id"))
            .collect();
        ids.sort_unstable();
        ids.dedup();

        if ids.is_empty() {
            std::collections::HashMap::new()
        } else {
            sqlx::query!(
                r#"
				select
					id,
					title,
					status,
					link,
					exceptions,
					proof,
					flame_project_id,
					inserted_at,
					inserted_by,
					updated_at,
					updated_by
				from moderation_external_licenses
				where id = ANY($1)
				"#,
                &ids,
            )
            .fetch_all(pool.as_ref())
            .await?
            .into_iter()
            .map(|row| {
                (
                    row.id,
                    ModerationExternalLicenseResponse {
                        id: row.id,
                        title: row.title,
                        status: ApprovalType::from_string(&row.status)
                            .unwrap_or(ApprovalType::Unidentified),
                        link: row.link,
                        exceptions: row.exceptions,
                        proof: row.proof,
                        flame_project_id: row.flame_project_id,
                        inserted_at: row.inserted_at,
                        inserted_by: row.inserted_by,
                        updated_at: row.updated_at,
                        updated_by: row.updated_by,
                    },
                )
            })
            .collect()
        }
    } else {
        std::collections::HashMap::new()
    };

    let mut all_version_ids: Vec<i64> = files
        .iter()
        .flat_map(|f| f.get::<Vec<i64>, _>("version_ids"))
        .collect();
    all_version_ids.sort_unstable();
    all_version_ids.dedup();

    let version_infos = if all_version_ids.is_empty() {
        Vec::new()
    } else {
        let rows = sqlx::query!(
            "
			select id, name, version_number, date_published
			from versions
			where id = ANY($1)
			order by date_published desc
			",
            &all_version_ids,
        )
        .fetch_all(pool.as_ref())
        .await?;
        rows.into_iter()
            .map(|v| VersionInfo {
                id: VersionId(v.id as u64),
                name: v.name,
                version_number: v.version_number,
                date_created: v.date_published,
            })
            .collect()
    };
    let version_order = version_infos
        .iter()
        .enumerate()
        .map(|(index, version)| (version.id, index))
        .collect::<std::collections::HashMap<_, _>>();

    let mut result = Vec::new();
    for group in groups {
        let group_files: Vec<AttributionFileResponse> = files
            .iter()
            .filter(|f| f.get::<i64, _>("group_id") == group.id.0)
            .map(|f| AttributionFileResponse {
                name: f.get("name"),
                sha1: f.get("sha1"),
                moderation_external_license_id:
                    if show_moderation_external_license_ids {
                        f.get("moderation_external_license_id")
                    } else {
                        None
                    },
                moderation_external_license:
                    if show_moderation_external_license_ids {
                        f.get::<Option<i64>, _>(
                            "moderation_external_license_id",
                        )
                        .and_then(|id| {
                            moderation_external_licenses.get(&id).cloned()
                        })
                    } else {
                        None
                    },
                versions: {
                    let mut versions: Vec<_> = f
                        .get::<Vec<i64>, _>("version_ids")
                        .into_iter()
                        .map(|id| VersionId(id as u64))
                        .collect();
                    versions.sort_by_key(|id| {
                        version_order.get(id).copied().unwrap_or(usize::MAX)
                    });
                    versions
                },
            })
            .collect();

        result.push(AttributionGroupResponse {
            id: group.id.into(),
            flame_project: group
                .flame_project
                .and_then(|v| serde_json::from_value(v).ok()),
            attribution: group
                .attribution
                .and_then(|v| serde_json::from_value(v).ok()),
            attributed_at: group.attributed_at,
            attributed_by: group
                .attributed_by
                .map(|id| ariadne::ids::UserId(id as u64)),
            files: group_files,
            versions: version_infos.clone(),
        });
    }

    Ok(web::Json(result))
}

#[derive(Deserialize, utoipa::ToSchema)]
struct UpdateGroupBody {
    attribution: AttributionResolution,
}

#[utoipa::path]
#[patch("group/{group_id}")]
async fn update_group(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<i64>,
    web::Json(body): web::Json<UpdateGroupBody>,
) -> Result<(), ApiError> {
    let group_id = path.into_inner();
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::VERSION_WRITE,
    )
    .await?
    .1;

    if !can_edit_attribution_group(pool.as_ref(), group_id, &user).await? {
        return Err(ApiError::CustomAuthentication(
            "This attribution group cannot be edited".to_string(),
        ));
    }

    if matches!(
        body.attribution.kind,
        AttributionResolutionKind::GloballyAllowed { .. }
    ) && !user.role.is_mod()
    {
        return Err(ApiError::CustomAuthentication(
            "Only moderators can set globally allowed attributions".to_string(),
        ));
    }

    if body.attribution.moderation_status.is_some() && !user.role.is_mod() {
        return Err(ApiError::CustomAuthentication(
            "Only moderators can set attribution moderation status".to_string(),
        ));
    }

    let result = sqlx::query!(
        "
		update project_attribution_groups
		set attribution = $1, attributed_at = now(), attributed_by = $3
		where id = $2
		",
        &serde_json::to_value(&body.attribution).unwrap_or_default(),
        group_id,
        user.id.0 as i64,
    )
    .execute(pool.as_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

#[derive(Deserialize, utoipa::ToSchema)]
struct AssignBody {
    sha1: String,
    target_group_id: i64,
    project_id: ProjectId,
}

#[utoipa::path]
#[post("assign")]
async fn assign(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    web::Json(body): web::Json<AssignBody>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::VERSION_WRITE,
    )
    .await?
    .1;

    let sha1_bytes = hex_to_bytes(&body.sha1).ok_or_else(|| {
        ApiError::InvalidInput("invalid sha1 hex string".to_string())
    })?;
    let project_id: DBProjectId = body.project_id.into();

    let source_group_id = sqlx::query_scalar!(
        "
		select paf.group_id
		from project_attribution_files paf
		inner join project_attribution_groups pag on pag.id = paf.group_id
		where paf.sha1 = $1 and pag.project_id = $2
		",
        &sha1_bytes,
        project_id as DBProjectId,
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or(ApiError::NotFound)?;

    let target_group_exists = sqlx::query_scalar!(
        "
		select exists(
			select 1 from project_attribution_groups where id = $1 and project_id = $2
		) as \"exists!\"
		",
        body.target_group_id,
        project_id as DBProjectId,
    )
    .fetch_one(pool.as_ref())
    .await?;

    if !target_group_exists {
        return Err(ApiError::NotFound);
    }

    if !can_edit_attribution_group(pool.as_ref(), source_group_id, &user)
        .await?
        || !can_edit_attribution_group(
            pool.as_ref(),
            body.target_group_id,
            &user,
        )
        .await?
    {
        return Err(ApiError::CustomAuthentication(
            "This attribution group cannot be edited".to_string(),
        ));
    }

    let result = sqlx::query!(
        "
		update project_attribution_files
		set group_id = $1
		where sha1 = $2
		and group_id in (
			select id from project_attribution_groups where project_id = $3
		)
		",
        body.target_group_id,
        &sha1_bytes,
        project_id as DBProjectId,
    )
    .execute(pool.as_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    cleanup_empty_groups(pool.as_ref()).await?;

    Ok(())
}

#[derive(Deserialize, utoipa::ToSchema)]
struct SplitBody {
    sha1: String,
    project_id: ProjectId,
}

#[utoipa::path]
#[post("split")]
async fn split(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    web::Json(body): web::Json<SplitBody>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::VERSION_WRITE,
    )
    .await?
    .1;

    let sha1_bytes = hex_to_bytes(&body.sha1).ok_or_else(|| {
        ApiError::InvalidInput("invalid sha1 hex string".to_string())
    })?;
    let project_id: DBProjectId = body.project_id.into();

    let existing = sqlx::query!(
        "
		select paf.group_id, paf.name from project_attribution_files paf
		inner join project_attribution_groups pag on pag.id = paf.group_id
		where paf.sha1 = $1 and pag.project_id = $2
		",
        &sha1_bytes,
        project_id as DBProjectId,
    )
    .fetch_optional(pool.as_ref())
    .await?;

    let Some(existing) = existing else {
        return Err(ApiError::NotFound);
    };

    if !can_edit_attribution_group(pool.as_ref(), existing.group_id, &user)
        .await?
    {
        return Err(ApiError::CustomAuthentication(
            "This attribution group cannot be edited".to_string(),
        ));
    }

    let mut txn = pool.begin().await?;

    let new_group_id = generate_attribution_group_id(&mut txn).await?;

    sqlx::query!(
        "
		insert into project_attribution_groups (id, project_id)
		values ($1, $2)
		",
        new_group_id as DBAttributionGroupId,
        project_id as DBProjectId,
    )
    .execute(&mut txn)
    .await?;

    sqlx::query!(
        "
		update project_attribution_files
		set group_id = $1
		where sha1 = $2 and group_id = $3
		",
        new_group_id as DBAttributionGroupId,
        &sha1_bytes,
        existing.group_id,
    )
    .execute(&mut txn)
    .await?;

    txn.commit().await?;

    cleanup_empty_groups(pool.as_ref()).await?;

    Ok(())
}

async fn can_edit_attribution_group(
    pool: &PgPool,
    group_id: i64,
    user: &User,
) -> Result<bool, ApiError> {
    if user.role.is_mod() {
        return Ok(true);
    }

    let attribution = sqlx::query_scalar!(
        "
		select attribution
		from project_attribution_groups
		where id = $1
		",
        group_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let attribution: Option<AttributionResolution> =
        attribution.and_then(|value| serde_json::from_value(value).ok());

    Ok(!matches!(
        attribution
            .and_then(|attribution| attribution.moderation_status)
            .map(|status| status.kind),
        Some(AttributionModerationStatusKind::NotAllowed)
    ))
}

async fn cleanup_empty_groups(pool: &PgPool) -> Result<(), ApiError> {
    sqlx::query!(
        "
		delete from project_attribution_groups g
		where not exists (
			select 1 from project_attribution_files f where f.group_id = g.id
		)
		",
    )
    .execute(pool)
    .await?;
    Ok(())
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}
