use crate::auth::checks::{is_visible_project, is_visible_version};
use crate::database::models::legacy_loader_fields::MinecraftGameVersion;
use crate::database::models::loader_fields::Loader;
use crate::database::models::project_item::ProjectQueryResult;
use crate::database::models::version_item::{
    FileQueryResult, VersionQueryResult,
};
use crate::database::redis::RedisPool;
use crate::models::ids::{ProjectId, VersionId};
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::{auth::get_user_from_headers, database};
use actix_web::{HttpRequest, HttpResponse, get, route, web};
use sqlx::PgPool;
use std::collections::HashSet;
use yaserde::YaSerialize;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(maven_metadata);
    cfg.service(version_file_sha512);
    cfg.service(version_file_sha1);
    cfg.service(version_file);
}

#[derive(Default, Debug, Clone, YaSerialize)]
#[yaserde(rename = "metadata")]
pub struct Metadata {
    #[yaserde(rename = "groupId")]
    group_id: String,
    #[yaserde(rename = "artifactId")]
    artifact_id: String,
    versioning: Versioning,
}

#[derive(Default, Debug, Clone, YaSerialize)]
#[yaserde(rename = "versioning")]
pub struct Versioning {
    latest: String,
    release: String,
    versions: Versions,
    #[yaserde(rename = "lastUpdated")]
    last_updated: String,
}

#[derive(Default, Debug, Clone, YaSerialize)]
#[yaserde(rename = "versions")]
pub struct Versions {
    #[yaserde(rename = "version")]
    versions: Vec<String>,
}

#[derive(Default, Debug, Clone, YaSerialize)]
#[yaserde(rename = "project", namespaces = { "" = "http://maven.apache.org/POM/4.0.0" })]
pub struct MavenPom {
    #[yaserde(rename = "xsi:schemaLocation", attribute = true)]
    schema_location: String,
    #[yaserde(rename = "xmlns:xsi", attribute = true)]
    xsi: String,
    #[yaserde(rename = "modelVersion")]
    model_version: String,
    #[yaserde(rename = "groupId")]
    group_id: String,
    #[yaserde(rename = "artifactId")]
    artifact_id: String,
    version: String,
    name: String,
    description: String,
}

#[get("maven/modrinth/{id}/maven-metadata.xml")]
pub async fn maven_metadata(
    req: HttpRequest,
    params: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let project_id = params.into_inner().0;
    let Some(project) =
        database::models::DBProject::get(&project_id, &**pool, &redis).await?
    else {
        return Err(ApiError::NotFound);
    };

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

    if !is_visible_project(&project.inner, &user_option, &pool, false).await? {
        return Err(ApiError::NotFound);
    }

    let version_names = sqlx::query!(
        "
        SELECT id, version_number, version_type
        FROM versions
        WHERE mod_id = $1 AND status = ANY($2)
        ORDER BY ordering ASC NULLS LAST, date_published ASC
        ",
        project.inner.id as database::models::ids::DBProjectId,
        &*crate::models::projects::VersionStatus::iterator()
            .filter(|x| x.is_listed())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch_all(&**pool)
    .await?;

    let mut new_versions = Vec::new();
    let mut vals = HashSet::new();
    let mut latest_release = None;

    for row in version_names {
        let value = if vals.contains(&row.version_number) {
            format!("{}", VersionId(row.id as u64))
        } else {
            row.version_number
        };

        vals.insert(value.clone());
        if row.version_type == "release" {
            latest_release = Some(value.clone())
        }

        new_versions.push(value);
    }

    let project_id: ProjectId = project.inner.id.into();

    let respdata = Metadata {
        group_id: "maven.modrinth".to_string(),
        artifact_id: project_id.to_string(),
        versioning: Versioning {
            latest: new_versions
                .last()
                .unwrap_or(&"release".to_string())
                .to_string(),
            release: latest_release.unwrap_or_default(),
            versions: Versions {
                versions: new_versions,
            },
            last_updated: project
                .inner
                .updated
                .format("%Y%m%d%H%M%S")
                .to_string(),
        },
    };

    Ok(HttpResponse::Ok()
        .content_type("text/xml")
        .body(yaserde::ser::to_string(&respdata).map_err(ApiError::Xml)?))
}

async fn find_version(
    project: &ProjectQueryResult,
    vcoords: &String,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Option<VersionQueryResult>, ApiError> {
    let id_option = ariadne::ids::base62_impl::parse_base62(vcoords)
        .ok()
        .map(|x| x as i64);

    let all_versions =
        database::models::DBVersion::get_many(&project.versions, pool, redis)
            .await?;

    let exact_matches = all_versions
        .iter()
        .filter(|x| {
            &x.inner.version_number == vcoords
                || Some(x.inner.id.0) == id_option
        })
        .collect::<Vec<_>>();

    if exact_matches.len() == 1 {
        return Ok(Some(exact_matches[0].clone()));
    }

    // Try to parse version filters from version coords.
    let Some((vnumber, filter)) = vcoords.rsplit_once('-') else {
        return Ok(exact_matches.first().map(|x| (*x).clone()));
    };

    let db_loaders: HashSet<String> = Loader::list(pool, redis)
        .await?
        .into_iter()
        .map(|x| x.loader)
        .collect();

    let (loaders, game_versions) = filter
        .split(',')
        .map(String::from)
        .partition::<Vec<_>, _>(|el| db_loaders.contains(el));

    let matched = all_versions
        .iter()
        .filter(|x| {
            let mut bool = x.inner.version_number == vnumber;

            if !loaders.is_empty() {
                bool &= x.loaders.iter().any(|y| loaders.contains(y));
            }

            // For maven in particular, we will hardcode it to use GameVersions rather than generic loader fields, as this is minecraft-java exclusive
            if !game_versions.is_empty() {
                let version_game_versions =
                    x.version_fields.clone().into_iter().find_map(|v| {
                        MinecraftGameVersion::try_from_version_field(&v).ok()
                    });
                if let Some(version_game_versions) = version_game_versions {
                    bool &= version_game_versions
                        .iter()
                        .any(|y| game_versions.contains(&y.version));
                }
            }

            bool
        })
        .collect::<Vec<_>>();

    Ok(matched
        .first()
        .or_else(|| exact_matches.first())
        .copied()
        .cloned())
}

fn find_file<'a>(
    project_id: &str,
    vcoords: &str,
    version: &'a VersionQueryResult,
    file: &str,
) -> Option<&'a FileQueryResult> {
    if let Some(selected_file) =
        version.files.iter().find(|x| x.filename == file)
    {
        return Some(selected_file);
    }

    // Minecraft mods are not going to be both a mod and a modpack, so this minecraft-specific handling is fine
    // As there can be multiple project types, returns the first allowable match
    let mut fileexts = vec![];
    for project_type in version.project_types.iter() {
        match project_type.as_str() {
            "mod" => fileexts.push("jar"),
            "modpack" => fileexts.push("mrpack"),
            _ => (),
        }
    }

    for fileext in fileexts {
        if file == format!("{}-{}.{}", &project_id, &vcoords, fileext) {
            return version
                .files
                .iter()
                .find(|x| x.primary)
                .or_else(|| version.files.iter().last());
        }
    }
    None
}

#[route(
    "maven/modrinth/{id}/{versionnum}/{file}",
    method = "GET",
    method = "HEAD"
)]
pub async fn version_file(
    req: HttpRequest,
    params: web::Path<(String, String, String)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (project_id, vnum, file) = params.into_inner();
    let Some(project) =
        database::models::DBProject::get(&project_id, &**pool, &redis).await?
    else {
        return Err(ApiError::NotFound);
    };

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

    if !is_visible_project(&project.inner, &user_option, &pool, false).await? {
        return Err(ApiError::NotFound);
    }

    let Some(version) = find_version(&project, &vnum, &pool, &redis).await?
    else {
        return Err(ApiError::NotFound);
    };

    if !is_visible_version(&version.inner, &user_option, &pool, &redis).await? {
        return Err(ApiError::NotFound);
    }

    if file == format!("{}-{}.pom", &project_id, &vnum) {
        let respdata = MavenPom {
            schema_location:
                "http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd"
                    .to_string(),
            xsi: "http://www.w3.org/2001/XMLSchema-instance".to_string(),
            model_version: "4.0.0".to_string(),
            group_id: "maven.modrinth".to_string(),
            artifact_id: project_id,
            version: vnum,
            name: project.inner.name,
            description: project.inner.description,
        };
        return Ok(HttpResponse::Ok()
            .content_type("text/xml")
            .body(yaserde::ser::to_string(&respdata).map_err(ApiError::Xml)?));
    } else if let Some(selected_file) =
        find_file(&project_id, &vnum, &version, &file)
    {
        return Ok(HttpResponse::TemporaryRedirect()
            .append_header(("location", &*selected_file.url))
            .body(""));
    }

    Err(ApiError::NotFound)
}

#[get("maven/modrinth/{id}/{versionnum}/{file}.sha1")]
pub async fn version_file_sha1(
    req: HttpRequest,
    params: web::Path<(String, String, String)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (project_id, vnum, file) = params.into_inner();
    let Some(project) =
        database::models::DBProject::get(&project_id, &**pool, &redis).await?
    else {
        return Err(ApiError::NotFound);
    };

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

    if !is_visible_project(&project.inner, &user_option, &pool, false).await? {
        return Err(ApiError::NotFound);
    }

    let Some(version) = find_version(&project, &vnum, &pool, &redis).await?
    else {
        return Err(ApiError::NotFound);
    };

    if !is_visible_version(&version.inner, &user_option, &pool, &redis).await? {
        return Err(ApiError::NotFound);
    }

    Ok(find_file(&project_id, &vnum, &version, &file)
        .and_then(|file| file.hashes.get("sha1"))
        .map(|hash_str| HttpResponse::Ok().body(hash_str.clone()))
        .unwrap_or_else(|| HttpResponse::NotFound().body("")))
}

#[get("maven/modrinth/{id}/{versionnum}/{file}.sha512")]
pub async fn version_file_sha512(
    req: HttpRequest,
    params: web::Path<(String, String, String)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (project_id, vnum, file) = params.into_inner();
    let Some(project) =
        database::models::DBProject::get(&project_id, &**pool, &redis).await?
    else {
        return Err(ApiError::NotFound);
    };

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

    if !is_visible_project(&project.inner, &user_option, &pool, false).await? {
        return Err(ApiError::NotFound);
    }

    let Some(version) = find_version(&project, &vnum, &pool, &redis).await?
    else {
        return Err(ApiError::NotFound);
    };

    if !is_visible_version(&version.inner, &user_option, &pool, &redis).await? {
        return Err(ApiError::NotFound);
    }

    Ok(find_file(&project_id, &vnum, &version, &file)
        .and_then(|file| file.hashes.get("sha512"))
        .map(|hash_str| HttpResponse::Ok().body(hash_str.clone()))
        .unwrap_or_else(|| HttpResponse::NotFound().body("")))
}
