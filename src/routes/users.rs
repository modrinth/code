use crate::auth::get_user_from_headers;
use crate::database::models::{TeamMember, User};
use crate::file_hosting::FileHost;
use crate::models::users::{Role, UserId};
use crate::routes::ApiError;
use actix_web::{delete, get, patch, web, HttpRequest, HttpResponse};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

#[get("user")]
pub async fn user_auth_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(
        get_user_from_headers(
            req.headers(),
            &mut *pool
                .acquire()
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?,
        )
        .await?,
    ))
}

#[derive(Serialize, Deserialize)]
pub struct UserIds {
    pub ids: String,
}

#[get("users")]
pub async fn users_get(
    web::Query(ids): web::Query<UserIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user_ids = serde_json::from_str::<Vec<UserId>>(&*ids.ids)?
        .into_iter()
        .map(|x| x.into())
        .collect();

    let users_data = User::get_many(user_ids, &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    let users: Vec<crate::models::users::User> = users_data.into_iter().map(convert_user).collect();

    Ok(HttpResponse::Ok().json(users))
}

#[get("@{id}")]
pub async fn user_username_get(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let user_data = User::get_from_username(id, &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if let Some(data) = user_data {
        let response = convert_user(data);
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("{id}")]
pub async fn user_get(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;
    let id_option: Option<UserId> = serde_json::from_str(&*format!("\"{}\"", string)).ok();

    let mut user_data;

    if let Some(id) = id_option {
        user_data = User::get(id.into(), &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;

        if user_data.is_none() {
            user_data = User::get_from_username(string, &**pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;
        }
    } else {
        user_data = User::get_from_username(string, &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
    }

    if let Some(data) = user_data {
        let response = convert_user(data);
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

fn convert_user(data: crate::database::models::user_item::User) -> crate::models::users::User {
    crate::models::users::User {
        id: data.id.into(),
        github_id: data.github_id.map(|i| i as u64),
        username: data.username,
        name: data.name,
        email: None,
        avatar_url: data.avatar_url,
        bio: data.bio,
        created: data.created,
        role: Role::from_string(&*data.role),
    }
}

#[get("{user_id}/mods")]
pub async fn mods_list(
    info: web::Path<(UserId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0.into();

    let user_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)",
        id as crate::database::models::UserId,
    )
    .fetch_one(&**pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?
    .exists;

    if user_exists.unwrap_or(false) {
        let mod_data = User::get_mods(id, &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;

        let response = mod_data
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<crate::models::ids::ModId>>();

        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("{user_id}/teams")]
pub async fn teams(
    req: HttpRequest,
    info: web::Path<(UserId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id: crate::database::models::UserId = info.into_inner().0.into();

    let current_user = get_user_from_headers(req.headers(), &**pool).await.ok();

    let results;
    let mut same_user = false;

    if let Some(user) = current_user {
        if user.id.0 == id.0 as u64 {
            results = TeamMember::get_from_user_private(id, &**pool).await?;
            same_user = true;
        } else {
            results = TeamMember::get_from_user_public(id, &**pool).await?;
        }
    } else {
        results = TeamMember::get_from_user_public(id, &**pool).await?;
    }

    let team_members: Vec<crate::models::teams::TeamMember> = results
        .into_iter()
        .map(|data| crate::models::teams::TeamMember {
            team_id: data.team_id.into(),
            user_id: data.user_id.into(),
            role: data.role,
            permissions: if same_user {
                Some(data.permissions)
            } else {
                None
            },
            accepted: data.accepted,
        })
        .collect();

    Ok(HttpResponse::Ok().json(team_members))
}

#[derive(Serialize, Deserialize)]
pub struct EditUser {
    pub username: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub email: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub bio: Option<Option<String>>,
    pub role: Option<String>,
}

#[patch("{id}")]
pub async fn user_edit(
    req: HttpRequest,
    info: web::Path<(UserId,)>,
    pool: web::Data<PgPool>,
    new_user: web::Json<EditUser>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let user_id = info.into_inner().0;
    let id: crate::database::models::ids::UserId = user_id.into();

    if user.id == user_id || user.role.is_mod() {
        let mut transaction = pool
            .begin()
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;

        if let Some(username) = &new_user.username {
            sqlx::query!(
                "
                    UPDATE users
                    SET username = $1
                    WHERE (id = $2)
                    ",
                username,
                id as crate::database::models::ids::UserId,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
        }

        if let Some(name) = &new_user.name {
            sqlx::query!(
                "
                    UPDATE users
                    SET name = $1
                    WHERE (id = $2)
                    ",
                name.as_deref(),
                id as crate::database::models::ids::UserId,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
        }

        if let Some(bio) = &new_user.bio {
            sqlx::query!(
                "
                    UPDATE users
                    SET bio = $1
                    WHERE (id = $2)
                    ",
                bio.as_deref(),
                id as crate::database::models::ids::UserId,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
        }

        if let Some(email) = &new_user.email {
            sqlx::query!(
                "
                    UPDATE users
                    SET email = $1
                    WHERE (id = $2)
                    ",
                email.as_deref(),
                id as crate::database::models::ids::UserId,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
        }

        if let Some(role) = &new_user.role {
            if !user.role.is_mod() {
                return Err(ApiError::CustomAuthenticationError(
                    "You do not have the permissions to edit the role of this user!".to_string(),
                ));
            }

            let role = Role::from_string(role).to_string();

            sqlx::query!(
                "
                    UPDATE users
                    SET role = $1
                    WHERE (id = $2)
                    ",
                role,
                id as crate::database::models::ids::UserId,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
        }

        transaction
            .commit()
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
        Ok(HttpResponse::Ok().body(""))
    } else {
        Err(ApiError::CustomAuthenticationError(
            "You do not have permission to edit this user!".to_string(),
        ))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[patch("{id}/icon")]
pub async fn user_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(UserId,)>,
    pool: web::Data<PgPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
) -> Result<HttpResponse, ApiError> {
    if let Some(content_type) = super::mod_creation::get_image_content_type(&*ext.ext) {
        let cdn_url = dotenv::var("CDN_URL")?;
        let user = get_user_from_headers(req.headers(), &**pool).await?;
        let id = info.into_inner().0;

        if user.id != id && !user.role.is_mod() {
            return Err(ApiError::CustomAuthenticationError(
                "You don't have permission to edit this user's icon.".to_string(),
            ));
        }

        let mut icon_url = user.avatar_url;

        if user.id != id {
            let new_user = User::get(id.into(), &**pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.into()))?;

            if let Some(new) = new_user {
                icon_url = new.avatar_url;
            } else {
                return Ok(HttpResponse::NotFound().body(""));
            }
        }

        if let Some(icon) = icon_url {
            if icon.starts_with(&cdn_url) {
                let name = icon.split('/').next();

                if let Some(icon_path) = name {
                    file_host.delete_file_version("", icon_path).await?;
                }
            }
        }

        let mut bytes = web::BytesMut::new();
        while let Some(item) = payload.next().await {
            bytes.extend_from_slice(&item.map_err(|_| {
                ApiError::InvalidInputError("Unable to parse bytes in payload sent!".to_string())
            })?);
        }

        if bytes.len() >= 262144 {
            return Err(ApiError::InvalidInputError(String::from(
                "Icons must be smaller than 256KiB",
            )));
        }

        let upload_data = file_host
            .upload_file(
                content_type,
                &format!("user/{}/icon.{}", id, ext.ext),
                bytes.to_vec(),
            )
            .await?;

        let mod_id: crate::database::models::ids::UserId = id.into();
        sqlx::query!(
            "
            UPDATE users
            SET avatar_url = $1
            WHERE (id = $2)
            ",
            format!("{}/{}", cdn_url, upload_data.file_name),
            mod_id as crate::database::models::ids::UserId,
        )
        .execute(&**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

        Ok(HttpResponse::Ok().body(""))
    } else {
        Err(ApiError::InvalidInputError(format!(
            "Invalid format for user icon: {}",
            ext.ext
        )))
    }
}

#[derive(Deserialize)]
pub struct RemovalType {
    #[serde(default = "default_removal")]
    removal_type: String,
}

fn default_removal() -> String {
    "partial".into()
}

#[delete("{id}")]
pub async fn user_delete(
    req: HttpRequest,
    info: web::Path<(UserId,)>,
    pool: web::Data<PgPool>,
    removal_type: web::Query<RemovalType>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let id = info.into_inner().0;

    if !user.role.is_mod() && user.id != id {
        return Err(ApiError::CustomAuthenticationError(
            "You do not have permission to delete this user!".to_string(),
        ));
    }

    let result;
    if &*removal_type.removal_type == "full" {
        result = crate::database::models::User::remove_full(id.into(), &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
    } else {
        result = crate::database::models::User::remove(id.into(), &**pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.into()))?;
    };

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
