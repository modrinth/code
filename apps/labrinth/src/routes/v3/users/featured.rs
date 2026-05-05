use crate::auth::checks::filter_visible_projects;
use crate::auth::get_user_from_headers;
use crate::database::models::featured_project_item::DBFeaturedProject;
use crate::database::models::project_item;
use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::models::ids::{ModId, UserId};
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{web, HttpRequest, HttpResponse};

pub fn config(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::scope("{user_id}/featured")
			.route("", web::get().to(get_user_featured_projects))
			.route("{mod_id}", web::post().to(toggle_featured_project)),
	);
}

#[utoipa::path(
	get,
	path = "/v3/user/{user_id}/featured",
	params(
		("user_id" = String, Path, description = "The ID of the user"),
	),
	responses(
		(status = 200, description = "Success", body = Vec<Project>),
		(status = 404, description = "The requested user was not found"),
	),
	tag = "users"
)]
pub async fn get_user_featured_projects(
	req: HttpRequest,
	info: web::Path<String>,
	pool: web::Data<PgPool>,
	redis: web::Data<RedisPool>,
	session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
	let user_id_str = info.into_inner();
	let user_id = UserId::parse(&user_id_str)?;

	let featured_ids = DBFeaturedProject::get_user_featured(user_id, &pool, &redis).await?;

	if featured_ids.is_empty() {
		return Ok(HttpResponse::Ok().json(vec![] as Vec<()>));
	}

	let projects = project_item::DBProject::get_many_ids(&featured_ids, &**pool, &redis).await?;

	let user = get_user_from_headers(&req, &**pool, &redis, &session_queue, Scopes::PROJECT_READ)
		.await
		.map(|x| x.1)
		.ok();

	let projects = filter_visible_projects(projects, &user, &**pool, false)
		.await?
		.into_iter()
		.map(|p| p.into())
		.collect::<Vec<_>>();

	Ok(HttpResponse::Ok().json(projects))
}

#[utoipa::path(
	post,
	path = "/v3/user/{user_id}/featured/{mod_id}",
	params(
		("user_id" = String, Path, description = "The ID of the user"),
		("mod_id" = String, Path, description = "The ID of the project to toggle"),
	),
	responses(
		(status = 204, description = "Success"),
		(status = 401, description = "Not authorized to modify this user's featured projects"),
		(status = 404, description = "The requested user or project was not found"),
	),
	tag = "users"
)]
pub async fn toggle_featured_project(
	req: HttpRequest,
	info: web::Path<(String, String)>,
	pool: web::Data<PgPool>,
	redis: web::Data<RedisPool>,
	session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
	let (user_id_str, mod_id_str) = info.into_inner();
	let user_id = UserId::parse(&user_id_str)?;
	let mod_id = ModId::parse(&mod_id_str)?;

	let current_user = get_user_from_headers(&req, &**pool, &redis, &session_queue, Scopes::PROJECT_WRITE)
		.await?
		.1;

	if current_user.id != user_id {
		return Err(ApiError::Unauthorized);
	}

	DBFeaturedProject::toggle_featured(user_id, mod_id, &pool, &redis).await?;

	Ok(HttpResponse::NoContent().finish())
}
