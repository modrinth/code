use actix_web::{HttpRequest, get, web};
use eyre::eyre;
use serde::Serialize;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::{PgPool, ReadOnlyPgPool, models as db_models};
use crate::models::pats::Scopes;
use crate::models::projects::{Project, Version};
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context as _;
use crate::validate::project::{
	ProjectNag, validate_with_context as validate_project,
};

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProjectValidationResponse {
	pub nags: Vec<ProjectNag>,
}

/// Validate that a project is ready to be submitted for review.
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
	responses((status = OK, body = ProjectValidationResponse))
)]
#[get("/{id}/validate")]
pub async fn validate(
	req: HttpRequest,
	info: web::Path<(String,)>,
	pool: web::Data<PgPool>,
	ro_pool: web::Data<ReadOnlyPgPool>,
	redis: web::Data<RedisPool>,
	session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectValidationResponse>, ApiError> {
	let user = get_user_from_headers(
		&req,
		&**pool,
		&redis,
		&session_queue,
		Scopes::PROJECT_READ,
	)
	.await
	.wrap_auth_err("authenticating API request")?
	.1;

	let project =
		db_models::DBProject::get(&info.into_inner().0, &***ro_pool, &redis)
			.await
			.wrap_internal_err("fetching project from database")?
			.wrap_not_found_err("resource not found")?;

	let (team_member, organization_team_member) =
		db_models::DBTeamMember::get_for_project_permissions(
			&project.inner,
			user.id.into(),
			&***ro_pool,
		)
		.await
		.wrap_internal_err("fetching project permissions")?;

	if ProjectPermissions::get_permissions_by_role(
		&user.role,
		&team_member,
		&organization_team_member,
	)
	.is_none()
	{
		return Err(ApiError::Auth(eyre!(
			"you do not have permission to validate this project"
		)));
	}

	let versions =
		db_models::DBVersion::get_many(&project.versions, &***ro_pool, &redis)
			.await
			.wrap_internal_err("fetching project versions from database")?
			.into_iter()
			.map(Version::from)
			.collect::<Vec<_>>();
	let available_categories =
		db_models::categories::Category::list(&**pool, &redis)
			.await
			.wrap_internal_err("fetching project categories")?;
	let disclosures = db_models::DBProjectDisclosure::get_many_for_project(
		project.inner.id,
		false,
		&***ro_pool,
	)
	.await
	.wrap_internal_err("fetching project disclosures")?
	.into_iter()
	.map(|disclosure| disclosure.disclosure)
	.collect::<Vec<_>>();
	let project = Project::from(project);

	Ok(web::Json(ProjectValidationResponse {
		nags: validate_project(
			&project,
			&versions,
			&available_categories,
			&disclosures,
		),
	}))
}
