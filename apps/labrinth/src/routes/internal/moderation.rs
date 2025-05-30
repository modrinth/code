use super::ApiError;
use crate::database;
use crate::database::redis::RedisPool;
use crate::models::projects::ProjectStatus;
use crate::queue::moderation::{ApprovalType, IdentifiedFile, MissingMetadata};
use crate::queue::session::AuthQueue;
use crate::{auth::check_is_moderator_from_headers, models::pats::Scopes};
use actix_web::{HttpRequest, HttpResponse, web};
use ariadne::ids::random_base62;
use serde::Deserialize;
use sqlx::PgPool;
use std::collections::HashMap;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("moderation/projects", web::get().to(get_projects));
    cfg.route("moderation/project/{id}", web::get().to(get_project_meta));
    cfg.route("moderation/project", web::post().to(set_project_meta));
}

#[derive(Deserialize)]
pub struct ResultCount {
    #[serde(default = "default_count")]
    pub count: i16,
}

fn default_count() -> i16 {
    100
}

pub async fn get_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    count: web::Query<ResultCount>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await?;

    use futures::stream::TryStreamExt;

    let project_ids = sqlx::query!(
        "
        SELECT id FROM mods
        WHERE status = $1
        ORDER BY queued ASC
        LIMIT $2;
        ",
        ProjectStatus::Processing.as_str(),
        count.count as i64
    )
    .fetch(&**pool)
    .map_ok(|m| database::models::DBProjectId(m.id))
    .try_collect::<Vec<database::models::DBProjectId>>()
    .await?;

    let projects: Vec<_> =
        database::DBProject::get_many_ids(&project_ids, &**pool, &redis)
            .await?
            .into_iter()
            .map(crate::models::projects::Project::from)
            .collect();

    Ok(HttpResponse::Ok().json(projects))
}

pub async fn get_project_meta(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    info: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await?;

    let project_id = info.into_inner().0;
    let project =
        database::models::DBProject::get(&project_id, &**pool, &redis).await?;

    if let Some(project) = project {
        let rows = sqlx::query!(
            "
            SELECT
            f.metadata, v.id version_id
            FROM versions v
            INNER JOIN files f ON f.version_id = v.id
            WHERE v.mod_id = $1
            ",
            project.inner.id.0
        )
        .fetch_all(&**pool)
        .await?;

        let mut merged = MissingMetadata {
            identified: HashMap::new(),
            flame_files: HashMap::new(),
            unknown_files: HashMap::new(),
        };

        let mut check_hashes = Vec::new();
        let mut check_flames = Vec::new();

        for row in rows {
            if let Some(metadata) = row
                .metadata
                .and_then(|x| serde_json::from_value::<MissingMetadata>(x).ok())
            {
                merged.identified.extend(metadata.identified);
                merged.flame_files.extend(metadata.flame_files);
                merged.unknown_files.extend(metadata.unknown_files);

                check_hashes.extend(merged.flame_files.keys().cloned());
                check_hashes.extend(merged.unknown_files.keys().cloned());
                check_flames
                    .extend(merged.flame_files.values().map(|x| x.id as i32));
            }
        }

        let rows = sqlx::query!(
            "
            SELECT encode(mef.sha1, 'escape') sha1, mel.status status
            FROM moderation_external_files mef
            INNER JOIN moderation_external_licenses mel ON mef.external_license_id = mel.id
            WHERE mef.sha1 = ANY($1)
            ",
            &check_hashes
                .iter()
                .map(|x| x.as_bytes().to_vec())
                .collect::<Vec<_>>()
        )
        .fetch_all(&**pool)
        .await?;

        for row in rows {
            if let Some(sha1) = row.sha1 {
                if let Some(val) = merged.flame_files.remove(&sha1) {
                    merged.identified.insert(
                        sha1,
                        IdentifiedFile {
                            file_name: val.file_name,
                            status: ApprovalType::from_string(&row.status)
                                .unwrap_or(ApprovalType::Unidentified),
                        },
                    );
                } else if let Some(val) = merged.unknown_files.remove(&sha1) {
                    merged.identified.insert(
                        sha1,
                        IdentifiedFile {
                            file_name: val,
                            status: ApprovalType::from_string(&row.status)
                                .unwrap_or(ApprovalType::Unidentified),
                        },
                    );
                }
            }
        }

        let rows = sqlx::query!(
            "
            SELECT mel.id, mel.flame_project_id, mel.status status
            FROM moderation_external_licenses mel
            WHERE mel.flame_project_id = ANY($1)
            ",
            &check_flames,
        )
        .fetch_all(&**pool)
        .await?;

        for row in rows {
            if let Some(sha1) = merged
                .flame_files
                .iter()
                .find(|x| Some(x.1.id as i32) == row.flame_project_id)
                .map(|x| x.0.clone())
            {
                if let Some(val) = merged.flame_files.remove(&sha1) {
                    merged.identified.insert(
                        sha1,
                        IdentifiedFile {
                            file_name: val.file_name.clone(),
                            status: ApprovalType::from_string(&row.status)
                                .unwrap_or(ApprovalType::Unidentified),
                        },
                    );
                }
            }
        }

        Ok(HttpResponse::Ok().json(merged))
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Judgement {
    Flame {
        id: i32,
        status: ApprovalType,
        link: String,
        title: String,
    },
    Unknown {
        status: ApprovalType,
        proof: Option<String>,
        link: Option<String>,
        title: Option<String>,
    },
}

pub async fn set_project_meta(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    judgements: web::Json<HashMap<String, Judgement>>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await?;

    let mut transaction = pool.begin().await?;

    let mut ids = Vec::new();
    let mut titles = Vec::new();
    let mut statuses = Vec::new();
    let mut links = Vec::new();
    let mut proofs = Vec::new();
    let mut flame_ids = Vec::new();

    let mut file_hashes = Vec::new();

    for (hash, judgement) in judgements.0 {
        let id = random_base62(8);

        let (title, status, link, proof, flame_id) = match judgement {
            Judgement::Flame {
                id,
                status,
                link,
                title,
            } => (
                Some(title),
                status,
                Some(link),
                Some("See Flame page/license for permission".to_string()),
                Some(id),
            ),
            Judgement::Unknown {
                status,
                proof,
                link,
                title,
            } => (title, status, link, proof, None),
        };

        ids.push(id as i64);
        titles.push(title);
        statuses.push(status.as_str());
        links.push(link);
        proofs.push(proof);
        flame_ids.push(flame_id);
        file_hashes.push(hash);
    }

    sqlx::query(
    "
        INSERT INTO moderation_external_licenses (id, title, status, link, proof, flame_project_id)
        SELECT * FROM UNNEST ($1::bigint[], $2::varchar[], $3::varchar[], $4::varchar[], $5::varchar[], $6::integer[])
        "
    )
        .bind(&ids[..])
        .bind(&titles[..])
        .bind(&statuses[..])
        .bind(&links[..])
        .bind(&proofs[..])
        .bind(&flame_ids[..])
        .execute(&mut *transaction)
        .await?;

    sqlx::query(
        "
            INSERT INTO moderation_external_files (sha1, external_license_id)
            SELECT * FROM UNNEST ($1::bytea[], $2::bigint[])
            ON CONFLICT (sha1)
            DO NOTHING
            ",
    )
    .bind(&file_hashes[..])
    .bind(&ids[..])
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
