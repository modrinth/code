use crate::auth::get_user_from_headers;
use crate::database;
use crate::models::mods::ModId;
use crate::routes::ApiError;
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
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;
    let id_option: Option<ModId> = serde_json::from_str(&*format!("\"{}\"", string)).ok();

    let mod_data = if let Some(id) = id_option {
        match database::models::Mod::get_full(id.into(), &**pool).await {
            Ok(Some(data)) => Ok(Some(data)),
            Ok(None) => database::models::Mod::get_full_from_slug(&string, &**pool).await,
            Err(e) => Err(e),
        }
    } else {
        database::models::Mod::get_full_from_slug(&string, &**pool).await
    }
    .map_err(|e| ApiError::DatabaseError(e.into()))?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    let data = if let Some(data) = mod_data {
        data
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let mut authorized = !data.status.is_hidden();

    if let Some(user) = user_option {
        if !authorized {
            if user.role.is_mod() {
                authorized = true;
            } else {
                let user_id: database::models::ids::UserId = user.id.into();

                let mod_exists = sqlx::query!(
                    "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
                    data.inner.team_id as database::models::ids::TeamId,
                    user_id as database::models::ids::UserId,
                )
                .fetch_one(&**pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?
                .exists;

                authorized = mod_exists.unwrap_or(false);
            }
        }
    }

    if !authorized {
        return Ok(HttpResponse::NotFound().body(""));
    }
    let version_names = sqlx::query!(
        "
            SELECT version_number, release_channels.channel channel
            FROM versions
            LEFT JOIN release_channels ON release_channels.id = versions.release_channel
            WHERE mod_id = $1
            ",
        data.inner.id as database::models::ids::ModId
    )
    .fetch_all(&**pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?;

    let respdata = Metadata {
        group_id: "maven.modrinth".to_string(),
        artifact_id: string,
        versioning: Versioning {
            latest: version_names
                .last()
                .map_or("release", |x| &x.version_number)
                .to_string(),
            release: version_names
                .iter()
                .rfind(|x| x.channel == "release")
                .map_or("", |x| &x.version_number)
                .to_string(),
            versions: Versions {
                versions: version_names
                    .iter()
                    .map(|x| x.version_number.clone())
                    .collect::<Vec<_>>(),
            },
            last_updated: data.inner.updated.format("%Y%m%d%H%M%S").to_string(),
        },
    };

    Ok(HttpResponse::Ok()
        .content_type("text/xml")
        .body(yaserde::ser::to_string(&respdata).map_err(|e| ApiError::XmlError(e))?))
}

#[get("maven/modrinth/{id}/{versionnum}/{file}")]
pub async fn version_file(
    req: HttpRequest,
    web::Path((string, vnum, file)): web::Path<(String, String, String)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id_option: Option<ModId> = serde_json::from_str(&*format!("\"{}\"", string)).ok();

    let mod_data = if let Some(id) = id_option {
        match database::models::Mod::get_full(id.into(), &**pool).await {
            Ok(Some(data)) => Ok(Some(data)),
            Ok(None) => database::models::Mod::get_full_from_slug(&string, &**pool).await,
            Err(e) => Err(e),
        }
    } else {
        database::models::Mod::get_full_from_slug(&string, &**pool).await
    }
    .map_err(|e| ApiError::DatabaseError(e.into()))?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    let data = if let Some(data) = mod_data {
        data
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let mut authorized = !data.status.is_hidden();

    if let Some(user) = user_option {
        if !authorized {
            if user.role.is_mod() {
                authorized = true;
            } else {
                let user_id: database::models::ids::UserId = user.id.into();

                let mod_exists = sqlx::query!(
                    "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
                    data.inner.team_id as database::models::ids::TeamId,
                    user_id as database::models::ids::UserId,
                )
                .fetch_one(&**pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?
                .exists;

                authorized = mod_exists.unwrap_or(false);
            }
        }
    }

    if !authorized {
        return Ok(HttpResponse::NotFound().body(""));
    }

    let vid = if let Some(vid) = sqlx::query!(
        "SELECT id FROM versions WHERE mod_id = $1 AND version_number = $2",
        data.inner.id as database::models::ids::ModId,
        vnum
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?
    {
        vid
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let version = if let Some(version) =
        database::models::Version::get_full(database::models::ids::VersionId(vid.id), &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?
    {
        version
    } else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    if file == format!("{}-{}.pom", &string, &version.version_number) {
        let respdata = MavenPom {
            schema_location:
                "http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd"
                    .to_string(),
            xsi: "http://www.w3.org/2001/XMLSchema-instance".to_string(),
            model_version: "4.0.0".to_string(),
            group_id: "maven.modrinth".to_string(),
            artifact_id: string,
            version: version.version_number,
            name: data.inner.title,
            description: data.inner.description,
        };
        return Ok(HttpResponse::Ok()
            .content_type("text/xml")
            .body(yaserde::ser::to_string(&respdata).map_err(|e| ApiError::XmlError(e))?));
    } else {
        if let Some(selected_file) = version.files.iter().find(|x| x.filename == file) {
            return Ok(HttpResponse::TemporaryRedirect()
                .header("Location", &*selected_file.url)
                .body(""));
        } else if file == format!("{}-{}.jar", &string, &version.version_number) {
            if let Some(selected_file) = version.files.iter().last() {
                return Ok(HttpResponse::TemporaryRedirect()
                    .header("Location", &*selected_file.url)
                    .body(""));
            }
        };
    }

    Ok(HttpResponse::NotFound().body(""))
}
