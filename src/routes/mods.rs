use super::ApiError;
use crate::auth::get_user_from_headers;
use crate::database;
use crate::file_hosting::FileHost;
use crate::models;
use crate::models::mods::{ModStatus, SearchRequest};
use crate::models::teams::Permissions;
use crate::search::{search_for_mod, SearchConfig, SearchError};
use actix_web::{delete, get, patch, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

#[get("mod")]
pub async fn mod_search(
    web::Query(info): web::Query<SearchRequest>,
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, SearchError> {
    let results = search_for_mod(&info, &**config).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[derive(Serialize, Deserialize)]
pub struct ModIds {
    pub ids: String,
}

// TODO: Make this return the full mod struct
#[get("mods")]
pub async fn mods_get(
    req: HttpRequest,
    web::Query(ids): web::Query<ModIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let mod_ids = serde_json::from_str::<Vec<models::ids::ModId>>(&*ids.ids)?
        .into_iter()
        .map(|x| x.into())
        .collect();

    let mods_data = database::models::Mod::get_many_full(mod_ids, &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    let mut mods = Vec::new();

    for mod_data_option in mods_data {
        if let Some(mod_data) = mod_data_option {
            let mut authorized = !mod_data.status.is_hidden();

            if let Some(user) = &user_option {
                if !authorized {
                    if user.role.is_mod() {
                        authorized = true;
                    } else {
                        let user_id: database::models::ids::UserId = user.id.into();

                        let mod_exists = sqlx::query!(
                            "SELECT EXISTS(SELECT 1 FROM team_members WHERE id = $1 AND user_id = $2)",
                            mod_data.inner.team_id as database::models::ids::TeamId,
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

            if authorized {
                mods.push(convert_mod(mod_data));
            }
        }
    }

    Ok(HttpResponse::Ok().json(mods))
}

#[get("{id}")]
pub async fn mod_get(
    req: HttpRequest,
    info: web::Path<(models::ids::ModId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let mod_data = database::models::Mod::get_full(id.into(), &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;
    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(data) = mod_data {
        let mut authorized = !data.status.is_hidden();

        if let Some(user) = user_option {
            if !authorized {
                if user.role.is_mod() {
                    authorized = true;
                } else {
                    let user_id: database::models::ids::UserId = user.id.into();

                    let mod_exists = sqlx::query!(
                        "SELECT EXISTS(SELECT 1 FROM team_members WHERE id = $1 AND user_id = $2)",
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

        if authorized {
            return Ok(HttpResponse::Ok().json(convert_mod(data)));
        }

        Ok(HttpResponse::NotFound().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

fn convert_mod(data: database::models::mod_item::QueryMod) -> models::mods::Mod {
    let m = data.inner;

    models::mods::Mod {
        id: m.id.into(),
        team: m.team_id.into(),
        title: m.title,
        description: m.description,
        body_url: m.body_url,
        published: m.published,
        updated: m.updated,
        status: data.status,
        downloads: m.downloads as u32,
        categories: data.categories,
        versions: data.versions.into_iter().map(|v| v.into()).collect(),
        icon_url: m.icon_url,
        issues_url: m.issues_url,
        source_url: m.source_url,
        wiki_url: m.wiki_url,
    }
}

/// A mod returned from the API
#[derive(Serialize, Deserialize)]
pub struct EditMod {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    pub status: Option<ModStatus>,
    pub categories: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub issues_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub source_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub wiki_url: Option<Option<String>>,
}

#[patch("{id}")]
pub async fn mod_edit(
    req: HttpRequest,
    info: web::Path<(models::ids::ModId,)>,
    pool: web::Data<PgPool>,
    config: web::Data<SearchConfig>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    new_mod: web::Json<EditMod>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let mod_id = info.into_inner().0;
    let id = mod_id.into();

    let result = database::models::Mod::get_full(id, &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if let Some(mod_item) = result {
        let team_member = database::models::TeamMember::get_from_user_id(
            mod_item.inner.team_id,
            user.id.into(),
            &**pool,
        )
        .await?;
        let permissions;

        if let Some(member) = team_member {
            permissions = Some(member.permissions)
        } else if user.role.is_mod() {
            permissions = Some(Permissions::ALL)
        } else {
            permissions = None
        }

        if let Some(perms) = permissions {
            let mut transaction = pool
                .begin()
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

            if let Some(title) = &new_mod.title {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthenticationError(
                        "You do not have the permissions to edit the title of this mod!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET title = $1
                    WHERE (id = $2)
                    ",
                    title,
                    id as database::models::ids::ModId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

            if let Some(description) = &new_mod.description {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthenticationError(
                        "You do not have the permissions to edit the description of this mod!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET description = $1
                    WHERE (id = $2)
                    ",
                    description,
                    id as database::models::ids::ModId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

            if let Some(status) = &new_mod.status {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthenticationError(
                        "You do not have the permissions to edit the status of this mod!"
                            .to_string(),
                    ));
                }

                if status == &ModStatus::Rejected || status == &ModStatus::Approved {
                    if !user.role.is_mod() {
                        return Err(ApiError::CustomAuthenticationError(
                            "You don't have permission to set this status".to_string(),
                        ));
                    }
                }

                let status_id = database::models::StatusId::get_id(&status, &mut *transaction)
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInputError(
                            "No database entry for status provided.".to_string(),
                        )
                    })?;
                sqlx::query!(
                    "
                    UPDATE mods
                    SET status = $1
                    WHERE (id = $2)
                    ",
                    status_id as database::models::ids::StatusId,
                    id as database::models::ids::ModId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

                if mod_item.status.is_searchable() && status.is_searchable() {
                    delete_from_index(id.into(), config).await?;
                }
            }

            if let Some(categories) = &new_mod.categories {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthenticationError(
                        "You do not have the permissions to edit the categories of this mod!"
                            .to_string(),
                    ));
                }

                if categories.len() > 3 {
                    return Err(ApiError::InvalidInputError(
                        "The maximum number of categories for a mod is four.".to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    DELETE FROM mods_categories
                    WHERE joining_mod_id = $1
                    ",
                    id as database::models::ids::ModId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

                for category in categories {
                    let category_id = database::models::categories::Category::get_id(
                        &category,
                        &mut *transaction,
                    )
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInputError(format!(
                            "Category {} does not exist.",
                            category.clone()
                        ))
                    })?;

                    sqlx::query!(
                        "
                        INSERT INTO mods_categories (joining_mod_id, joining_category_id)
                        VALUES ($1, $2)
                        ",
                        id as database::models::ids::ModId,
                        category_id as database::models::ids::CategoryId,
                    )
                    .execute(&mut *transaction)
                    .await
                    .map_err(|e| ApiError::DatabaseError(e.into()))?;
                }
            }

            if let Some(issues_url) = &new_mod.issues_url {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthenticationError(
                        "You do not have the permissions to edit the issues URL of this mod!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET issues_url = $1
                    WHERE (id = $2)
                    ",
                    issues_url.as_deref(),
                    id as database::models::ids::ModId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

            if let Some(source_url) = &new_mod.source_url {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthenticationError(
                        "You do not have the permissions to edit the source URL of this mod!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET source_url = $1
                    WHERE (id = $2)
                    ",
                    source_url.as_deref(),
                    id as database::models::ids::ModId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

            if let Some(wiki_url) = &new_mod.wiki_url {
                if !perms.contains(Permissions::EDIT_DETAILS) {
                    return Err(ApiError::CustomAuthenticationError(
                        "You do not have the permissions to edit the wiki URL of this mod!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE mods
                    SET wiki_url = $1
                    WHERE (id = $2)
                    ",
                    wiki_url.as_deref(),
                    id as database::models::ids::ModId,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            }

            if let Some(body) = &new_mod.body {
                if !perms.contains(Permissions::EDIT_BODY) {
                    return Err(ApiError::CustomAuthenticationError(
                        "You do not have the permissions to edit the body of this mod!".to_string(),
                    ));
                }

                let body_path = format!("data/{}/description.md", mod_id);

                file_host.delete_file_version("", &*body_path).await?;

                file_host
                    .upload_file("text/plain", &body_path, body.clone().into_bytes())
                    .await?;
            }

            transaction
                .commit()
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
            Ok(HttpResponse::Ok().body(""))
        } else {
            Err(ApiError::CustomAuthenticationError(
                "You do not have permission to edit this mod!".to_string(),
            ))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[delete("{id}")]
pub async fn mod_delete(
    req: HttpRequest,
    info: web::Path<(models::ids::ModId,)>,
    pool: web::Data<PgPool>,
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let id = info.into_inner().0;

    if !user.role.is_mod() {
        let mod_item = database::models::Mod::get(id.into(), &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?
            .ok_or_else(|| ApiError::InvalidInputError("Invalid Mod ID specified!".to_string()))?;
        let team_member = database::models::TeamMember::get_from_user_id(
            mod_item.team_id,
            user.id.into(),
            &**pool,
        )
        .await
        .map_err(ApiError::DatabaseError)?
        .ok_or_else(|| ApiError::InvalidInputError("Invalid Mod ID specified!".to_string()))?;

        if !team_member.permissions.contains(Permissions::DELETE_MOD) {
            return Err(ApiError::CustomAuthenticationError(
                "You don't have permission to delete this mod".to_string(),
            ));
        }
    }

    let result = database::models::Mod::remove_full(id.into(), &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    delete_from_index(id, config).await?;

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

pub async fn delete_from_index(
    id: crate::models::mods::ModId,
    config: web::Data<SearchConfig>,
) -> Result<(), meilisearch_sdk::errors::Error> {
    let client = meilisearch_sdk::client::Client::new(&*config.address, &*config.key);

    let indexes: Vec<meilisearch_sdk::indexes::Index> = client.get_indexes().await?;
    for index in indexes {
        index.delete_document(format!("local-{}", id)).await?;
    }

    Ok(())
}
