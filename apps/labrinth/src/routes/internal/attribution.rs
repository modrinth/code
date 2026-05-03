use actix_web::{get, patch, post, web};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::database::models::ids::{
	DBAttributionGroupId, DBProjectId, generate_attribution_group_id,
};
use crate::database::PgPool;
use crate::models::ids::ProjectId;
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
	files: Vec<AttributionFileResponse>,
}

#[derive(Serialize)]
struct AttributionFileResponse {
	name: String,
	sha1: String,
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
			g.attribution
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
			select group_id, name, encode(sha1, 'escape') as sha1
			from project_attribution_files
			where group_id = ANY($1)
			",
		)
		.bind(&group_ids)
		.fetch_all(pool.as_ref())
		.await?
	};

	let mut result = Vec::new();
	for group in groups {
		let group_files: Vec<AttributionFileResponse> = files
			.iter()
			.filter(|f| f.get::<i64, _>("group_id") == group.id.0)
			.map(|f| AttributionFileResponse {
				name: f.get("name"),
				sha1: f.get("sha1"),
			})
			.collect();

		result.push(AttributionGroupResponse {
			id: group.id.into(),
			flame_project_id: group.flame_project_id,
			flame_project_title: group.flame_project_title,
			attribution: group.attribution,
			files: group_files,
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
	pool: web::Data<PgPool>,
	path: web::Path<i64>,
	web::Json(body): web::Json<UpdateGroupBody>,
) -> Result<(), ApiError> {
	let group_id = path.into_inner();
	let result = sqlx::query!(
		"
		update project_attribution_groups
		set attribution = $1
		where id = $2
		",
		&body.attribution,
		group_id,
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

	let result = sqlx::query!(
		"
		update project_attribution_files
		set group_id = $1
		where sha1 = $2
		",
		body.target_group_id,
		&sha1_bytes,
	)
	.execute(pool.as_ref())
	.await?;

	if result.rows_affected() == 0 {
		return Err(ApiError::NotFound);
	}

	Ok(())
}

#[derive(Deserialize, utoipa::ToSchema)]
struct SplitBody {
	sha1: String,
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

	let existing = sqlx::query!(
		"
		select group_id, name from project_attribution_files
		where sha1 = $1
		",
		&sha1_bytes,
	)
	.fetch_optional(pool.as_ref())
	.await?;

	let Some(existing) = existing else {
		return Err(ApiError::NotFound);
	};

	let mut txn = pool.begin().await?;

	let project_id: DBProjectId = DBProjectId(
		sqlx::query_scalar!(
			"
			select project_id from project_attribution_groups
			where id = $1
			",
			existing.group_id,
		)
		.fetch_one(&mut txn)
		.await?,
	);

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
		where sha1 = $2
		",
		new_group_id as DBAttributionGroupId,
		&sha1_bytes,
	)
	.execute(&mut txn)
	.await?;

	txn.commit().await?;

	Ok(())
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
	if hex.len() % 2 != 0 {
		return None;
	}
	(0..hex.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
		.collect()
}
