use crate::database::PgPool;
use crate::routes::ApiError;
use crate::util::error::Context as _;
use actix_web::{HttpResponse, get, web};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_stats_route);
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct V3Stats {
    pub projects: Option<i64>,
    pub versions: Option<i64>,
    pub authors: Option<i64>,
    pub files: Option<i64>,
}

#[utoipa::path(tag = "statistics", responses((status = OK)))]
#[get("/statistics")]
pub async fn get_stats_route(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    get_stats(pool).await
}

pub async fn get_stats(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let projects = sqlx::query!(
        "
        SELECT COUNT(id)
        FROM mods
        WHERE status = ANY($1)
        ",
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| x.is_searchable())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch_one(&**pool)
    .await
    .wrap_internal_err("counting projects")?;

    let versions = sqlx::query!(
        "
        SELECT COUNT(v.id)
        FROM versions v
        INNER JOIN mods m on v.mod_id = m.id AND m.status = ANY($1)
        WHERE v.status = ANY($2)
        ",
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| x.is_searchable())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        &*crate::models::projects::VersionStatus::iterator()
            .filter(|x| x.is_listed())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch_one(&**pool)
    .await
    .wrap_internal_err("counting versions")?;

    let authors = sqlx::query!(
        "
        SELECT COUNT(DISTINCT u.id)
        FROM users u
        INNER JOIN team_members tm on u.id = tm.user_id AND tm.accepted = TRUE
        INNER JOIN mods m on tm.team_id = m.team_id AND m.status = ANY($1)
        ",
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| x.is_searchable())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch_one(&**pool)
    .await
    .wrap_internal_err("counting project authors")?;

    let files = sqlx::query!(
        "
        SELECT COUNT(f.id) FROM files f
        INNER JOIN versions v on f.version_id = v.id AND v.status = ANY($2)
        INNER JOIN mods m on v.mod_id = m.id AND m.status = ANY($1)
        ",
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| x.is_searchable())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        &*crate::models::projects::VersionStatus::iterator()
            .filter(|x| x.is_listed())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch_one(&**pool)
    .await
    .wrap_internal_err("counting version files")?;

    let v3_stats = V3Stats {
        projects: projects.count,
        versions: versions.count,
        authors: authors.count,
        files: files.count,
    };

    Ok(HttpResponse::Ok().json(v3_stats))
}
