use crate::auth::checks::is_visible_collection;
use crate::auth::{filter_visible_collections, get_user_from_headers};
use crate::database::models::{
    collection_item, generate_collection_id, project_item,
};
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::collections::{Collection, CollectionStatus};
use crate::models::ids::{CollectionId, ProjectId};
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::routes::v3::project_creation::CreateError;
use crate::util::img::delete_old_images;
use crate::util::routes::read_from_payload;
use crate::util::validate::validation_errors_to_string;
use crate::{database, models};
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse, web};
use ariadne::ids::base62_impl::parse_base62;
use chrono::Utc;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("collections", web::get().to(collections_get));
    cfg.route("collection", web::post().to(collection_create));

    cfg.service(
        web::scope("collection")
            .route("{id}", web::get().to(collection_get))
            .route("{id}", web::delete().to(collection_delete))
            .route("{id}", web::patch().to(collection_edit))
            .route("{id}/icon", web::patch().to(collection_icon_edit))
            .route("{id}/icon", web::delete().to(delete_collection_icon)),
    );
}

#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct CollectionCreateData {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    /// The title or name of the project.
    pub name: String,
    #[validate(length(min = 3, max = 255))]
    /// A short description of the collection.
    pub description: Option<String>,
    #[validate(length(max = 1024))]
    #[serde(default = "Vec::new")]
    /// A list of initial projects to use with the created collection
    pub projects: Vec<String>,
}

pub async fn collection_create(
    req: HttpRequest,
    collection_create_data: web::Json<CollectionCreateData>,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, CreateError> {
    let collection_create_data = collection_create_data.into_inner();

    // The currently logged in user
    let current_user = get_user_from_headers(
        &req,
        &**client,
        &redis,
        &session_queue,
        Some(&[Scopes::COLLECTION_CREATE]),
    )
    .await?
    .1;

    collection_create_data.validate().map_err(|err| {
        CreateError::InvalidInput(validation_errors_to_string(err, None))
    })?;

    let mut transaction = client.begin().await?;

    let collection_id: CollectionId =
        generate_collection_id(&mut transaction).await?.into();

    let initial_project_ids = project_item::DBProject::get_many(
        &collection_create_data.projects,
        &mut *transaction,
        &redis,
    )
    .await?
    .into_iter()
    .map(|x| x.inner.id.into())
    .collect::<Vec<ProjectId>>();

    let collection_builder_actual = collection_item::CollectionBuilder {
        collection_id: collection_id.into(),
        user_id: current_user.id.into(),
        name: collection_create_data.name,
        description: collection_create_data.description,
        status: CollectionStatus::Listed,
        projects: initial_project_ids
            .iter()
            .copied()
            .map(|x| x.into())
            .collect(),
    };
    let collection_builder = collection_builder_actual.clone();

    let now = Utc::now();
    collection_builder_actual.insert(&mut transaction).await?;

    let response = crate::models::collections::Collection {
        id: collection_id,
        user: collection_builder.user_id.into(),
        name: collection_builder.name.clone(),
        description: collection_builder.description.clone(),
        created: now,
        updated: now,
        icon_url: None,
        color: None,
        status: collection_builder.status,
        projects: initial_project_ids,
    };
    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Serialize, Deserialize)]
pub struct CollectionIds {
    pub ids: String,
}
pub async fn collections_get(
    req: HttpRequest,
    web::Query(ids): web::Query<CollectionIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids = serde_json::from_str::<Vec<&str>>(&ids.ids)?;
    let ids = ids
        .into_iter()
        .map(|x| {
            parse_base62(x).map(|x| database::models::DBCollectionId(x as i64))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let collections_data =
        database::models::DBCollection::get_many(&ids, &**pool, &redis).await?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::COLLECTION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let collections =
        filter_visible_collections(collections_data, &user_option).await?;

    Ok(HttpResponse::Ok().json(collections))
}

pub async fn collection_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let id = database::models::DBCollectionId(parse_base62(&string)? as i64);
    let collection_data =
        database::models::DBCollection::get(id, &**pool, &redis).await?;
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::COLLECTION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    if let Some(data) = collection_data {
        if is_visible_collection(&data, &user_option).await? {
            return Ok(HttpResponse::Ok().json(Collection::from(data)));
        }
    }
    Err(ApiError::NotFound)
}

#[derive(Deserialize, Validate)]
pub struct EditCollection {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub name: Option<String>,
    #[validate(length(min = 3, max = 256))]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub description: Option<Option<String>>,
    pub status: Option<CollectionStatus>,
    #[validate(length(max = 1024))]
    pub new_projects: Option<Vec<String>>,
}

pub async fn collection_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    new_collection: web::Json<EditCollection>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::COLLECTION_WRITE]),
    )
    .await?
    .1;

    new_collection.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let string = info.into_inner().0;
    let id = database::models::DBCollectionId(parse_base62(&string)? as i64);
    let result =
        database::models::DBCollection::get(id, &**pool, &redis).await?;

    if let Some(collection_item) = result {
        if !can_modify_collection(&collection_item, &user) {
            return Ok(HttpResponse::Unauthorized().body(""));
        }

        let id = collection_item.id;

        let mut transaction = pool.begin().await?;

        if let Some(name) = &new_collection.name {
            sqlx::query!(
                "
                UPDATE collections
                SET name = $1
                WHERE (id = $2)
                ",
                name.trim(),
                id as database::models::ids::DBCollectionId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        if let Some(description) = &new_collection.description {
            sqlx::query!(
                "
                UPDATE collections
                SET description = $1
                WHERE (id = $2)
                ",
                description.as_ref(),
                id as database::models::ids::DBCollectionId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        if let Some(status) = &new_collection.status {
            if !(user.role.is_mod()
                || collection_item.status.is_approved()
                    && status.can_be_requested())
            {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to set this status!".to_string(),
                ));
            }

            sqlx::query!(
                "
                UPDATE collections
                SET status = $1
                WHERE (id = $2)
                ",
                status.to_string(),
                id as database::models::ids::DBCollectionId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        if let Some(new_project_ids) = &new_collection.new_projects {
            // Delete all existing projects
            sqlx::query!(
                "
                DELETE FROM collections_mods
                WHERE collection_id = $1
                ",
                collection_item.id as database::models::ids::DBCollectionId,
            )
            .execute(&mut *transaction)
            .await?;

            let collection_item_ids = new_project_ids
                .iter()
                .map(|_| collection_item.id.0)
                .collect_vec();
            let mut validated_project_ids = Vec::new();
            for project_id in new_project_ids {
                let project = database::models::DBProject::get(
                    project_id, &**pool, &redis,
                )
                .await?
                .ok_or_else(|| {
                    ApiError::InvalidInput(format!(
                        "The specified project {project_id} does not exist!"
                    ))
                })?;
                validated_project_ids.push(project.inner.id.0);
            }
            // Insert- don't throw an error if it already exists
            sqlx::query!(
                "
                        INSERT INTO collections_mods (collection_id, mod_id)
                        SELECT * FROM UNNEST ($1::int8[], $2::int8[])
                        ON CONFLICT DO NOTHING
                        ",
                &collection_item_ids[..],
                &validated_project_ids[..],
            )
            .execute(&mut *transaction)
            .await?;

            sqlx::query!(
                "
                UPDATE collections
                SET updated = NOW()
                WHERE id = $1
                ",
                collection_item.id as database::models::ids::DBCollectionId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        database::models::DBCollection::clear_cache(collection_item.id, &redis)
            .await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn collection_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::COLLECTION_WRITE]),
    )
    .await?
    .1;

    let string = info.into_inner().0;
    let id = database::models::DBCollectionId(parse_base62(&string)? as i64);
    let collection_item =
        database::models::DBCollection::get(id, &**pool, &redis)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified collection does not exist!".to_string(),
                )
            })?;

    if !can_modify_collection(&collection_item, &user) {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    delete_old_images(
        collection_item.icon_url,
        collection_item.raw_icon_url,
        &***file_host,
    )
    .await?;

    let bytes = read_from_payload(
        &mut payload,
        262144,
        "Icons must be smaller than 256KiB",
    )
    .await?;

    let collection_id: CollectionId = collection_item.id.into();
    let upload_result = crate::util::img::upload_image_optimized(
        &format!("data/{collection_id}"),
        bytes.freeze(),
        &ext.ext,
        Some(96),
        Some(1.0),
        &***file_host,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        UPDATE collections
        SET icon_url = $1, raw_icon_url = $2, color = $3
        WHERE (id = $4)
        ",
        upload_result.url,
        upload_result.raw_url,
        upload_result.color.map(|x| x as i32),
        collection_item.id as database::models::ids::DBCollectionId,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    database::models::DBCollection::clear_cache(collection_item.id, &redis)
        .await?;

    Ok(HttpResponse::NoContent().body(""))
}

pub async fn delete_collection_icon(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::COLLECTION_WRITE]),
    )
    .await?
    .1;

    let string = info.into_inner().0;
    let id = database::models::DBCollectionId(parse_base62(&string)? as i64);
    let collection_item =
        database::models::DBCollection::get(id, &**pool, &redis)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "The specified collection does not exist!".to_string(),
                )
            })?;
    if !can_modify_collection(&collection_item, &user) {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    delete_old_images(
        collection_item.icon_url,
        collection_item.raw_icon_url,
        &***file_host,
    )
    .await?;
    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        UPDATE collections
        SET icon_url = NULL, raw_icon_url = NULL, color = NULL
        WHERE (id = $1)
        ",
        collection_item.id as database::models::ids::DBCollectionId,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    database::models::DBCollection::clear_cache(collection_item.id, &redis)
        .await?;

    Ok(HttpResponse::NoContent().body(""))
}

pub async fn collection_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::COLLECTION_DELETE]),
    )
    .await?
    .1;

    let string = info.into_inner().0;
    let id = database::models::DBCollectionId(parse_base62(&string)? as i64);
    let collection = database::models::DBCollection::get(id, &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified collection does not exist!".to_string(),
            )
        })?;
    if !can_modify_collection(&collection, &user) {
        return Ok(HttpResponse::Unauthorized().body(""));
    }
    let mut transaction = pool.begin().await?;

    let result = database::models::DBCollection::remove(
        collection.id,
        &mut transaction,
        &redis,
    )
    .await?;

    transaction.commit().await?;
    database::models::DBCollection::clear_cache(collection.id, &redis).await?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}

fn can_modify_collection(
    collection: &database::models::DBCollection,
    user: &models::users::User,
) -> bool {
    collection.user_id == user.id.into() || user.role.is_mod()
}
