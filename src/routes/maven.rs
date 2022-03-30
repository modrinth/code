use crate::database::models::project_item::QueryProject;
use crate::database::models::version_item::{QueryFile, QueryVersion};
use crate::models::projects::ProjectId;
use crate::routes::ApiError;
use crate::util::auth::get_user_from_headers;
use crate::{database, util::auth::is_authorized};
use actix_web::{get, web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use yaserde_derive::YaSerialize;

#[derive(Default, Debug, Clone, YaSerialize)]
#[yaserde(root = "metadata", rename = "metadata")]
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
#[yaserde(rename = "project", namespace = "http://maven.apache.org/POM/4.0.0")]
pub struct MavenPom {
    #[yaserde(rename = "xsi:schemaLocation", attribute)]
    schema_location: String,
    #[yaserde(rename = "xmlns:xsi", attribute)]
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
) -> Result<HttpResponse, ApiError> {
    let project_id = params.into_inner().0;
    let project_data =
        database::models::Project::get_full_from_slug_or_project_id(
            &*project_id,
            &**pool,
        )
        .await?;

    let data = if let Some(data) = project_data {
        data
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if !is_authorized(&data, &user_option, &pool).await? {
        return Ok(HttpResponse::NotFound().body(""));
    }

    let version_names = sqlx::query!(
        "
            SELECT version_number, version_type
            FROM versions
            WHERE mod_id = $1
            ",
        data.inner.id as database::models::ids::ProjectId
    )
    .fetch_all(&**pool)
    .await?;

    let project_id: ProjectId = data.inner.id.into();

    let respdata = Metadata {
        group_id: "maven.modrinth".to_string(),
        artifact_id: format!("{}", project_id),
        versioning: Versioning {
            latest: version_names
                .last()
                .map_or("release", |x| &x.version_number)
                .to_string(),
            release: version_names
                .iter()
                .rfind(|x| x.version_type == "release")
                .map_or("", |x| &x.version_number)
                .to_string(),
            versions: Versions {
                versions: version_names
                    .iter()
                    .map(|x| x.version_number.clone())
                    .collect::<Vec<_>>(),
            },
            last_updated: data.inner.updated.format("%Y%m%d%H%M%S"),
        },
    };

    Ok(HttpResponse::Ok()
        .content_type("text/xml")
        .body(yaserde::ser::to_string(&respdata).map_err(ApiError::Xml)?))
}

fn find_file<'a>(
    project_id: &str,
    project: &QueryProject,
    version: &'a QueryVersion,
    file: &str,
) -> Option<&'a QueryFile> {
    if let Some(selected_file) =
        version.files.iter().find(|x| x.filename == file)
    {
        return Some(selected_file);
    }

    let fileext = match project.project_type.as_str() {
        "mod" => "jar",
        "modpack" => "mrpack",
        _ => return None,
    };

    if file
        == format!("{}-{}.{}", &project_id, &version.version_number, fileext)
    {
        version
            .files
            .iter()
            .find(|x| x.primary)
            .or_else(|| version.files.iter().last())
    } else {
        None
    }
}

#[get("maven/modrinth/{id}/{versionnum}/{file}")]
pub async fn version_file(
    req: HttpRequest,
    params: web::Path<(String, String, String)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let (project_id, vnum, file) = params.into_inner();
    let project_data =
        database::models::Project::get_full_from_slug_or_project_id(
            &project_id,
            &**pool,
        )
        .await?;

    let project = if let Some(data) = project_data {
        data
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if !is_authorized(&project, &user_option, &pool).await? {
        return Ok(HttpResponse::NotFound().body(""));
    }

    let vid = if let Some(vid) = sqlx::query!(
        "SELECT id FROM versions WHERE mod_id = $1 AND version_number = $2",
        project.inner.id as database::models::ids::ProjectId,
        vnum
    )
    .fetch_optional(&**pool)
    .await?
    {
        vid
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let version = if let Some(version) = database::models::Version::get_full(
        database::models::ids::VersionId(vid.id),
        &**pool,
    )
    .await?
    {
        version
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    if file == format!("{}-{}.pom", &project_id, &version.version_number) {
        let respdata = MavenPom {
            schema_location:
                "http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd"
                    .to_string(),
            xsi: "http://www.w3.org/2001/XMLSchema-instance".to_string(),
            model_version: "4.0.0".to_string(),
            group_id: "maven.modrinth".to_string(),
            artifact_id: project_id,
            version: version.version_number,
            name: project.inner.title,
            description: project.inner.description,
        };
        return Ok(HttpResponse::Ok()
            .content_type("text/xml")
            .body(yaserde::ser::to_string(&respdata).map_err(ApiError::Xml)?));
    } else if let Some(selected_file) =
        find_file(&project_id, &project, &version, &file)
    {
        return Ok(HttpResponse::TemporaryRedirect()
            .append_header(("location", &*selected_file.url))
            .body(""));
    }

    Ok(HttpResponse::NotFound().body(""))
}

#[get("maven/modrinth/{id}/{versionnum}/{file}.sha1")]
pub async fn version_file_sha1(
    req: HttpRequest,
    params: web::Path<(String, String, String)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let (project_id, vnum, file) = params.into_inner();
    let project_data =
        database::models::Project::get_full_from_slug_or_project_id(
            &project_id,
            &**pool,
        )
        .await?;

    let project = if let Some(data) = project_data {
        data
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if !is_authorized(&project, &user_option, &pool).await? {
        return Ok(HttpResponse::NotFound().body(""));
    }

    let vid = if let Some(vid) = sqlx::query!(
        "SELECT id FROM versions WHERE mod_id = $1 AND version_number = $2",
        project.inner.id as database::models::ids::ProjectId,
        vnum
    )
    .fetch_optional(&**pool)
    .await?
    {
        vid
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let version = if let Some(version) = database::models::Version::get_full(
        database::models::ids::VersionId(vid.id),
        &**pool,
    )
    .await?
    {
        version
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    Ok(find_file(&project_id, &project, &version, &file)
        .and_then(|file| file.hashes.get("sha1"))
        .and_then(|hash_bytes| std::str::from_utf8(hash_bytes).ok())
        .map(|hash_str| HttpResponse::Ok().body(hash_str.to_string()))
        .unwrap_or_else(|| HttpResponse::NotFound().body("")))
}

#[get("maven/modrinth/{id}/{versionnum}/{file}.sha512")]
pub async fn version_file_sha512(
    req: HttpRequest,
    params: web::Path<(String, String, String)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let (project_id, vnum, file) = params.into_inner();
    let project_data =
        database::models::Project::get_full_from_slug_or_project_id(
            &project_id,
            &**pool,
        )
        .await?;

    let project = if let Some(data) = project_data {
        data
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if !is_authorized(&project, &user_option, &pool).await? {
        return Ok(HttpResponse::NotFound().body(""));
    }

    let vid = if let Some(vid) = sqlx::query!(
        "SELECT id FROM versions WHERE mod_id = $1 AND version_number = $2",
        project.inner.id as database::models::ids::ProjectId,
        vnum
    )
    .fetch_optional(&**pool)
    .await?
    {
        vid
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let version = if let Some(version) = database::models::Version::get_full(
        database::models::ids::VersionId(vid.id),
        &**pool,
    )
    .await?
    {
        version
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    Ok(find_file(&project_id, &project, &version, &file)
        .and_then(|file| file.hashes.get("sha512"))
        .and_then(|hash_bytes| std::str::from_utf8(hash_bytes).ok())
        .map(|hash_str| HttpResponse::Ok().body(hash_str.to_string()))
        .unwrap_or_else(|| HttpResponse::NotFound().body("")))
}
