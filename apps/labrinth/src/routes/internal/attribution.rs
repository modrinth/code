use actix_web::{HttpRequest, delete, get, patch, post, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::auth::{check_is_moderator_from_headers, get_user_from_headers};
use crate::database::PgPool;
use crate::database::models::{
    DBFileId, DBOrganization, DBProject, DBTeamMember, DBVersion,
    ids::{
        DBAttributionGroupId, DBProjectId, DBVersionId,
        generate_attribution_group_id,
    },
};
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::ids::{FileId, ProjectId, VersionId};
use crate::models::pats::Scopes;
use crate::models::projects::{
    AttributionModerationStatusKind, AttributionResolution,
    AttributionResolutionKind, FlameProject,
};
use crate::models::teams::ProjectPermissions;
use crate::models::users::User;
use crate::queue::file_scan::{FileScanSummary, scan_file};
use crate::queue::moderation::ApprovalType;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(list)
        .service(update_group)
        .service(delete_group)
        .service(scan)
        .service(force_scan_file)
        .service(assign)
        .service(split);
}

#[derive(Serialize, utoipa::ToSchema)]
struct AttributionGroupResponse {
    id: crate::models::ids::AttributionGroupId,
    flame_project: Option<FlameProject>,
    attribution: Option<crate::models::projects::AttributionResolution>,
    attributed_at: Option<chrono::DateTime<chrono::Utc>>,
    attributed_by: Option<ariadne::ids::UserId>,
    files: Vec<AttributionFileResponse>,
    versions: Vec<VersionInfo>,
    override_files_on_platform: Vec<OverrideFileOnPlatformResponse>,
}

#[derive(Clone, Serialize, utoipa::ToSchema)]
struct VersionInfo {
    id: VersionId,
    name: String,
    version_number: String,
    date_created: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct AttributionFileResponse {
    name: String,
    sha1: String,
    versions: Vec<VersionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    moderation_external_license_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    moderation_external_license: Option<ModerationExternalLicenseResponse>,
}

#[derive(Clone, Serialize, utoipa::ToSchema)]
struct OverrideFileOnPlatformResponse {
    file_path: String,
    sha1: String,
    version_id: VersionId,
    platform_version_id: VersionId,
    platform_project_id: ProjectId,
}

#[derive(Clone, Serialize, utoipa::ToSchema)]
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

#[derive(Deserialize, utoipa::ToSchema)]
struct ScanBody {
    version_ids: Vec<VersionId>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ScanResponse {
    queued_files: u64,
}

/// Queue an attribution scan.
#[utoipa::path(
	context_path = "/attribution",
	tag = "attribution",
	responses((status = OK, body = ScanResponse))
)]
#[post("/scan")]
pub async fn scan(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    web::Json(body): web::Json<ScanBody>,
) -> Result<web::Json<ScanResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::VERSION_WRITE,
    )
    .await?
    .1;

    let mut version_ids: Vec<i64> = body
        .version_ids
        .into_iter()
        .map(|id| DBVersionId::from(id).0)
        .collect();
    version_ids.sort_unstable();
    version_ids.dedup();

    if version_ids.is_empty() {
        return Ok(web::Json(ScanResponse { queued_files: 0 }));
    }

    let versions = sqlx::query!(
        r#"
		select
			id as "id: DBVersionId",
			mod_id as "project_id: DBProjectId"
		from versions
		where id = any($1)
		"#,
        &version_ids,
    )
    .fetch_all(pool.as_ref())
    .await
    .wrap_internal_err("failed to fetch versions for attribution scan")?;

    if versions.len() != version_ids.len() {
        return Err(ApiError::NotFound);
    }

    let mut project_ids: Vec<DBProjectId> =
        versions.iter().map(|version| version.project_id).collect();
    project_ids.sort_unstable_by_key(|id| id.0);
    project_ids.dedup_by_key(|id| id.0);

    for project_id in &project_ids {
        ensure_can_upload_versions_to_project(
            pool.as_ref(),
            *project_id,
            &user,
            "you do not have permission to upload versions to this project",
        )
        .await?;
    }

    let project_ids = project_ids.iter().map(|id| id.0).collect::<Vec<_>>();
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin attribution scan transaction")?;

    sqlx::query!(
        r#"
		delete from attributions_exemptions
		where version_id = any($1) or project_id = any($2)
		"#,
        &version_ids,
        &project_ids,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("failed to remove attribution scan exemptions")?;

    let result = sqlx::query!(
        r#"
		insert into file_scans (file_id)
		select f.id
		from files f
		where f.version_id = any($1)
		on conflict (file_id) do nothing
		"#,
        &version_ids,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("failed to queue version files for attribution scan")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit attribution scan transaction")?;

    let version_ids =
        version_ids.into_iter().map(DBVersionId).collect::<Vec<_>>();
    DBVersion::clear_cache_ids(&version_ids, redis.as_ref())
        .await
        .wrap_internal_err("failed to clear version cache")?;

    Ok(web::Json(ScanResponse {
        queued_files: result.rows_affected(),
    }))
}

#[utoipa::path]
#[post("/file/{file_id}/scan")]
async fn force_scan_file(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    file_host: web::Data<dyn FileHost>,
    path: web::Path<FileId>,
) -> Result<web::Json<FileScanSummary>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let file_id: DBFileId = path.into_inner().into();
    let file = sqlx::query!(
        r#"
		select
			f.url,
			f.version_id as "version_id: DBVersionId",
			v.mod_id as "project_id: DBProjectId"
		from files f
		inner join versions v on v.id = f.version_id
		where f.id = $1
		"#,
        file_id as DBFileId,
    )
    .fetch_optional(pool.as_ref())
    .await
    .wrap_internal_err("failed to fetch attribution scan file")?
    .ok_or(ApiError::NotFound)?;

    let mut transaction = pool.begin().await.wrap_internal_err(
        "failed to begin attribution file scan transaction",
    )?;

    sqlx::query!(
        r#"
		delete from attributions_exemptions
		where version_id = $1
		"#,
        file.version_id as DBVersionId,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("failed to remove attribution scan exemption")?;

    let scan_summary = scan_file(
        &mut transaction,
        redis.as_ref(),
        &**file_host,
        file.project_id,
        file_id,
        &file.url,
    )
    .await
    .wrap_internal_err("failed to scan file for attributions")?;

    transaction.commit().await.wrap_internal_err(
        "failed to commit attribution file scan transaction",
    )?;

    DBVersion::clear_cache_ids(&[file.version_id], redis.as_ref())
        .await
        .wrap_internal_err("failed to clear version cache")?;

    Ok(web::Json(scan_summary))
}

/// List project attribution groups.
#[utoipa::path(
	context_path = "/attribution",
	tag = "attribution",
	params(
		("project_id" = ProjectId, Path)
	),
	responses((status = OK, body = inline(Vec<AttributionGroupResponse>)))
)]
#[get("/{project_id}")]
pub async fn list(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<ProjectId>,
) -> Result<web::Json<Vec<AttributionGroupResponse>>, ApiError> {
    let project_id: DBProjectId = path.into_inner().into();
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::VERSION_READ,
    )
    .await?
    .1;
    let requester_is_mod = user.role.is_mod();

    let project = DBProject::get_id(project_id, pool.as_ref(), redis.as_ref())
        .await?
        .ok_or(ApiError::NotFound)?;
    let (team_member, organization_team_member) =
        DBTeamMember::get_for_project_permissions(
            &project.inner,
            user.id.into(),
            pool.as_ref(),
        )
        .await
        .wrap_internal_err("failed to fetch project permissions")?;
    if ProjectPermissions::get_permissions_by_role(
        &user.role,
        &team_member,
        &organization_team_member,
    )
    .is_none()
    {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to read versions for this project"
        )));
    }

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
    .await
    .wrap_internal_err("failed to fetch attribution groups")?;

    let group_ids: Vec<i64> = groups.iter().map(|g| g.id.0).collect();

    let files = if group_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query!(
            r#"
			select
				paf.group_id as "group_id!",
				paf.name as "name!",
				convert_from(paf.sha1, 'UTF8') as "sha1!",
				paf.moderation_external_license_id,
				coalesce(array_agg(distinct aev.id) filter (where aev.id is not null), '{}') as "version_ids!: Vec<i64>"
			from project_attribution_files paf
			left join override_file_sources ofs on ofs.sha1 = paf.sha1
			left join files f on f.id = ofs.file_id
			left join versions v on v.id = f.version_id and v.mod_id = $2
			left join attribution_enforced_versions aev on aev.id = v.id
			where paf.group_id = ANY($1)
			group by paf.group_id, paf.name, paf.sha1, paf.moderation_external_license_id
			"#,
            &group_ids,
            project_id as DBProjectId,
        )
        .fetch_all(pool.as_ref())
        .await
        .wrap_internal_err("failed to fetch attribution group files")?
    };

    let override_files_on_platform = if requester_is_mod {
        sqlx::query!(
            r#"
			SELECT
				ofs.file_path AS "file_path!",
				CONVERT_FROM(ofs.sha1, 'UTF8') AS "sha1!",
				source_file.version_id AS "version_id: DBVersionId",
				platform_file.version_id AS "platform_version_id: DBVersionId",
				platform_version.mod_id AS "platform_project_id: DBProjectId"
			FROM files source_file
			INNER JOIN versions source_version
				ON source_version.id = source_file.version_id
			INNER JOIN override_file_sources ofs
				ON ofs.file_id = source_file.id
			INNER JOIN hashes h
				ON h.algorithm = 'sha1' AND h.hash = ofs.sha1
			INNER JOIN files platform_file
				ON platform_file.id = h.file_id
			INNER JOIN versions platform_version
				ON platform_version.id = platform_file.version_id
			WHERE source_version.mod_id = $1
			"#,
            project_id as DBProjectId,
        )
        .fetch_all(pool.as_ref())
        .await
        .wrap_internal_err(
            "failed to fetch override files already on platform",
        )?
        .into_iter()
        .map(|row| OverrideFileOnPlatformResponse {
            file_path: row.file_path,
            sha1: row.sha1,
            version_id: row.version_id.into(),
            platform_version_id: row.platform_version_id.into(),
            platform_project_id: row.platform_project_id.into(),
        })
        .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let moderation_external_licenses = if requester_is_mod {
        let mut ids: Vec<i64> = files
            .iter()
            .filter_map(|f| f.moderation_external_license_id)
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
            .await
            .wrap_internal_err("failed to fetch moderation external licenses")?
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
        .flat_map(|f| f.version_ids.iter().copied())
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
        .await
        .wrap_internal_err("failed to fetch attribution group versions")?;
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
            .filter(|f| f.group_id == group.id.0)
            .map(|f| AttributionFileResponse {
                name: f.name.clone(),
                sha1: f.sha1.clone(),
                moderation_external_license_id: if requester_is_mod {
                    f.moderation_external_license_id
                } else {
                    None
                },
                moderation_external_license: if requester_is_mod {
                    f.moderation_external_license_id.and_then(|id| {
                        moderation_external_licenses.get(&id).cloned()
                    })
                } else {
                    None
                },
                versions: {
                    let mut versions: Vec<_> = f
                        .version_ids
                        .iter()
                        .copied()
                        .map(|id| VersionId(id as u64))
                        .collect();
                    versions.sort_by_key(|id| {
                        version_order.get(id).copied().unwrap_or(usize::MAX)
                    });
                    versions
                },
            })
            .collect();
        let group_version_ids = group_files
            .iter()
            .flat_map(|file| file.versions.iter().copied())
            .collect::<std::collections::HashSet<_>>();
        let group_versions = version_infos
            .iter()
            .filter(|version| group_version_ids.contains(&version.id))
            .cloned()
            .collect();
        let group_file_sha1s = group_files
            .iter()
            .map(|file| file.sha1.as_str())
            .collect::<std::collections::HashSet<_>>();
        let group_override_files_on_platform = override_files_on_platform
            .iter()
            .filter(|file| group_file_sha1s.contains(file.sha1.as_str()))
            .cloned()
            .collect();

        let mut attribution = group.attribution.and_then(|v| {
            serde_json::from_value::<AttributionResolution>(v).ok()
        });
        if let Some(moderation_status) = attribution
            .as_mut()
            .and_then(|a| a.moderation_status.as_mut())
            && !requester_is_mod
        {
            moderation_status.moderated_by = None;
        }
        let attributed_by = if attribution
            .as_ref()
            .is_some_and(|attribution| attribution.updated_by_moderator)
            && !requester_is_mod
        {
            None
        } else {
            group
                .attributed_by
                .map(|id| ariadne::ids::UserId(id as u64))
        };

        result.push(AttributionGroupResponse {
            id: group.id.into(),
            flame_project: group
                .flame_project
                .and_then(|v| serde_json::from_value(v).ok()),
            attribution,
            attributed_at: group.attributed_at,
            attributed_by,
            files: group_files,
            versions: group_versions,
            override_files_on_platform: group_override_files_on_platform,
        });
    }

    Ok(web::Json(result))
}

#[derive(Deserialize, utoipa::ToSchema)]
struct UpdateGroupBody {
    attribution: AttributionResolution,
}

/// Update an attribution group.
#[utoipa::path(
	context_path = "/attribution",
	tag = "attribution",
	responses((status = NO_CONTENT))
)]
#[patch("/group/{group_id}")]
pub async fn update_group(
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

    let mut attribution = body.attribution;
    attribution.updated_by_moderator = user.role.is_mod();
    if let Some(moderation_status) = &mut attribution.moderation_status {
        moderation_status.moderated_at = Some(Utc::now());
        moderation_status.moderated_by = Some(user.id);
    }

    let result = sqlx::query!(
        "
		update project_attribution_groups
		set attribution = $1, attributed_at = now(), attributed_by = $3
		where id = $2
		",
        &serde_json::to_value(&attribution).unwrap_or_default(),
        group_id,
        user.id.0 as i64,
    )
    .execute(pool.as_ref())
    .await
    .wrap_internal_err("failed to update attribution group")?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    clear_group_version_cache(pool.as_ref(), redis.as_ref(), &[group_id])
        .await?;

    Ok(())
}

/// Delete an attribution group and all files inside it.
#[utoipa::path(
	context_path = "/attribution",
	tag = "attribution",
	responses((status = NO_CONTENT))
)]
#[delete("/group/{group_id}")]
pub async fn delete_group(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<i64>,
) -> Result<(), ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let mut txn = pool.begin().await.wrap_internal_err(
        "failed to begin attribution group deletion transaction",
    )?;

    let group_id = path.into_inner();
    let version_ids = sqlx::query_scalar!(
        r#"
		SELECT DISTINCT f.version_id AS "version_id: DBVersionId"
		FROM project_attribution_files paf
		INNER JOIN project_attribution_groups pag ON pag.id = paf.group_id
		INNER JOIN override_file_sources ofs ON ofs.sha1 = paf.sha1
		INNER JOIN files f ON f.id = ofs.file_id
		INNER JOIN versions v ON v.id = f.version_id
		WHERE paf.group_id = $1
			AND pag.project_id = v.mod_id
		"#,
        group_id,
    )
    .fetch_all(&mut txn)
    .await
    .wrap_internal_err("failed to fetch attribution group versions")?;

    sqlx::query!(
        "
		DELETE FROM project_attribution_files
		WHERE group_id = $1
		",
        group_id,
    )
    .execute(&mut txn)
    .await
    .wrap_internal_err("failed to delete attribution group files")?;

    let result = sqlx::query!(
        "
		DELETE FROM project_attribution_groups
		WHERE id = $1
		",
        group_id,
    )
    .execute(&mut txn)
    .await
    .wrap_internal_err("failed to delete attribution group")?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    txn.commit().await.wrap_internal_err(
        "failed to commit attribution group deletion transaction",
    )?;

    DBVersion::clear_cache_ids(&version_ids, redis.as_ref())
        .await
        .wrap_internal_err("failed to clear version attribution cache")?;

    Ok(())
}

#[derive(Deserialize, utoipa::ToSchema)]
struct AssignBody {
    sha1: String,
    target_group_id: i64,
    project_id: ProjectId,
}

/// Move a file to an attribution group.
#[utoipa::path(
	context_path = "/attribution",
	tag = "attribution",
	responses((status = NO_CONTENT))
)]
#[post("/assign")]
pub async fn assign(
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

    let sha1 = body.sha1.trim().to_lowercase();
    if hex_to_bytes(&sha1).is_none() {
        return Err(ApiError::InvalidInput(
            "invalid sha1 hex string".to_string(),
        ));
    }
    let sha1_bytes = sha1.as_bytes().to_vec();
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
    .await
    .wrap_internal_err("failed to fetch source attribution group")?
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
    .await
    .wrap_internal_err("failed to check target attribution group")?;

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

    let mut txn = pool.begin().await.wrap_internal_err(
        "failed to begin attribution assignment transaction",
    )?;

    let result = sqlx::query!(
        r#"
		insert into project_attribution_files (
			group_id,
			name,
			sha1,
			moderation_external_license_id
		)
		select
			$1,
			paf.name,
			paf.sha1,
			paf.moderation_external_license_id
		from project_attribution_files paf
		inner join project_attribution_groups pag on pag.id = paf.group_id
		where paf.sha1 = $2 and pag.project_id = $3
		order by paf.moderation_external_license_id nulls last, paf.name
		limit 1
		on conflict (group_id, sha1) do update
		set moderation_external_license_id = coalesce(
			project_attribution_files.moderation_external_license_id,
			excluded.moderation_external_license_id
		)
		"#,
        body.target_group_id,
        &sha1_bytes,
        project_id as DBProjectId,
    )
    .execute(&mut txn)
    .await
    .wrap_internal_err("failed to insert assigned attribution file")?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    sqlx::query!(
        r#"
		delete from project_attribution_files paf
		using project_attribution_groups pag
		where pag.id = paf.group_id
			and pag.project_id = $1
			and paf.sha1 = $2
			and paf.group_id != $3
		"#,
        project_id as DBProjectId,
        &sha1_bytes,
        body.target_group_id,
    )
    .execute(&mut txn)
    .await
    .wrap_internal_err("failed to remove old assigned attribution files")?;

    txn.commit().await.wrap_internal_err(
        "failed to commit attribution assignment transaction",
    )?;

    cleanup_empty_groups(pool.as_ref())
        .await
        .wrap_internal_err("failed to clean up empty attribution groups")?;

    clear_project_sha1_version_cache(
        pool.as_ref(),
        redis.as_ref(),
        project_id,
        &sha1_bytes,
    )
    .await?;

    Ok(())
}

#[derive(Deserialize, utoipa::ToSchema)]
struct SplitBody {
    sha1: String,
    project_id: ProjectId,
}

/// Split a file into a new attribution group.
#[utoipa::path(
	context_path = "/attribution",
	tag = "attribution",
	responses((status = NO_CONTENT))
)]
#[post("/split")]
pub async fn split(
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

    let sha1 = body.sha1.trim().to_lowercase();
    if hex_to_bytes(&sha1).is_none() {
        return Err(ApiError::InvalidInput(
            "invalid sha1 hex string".to_string(),
        ));
    }
    let sha1_bytes = sha1.as_bytes().to_vec();
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
    .await
    .wrap_internal_err("failed to fetch attribution file to split")?;

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

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin attribution split transaction")?;

    let new_group_id = generate_attribution_group_id(&mut txn)
        .await
        .wrap_internal_err("failed to generate attribution group id")?;

    sqlx::query!(
        "
		insert into project_attribution_groups (id, project_id)
		values ($1, $2)
		",
        new_group_id as DBAttributionGroupId,
        project_id as DBProjectId,
    )
    .execute(&mut txn)
    .await
    .wrap_internal_err("failed to insert split attribution group")?;

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
    .await
    .wrap_internal_err("failed to move attribution file to split group")?;

    txn.commit()
        .await
        .wrap_internal_err("failed to commit attribution split transaction")?;

    cleanup_empty_groups(pool.as_ref())
        .await
        .wrap_internal_err("failed to clean up empty attribution groups")?;

    clear_project_sha1_version_cache(
        pool.as_ref(),
        redis.as_ref(),
        project_id,
        &sha1_bytes,
    )
    .await?;

    Ok(())
}

async fn clear_group_version_cache(
    pool: &PgPool,
    redis: &RedisPool,
    group_ids: &[i64],
) -> Result<(), ApiError> {
    let version_ids = sqlx::query_scalar!(
        r#"
		select distinct f.version_id as "version_id: DBVersionId"
		from project_attribution_files paf
		inner join project_attribution_groups pag on pag.id = paf.group_id
		inner join override_file_sources ofs on ofs.sha1 = paf.sha1
		inner join files f on f.id = ofs.file_id
		inner join versions v on v.id = f.version_id
		where paf.group_id = any($1)
			and pag.project_id = v.mod_id
		"#,
        group_ids,
    )
    .fetch_all(pool)
    .await
    .wrap_internal_err("failed to fetch attribution group versions")?;

    DBVersion::clear_cache_ids(&version_ids, redis)
        .await
        .wrap_internal_err("failed to clear version attribution cache")?;

    Ok(())
}

async fn clear_project_sha1_version_cache(
    pool: &PgPool,
    redis: &RedisPool,
    project_id: DBProjectId,
    sha1: &[u8],
) -> Result<(), ApiError> {
    let version_ids = sqlx::query_scalar!(
        r#"
		select distinct f.version_id as "version_id: DBVersionId"
		from override_file_sources ofs
		inner join files f on f.id = ofs.file_id
		inner join versions v on v.id = f.version_id
		where ofs.sha1 = $1
			and v.mod_id = $2
		"#,
        sha1,
        project_id as DBProjectId,
    )
    .fetch_all(pool)
    .await
    .wrap_internal_err("failed to fetch attribution file versions")?;

    DBVersion::clear_cache_ids(&version_ids, redis)
        .await
        .wrap_internal_err("failed to clear version attribution cache")?;

    Ok(())
}

async fn can_edit_attribution_group(
    pool: &PgPool,
    group_id: i64,
    user: &User,
) -> Result<bool, ApiError> {
    let group = sqlx::query!(
        r#"
		select attribution, project_id as "project_id: DBProjectId"
		from project_attribution_groups
		where id = $1
		"#,
        group_id,
    )
    .fetch_optional(pool)
    .await
    .wrap_internal_err("failed to fetch attribution group")?
    .ok_or(ApiError::NotFound)?;

    ensure_can_upload_versions_to_project(
        pool,
        group.project_id,
        user,
        "you do not have permission to edit this attribution group",
    )
    .await?;

    let attribution: Option<AttributionResolution> = group
        .attribution
        .and_then(|value| serde_json::from_value(value).ok());

    Ok(!matches!(
        attribution
            .and_then(|attribution| attribution.moderation_status)
            .map(|status| status.kind),
        Some(AttributionModerationStatusKind::NotAllowed)
    ))
}

async fn ensure_can_upload_versions_to_project(
    pool: &PgPool,
    project_id: DBProjectId,
    user: &User,
    permission_error: &'static str,
) -> Result<(), ApiError> {
    if user.role.is_mod() {
        return Ok(());
    }

    let team_member = DBTeamMember::get_from_user_id_project(
        project_id,
        user.id.into(),
        false,
        pool,
    )
    .await
    .wrap_internal_err("failed to fetch project team member")?;

    let organization = DBOrganization::get_associated_organization_project_id(
        project_id, pool,
    )
    .await
    .wrap_internal_err("failed to fetch associated organization")?;

    let organization_team_member = if let Some(organization) = organization {
        DBTeamMember::get_from_user_id(
            organization.team_id,
            user.id.into(),
            pool,
        )
        .await
        .wrap_internal_err("failed to fetch organization team member")?
    } else {
        None
    };

    let permissions = ProjectPermissions::get_permissions_by_role(
        &user.role,
        &team_member,
        &organization_team_member,
    )
    .unwrap_or_default();

    if !permissions.contains(ProjectPermissions::UPLOAD_VERSION) {
        return Err(ApiError::Auth(eyre!(permission_error)));
    }

    Ok(())
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
    .await
    .wrap_internal_err("failed to delete empty attribution groups")?;
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
