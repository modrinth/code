use std::collections::HashMap;

use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::{filter_authorized_versions, get_user_from_headers, is_authorized};
use crate::database;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::models::projects::VersionType;
use crate::queue::session::AuthQueue;

use super::ApiError;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(forge_updates);
}

#[derive(Serialize, Deserialize)]
pub struct NeoForge {
    #[serde(default = "default_neoforge")]
    pub neoforge: String,
}

fn default_neoforge() -> String {
    "none".into()
}

#[get("{id}/forge_updates.json")]
pub async fn forge_updates(
    req: HttpRequest,
    web::Query(neo): web::Query<NeoForge>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    const ERROR: &str = "The specified project does not exist!";

    let (id,) = info.into_inner();

    let project = database::models::Project::get(&id, &**pool, &redis)
        .await?
        .ok_or_else(|| ApiError::InvalidInput(ERROR.to_string()))?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    if !is_authorized(&project.inner, &user_option, &pool).await? {
        return Err(ApiError::InvalidInput(ERROR.to_string()));
    }

    let versions = database::models::Version::get_many(&project.versions, &**pool, &redis).await?;

    let loaders = match &*neo.neoforge {
        "only" => |x: &String| *x == "neoforge",
        "include" => |x: &String| *x == "forge" || *x == "neoforge",
        _ => |x: &String| *x == "forge",
    };

    let mut versions = filter_authorized_versions(
        versions
            .into_iter()
            .filter(|x| x.loaders.iter().any(loaders))
            .collect(),
        &user_option,
        &pool,
    )
    .await?;

    versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));

    #[derive(Serialize)]
    struct ForgeUpdates {
        homepage: String,
        promos: HashMap<String, String>,
    }

    let mut response = ForgeUpdates {
        homepage: format!(
            "{}/mod/{}",
            dotenvy::var("SITE_URL").unwrap_or_default(),
            id
        ),
        promos: HashMap::new(),
    };

    for version in versions {
        if version.version_type == VersionType::Release {
            for game_version in &version.game_versions {
                response
                    .promos
                    .entry(format!("{}-recommended", game_version.0))
                    .or_insert_with(|| version.version_number.clone());
            }
        }

        for game_version in &version.game_versions {
            response
                .promos
                .entry(format!("{}-latest", game_version.0))
                .or_insert_with(|| version.version_number.clone());
        }
    }

    Ok(HttpResponse::Ok().json(response))
}
