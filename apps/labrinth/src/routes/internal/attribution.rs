use actix_web::{HttpRequest, get, patch, post, web};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::auth::get_user_from_headers;
use crate::database::models::ids::{
	DBAttributionGroupId, DBProjectId, generate_attribution_group_id,
};
use crate::database::redis::RedisPool;
use crate::database::PgPool;
use crate::models::ids::{ProjectId, VersionId};
use crate::models::pats::Scopes;
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

#[derive(Serialize)]
struct AttributionGroupResponse {
	id: crate::models::ids::AttributionGroupId,
	flame_project_id: Option<i64>,
	flame_project_title: Option<String>,
	attribution: Option<serde_json::Value>,
	attributed_at: Option<chrono::DateTime<chrono::Utc>>,
	attributed_by: Option<ariadne::ids::UserId>,
	files: Vec<AttributionFileResponse>,
	versions: std::collections::HashMap<VersionId, VersionInfo>,
}

#[derive(Clone, Serialize)]
struct VersionInfo {
	name: String,
	version_number: String,
}

#[derive(Serialize)]
struct AttributionFileResponse {
	name: String,
	sha1: String,
	versions: Vec<VersionId>,
}

#[utoipa::path]
#[get("{project_id}")]
async fn list(
	pool: web::Data<PgPool>,
	path: web::Path<ProjectId>,
) -> Result<web::Json<Vec<AttributionGroupResponse>>, ApiError> {
	let project_id: DBProjectId = path.into_inner().into();

	let groups = sqlx::query!(
		r#"
		select
			g.id as "id: DBAttributionGroupId",
			g.flame_project_id,
			g.flame_project_title,
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
			select paf.group_id, paf.name, encode(paf.sha1, 'escape') as sha1,
				array_agg(distinct f.version_id) as version_ids
			from project_attribution_files paf
			left join override_file_sources ofs on ofs.sha1 = paf.sha1
			left join files f on f.id = ofs.file_id
			where paf.group_id = ANY($1)
			group by paf.group_id, paf.name, paf.sha1
			",
		)
		.bind(&group_ids)
		.fetch_all(pool.as_ref())
		.await?
	};

	let mut all_version_ids: Vec<i64> = files
		.iter()
		.filter_map(|f| f.get::<Option<Vec<i64>>, _>("version_ids"))
		.flatten()
		.collect();
	all_version_ids.sort_unstable();
	all_version_ids.dedup();

	let versions = if all_version_ids.is_empty() {
		std::collections::HashMap::new()
	} else {
		let rows = sqlx::query!(
			"
			select id, name, version_number
			from versions
			where id = ANY($1)
			",
			&all_version_ids,
		)
		.fetch_all(pool.as_ref())
		.await?;
		rows.into_iter()
			.map(|v| {
				(
					VersionId(v.id as u64),
					VersionInfo {
						name: v.name,
						version_number: v.version_number,
					},
				)
			})
			.collect()
	};

	let mut result = Vec::new();
	for group in groups {
		let group_files: Vec<AttributionFileResponse> = files
			.iter()
			.filter(|f| f.get::<i64, _>("group_id") == group.id.0)
			.map(|f| AttributionFileResponse {
				name: f.get("name"),
				sha1: f.get("sha1"),
				versions: f
					.get::<Option<Vec<i64>>, _>("version_ids")
					.unwrap_or_default()
					.into_iter()
					.map(|id| VersionId(id as u64))
					.collect(),
			})
			.collect();

		result.push(AttributionGroupResponse {
			id: group.id.into(),
			flame_project_id: group.flame_project_id,
			flame_project_title: group.flame_project_title,
			attribution: group.attribution,
			attributed_at: group.attributed_at,
			attributed_by: group.attributed_by.map(|id| ariadne::ids::UserId(id as u64)),
			files: group_files,
			versions: versions.clone(),
		});
	}

	Ok(web::Json(result))
}

#[derive(Deserialize, utoipa::ToSchema)]
struct UpdateGroupBody {
	attribution: serde_json::Value,
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

	let result = sqlx::query!(
		"
		update project_attribution_groups
		set attribution = $1, attributed_at = now(), attributed_by = $3
		where id = $2
		",
		&body.attribution,
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
	pool: web::Data<PgPool>,
	web::Json(body): web::Json<AssignBody>,
) -> Result<(), ApiError> {
	let sha1_bytes = hex_to_bytes(&body.sha1).ok_or_else(|| {
		ApiError::InvalidInput("invalid sha1 hex string".to_string())
	})?;
	let project_id: DBProjectId = body.project_id.into();

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
	pool: web::Data<PgPool>,
	web::Json(body): web::Json<SplitBody>,
) -> Result<(), ApiError> {
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
