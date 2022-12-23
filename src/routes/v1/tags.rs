use crate::database::models::categories::{Category, GameVersion, Loader};
use crate::routes::ApiError;
use actix_web::{get, web, HttpResponse};
use sqlx::PgPool;

#[get("category")]
pub async fn category_list(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let results = Category::list(&**pool)
        .await?
        .into_iter()
        .filter(|x| &*x.project_type == "mod")
        .map(|x| x.category)
        .collect::<Vec<String>>();
    Ok(HttpResponse::Ok().json(results))
}

#[get("loader")]
pub async fn loader_list(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let results = Loader::list(&**pool)
        .await?
        .into_iter()
        .filter(|x| x.supported_project_types.contains(&"mod".to_string()))
        .map(|x| x.loader)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(results))
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
        let results = GameVersion::list_filter(
            query.type_.as_deref(),
            query.major,
            &**pool,
        )
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
