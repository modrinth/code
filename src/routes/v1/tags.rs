use crate::auth::check_is_admin_from_headers;
use crate::database::models::categories::{Category, GameVersion, Loader, ProjectType};
use crate::routes::ApiError;
use actix_web::{get, put, web};
use actix_web::{HttpRequest, HttpResponse};
use sqlx::PgPool;

const DEFAULT_ICON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>"#;

#[get("category")]
pub async fn category_list(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let results = Category::list(&**pool)
        .await?
        .into_iter()
        .filter(|x| &*x.project_type == "mod")
        .map(|x| x.category)
        .collect::<Vec<String>>();
    Ok(HttpResponse::Ok().json(results))
}

#[put("category/{name}")]
pub async fn category_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    category: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = category.into_inner().0;

    let project_type = crate::database::models::ProjectTypeId::get_id("mod".to_string(), &**pool)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInputError("Specified project type does not exist!".to_string())
        })?;

    let _id = Category::builder()
        .name(&name)?
        .icon(DEFAULT_ICON)?
        .project_type(&project_type)?
        .insert(&**pool)
        .await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[get("loader")]
pub async fn loader_list(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let results = Loader::list(&**pool)
        .await?
        .into_iter()
        .filter(|x| x.supported_project_types.contains(&"mod".to_string()))
        .map(|x| x.loader)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(results))
}

#[put("loader/{name}")]
pub async fn loader_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    loader: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = loader.into_inner().0;
    let mut transaction = pool.begin().await?;

    let project_types = ProjectType::get_many_id(&["mod".to_string()], &mut *transaction).await?;

    let _id = Loader::builder()
        .name(&name)?
        .icon(DEFAULT_ICON)?
        .supported_project_types(&*project_types.into_iter().map(|x| x.id).collect::<Vec<_>>())?
        .insert(&mut transaction)
        .await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(serde::Deserialize)]
pub struct GameVersionQueryData {
    #[serde(rename = "type")]
    type_: Option<String>,
    major: Option<bool>,
}

#[get("game_version")]
pub async fn game_version_list(
    pool: web::Data<PgPool>,
    query: web::Query<GameVersionQueryData>,
) -> Result<HttpResponse, ApiError> {
    if query.type_.is_some() || query.major.is_some() {
        let results = GameVersion::list_filter(query.type_.as_deref(), query.major, &**pool)
            .await?
            .into_iter()
            .map(|x| x.version)
            .collect::<Vec<String>>();
        Ok(HttpResponse::Ok().json(results))
    } else {
        let results = GameVersion::list(&**pool)
            .await?
            .into_iter()
            .map(|x| x.version)
            .collect::<Vec<String>>();
        Ok(HttpResponse::Ok().json(results))
    }
}
