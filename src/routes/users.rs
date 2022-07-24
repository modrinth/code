use crate::database::models::User;
use crate::file_hosting::FileHost;
use crate::models::notifications::Notification;
use crate::models::projects::{Project, ProjectStatus};
use crate::models::users::{Role, UserId};
use crate::routes::ApiError;
use crate::util::auth::get_user_from_headers;
use crate::util::routes::read_from_payload;
use crate::util::validate::validation_errors_to_string;
use actix_web::{delete, get, patch, web, HttpRequest, HttpResponse};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use validator::Validate;

#[get("user")]
pub async fn user_auth_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(
        get_user_from_headers(req.headers(), &mut *pool.acquire().await?)
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

    let users_data = User::get_many(user_ids, &**pool).await?;

    let users: Vec<crate::models::users::User> =
        users_data.into_iter().map(From::from).collect();

    Ok(HttpResponse::Ok().json(users))
}

#[get("{id}")]
pub async fn user_get(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;
    let id_option: Option<UserId> =
        serde_json::from_str(&*format!("\"{}\"", string)).ok();

    let mut user_data;

    if let Some(id) = id_option {
        user_data = User::get(id.into(), &**pool).await?;

        if user_data.is_none() {
            user_data = User::get_from_username(string, &**pool).await?;
        }
    } else {
        user_data = User::get_from_username(string, &**pool).await?;
    }

    if let Some(data) = user_data {
        let response: crate::models::users::User = data.into();
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("{user_id}/projects")]
pub async fn projects_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await.ok();

    let id_option = crate::database::models::User::get_id_from_username_or_id(
        &*info.into_inner().0,
        &**pool,
    )
    .await?;

    if let Some(id) = id_option {
        let user_id: UserId = id.into();

        let project_data = if let Some(current_user) = user {
            if current_user.role.is_mod() || current_user.id == user_id {
                User::get_projects_private(id, &**pool).await?
            } else {
                User::get_projects(
                    id,
                    ProjectStatus::Approved.as_str(),
                    &**pool,
                )
                .await?
            }
        } else {
            User::get_projects(id, ProjectStatus::Approved.as_str(), &**pool)
                .await?
        };

        let response: Vec<_> =
            crate::database::Project::get_many_full(project_data, &**pool)
                .await?
                .into_iter()
                .map(Project::from)
                .collect();

        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

lazy_static! {
    static ref RE_URL_SAFE: Regex = Regex::new(r"^[a-zA-Z0-9_-]*$").unwrap();
}

#[derive(Serialize, Deserialize, Validate)]
pub struct EditUser {
    #[validate(length(min = 1, max = 255), regex = "RE_URL_SAFE")]
    pub username: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 255), regex = "RE_URL_SAFE")]
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(email)]
    pub email: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 160))]
    pub bio: Option<Option<String>>,
    pub role: Option<Role>,
}

#[patch("{id}")]
pub async fn user_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    new_user: web::Json<EditUser>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    new_user.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let id_option = crate::database::models::User::get_id_from_username_or_id(
        &*info.into_inner().0,
        &**pool,
    )
    .await?;

    if let Some(id) = id_option {
        let user_id: UserId = id.into();

        if user.id == user_id || user.role.is_mod() {
            let mut transaction = pool.begin().await?;

            if let Some(username) = &new_user.username {
                let existing_user_id_option =
                    crate::database::models::User::get_id_from_username_or_id(
                        username, &**pool,
                    )
                    .await?;

                if existing_user_id_option
                    .map(UserId::from)
                    .map(|id| id == user.id)
                    .unwrap_or(true)
                {
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
                    .await?;
                } else {
                    return Err(ApiError::InvalidInput(format!(
                        "Username {} is taken!",
                        username
                    )));
                }
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
                .await?;
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
                .await?;
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
                .await?;
            }

            if let Some(role) = &new_user.role {
                if !user.role.is_admin() {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the role of this user!"
                            .to_string(),
                    ));
                }

                let role = role.to_string();

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
                .await?;
            }

            transaction.commit().await?;
            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthentication(
                "You do not have permission to edit this user!".to_string(),
            ))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
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
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
) -> Result<HttpResponse, ApiError> {
    if let Some(content_type) =
        crate::util::ext::get_image_content_type(&*ext.ext)
    {
        let cdn_url = dotenv::var("CDN_URL")?;
        let user = get_user_from_headers(req.headers(), &**pool).await?;
        let id_option =
            crate::database::models::User::get_id_from_username_or_id(
                &*info.into_inner().0,
                &**pool,
            )
            .await?;

        if let Some(id) = id_option {
            if user.id != id.into() && !user.role.is_mod() {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to edit this user's icon."
                        .to_string(),
                ));
            }

            let mut icon_url = user.avatar_url;

            let user_id: UserId = id.into();

            if user.id != user_id {
                let new_user = User::get(id, &**pool).await?;

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

            let bytes = read_from_payload(
                &mut payload,
                2097152,
                "Icons must be smaller than 2MiB",
            )
            .await?;

            let upload_data = file_host
                .upload_file(
                    content_type,
                    &format!("user/{}/icon.{}", user_id, ext.ext),
                    bytes.freeze(),
                )
                .await?;

            sqlx::query!(
                "
                UPDATE users
                SET avatar_url = $1
                WHERE (id = $2)
                ",
                format!("{}/{}", cdn_url, upload_data.file_name),
                id as crate::database::models::ids::UserId,
            )
            .execute(&**pool)
            .await?;
            Ok(HttpResponse::NoContent().body(""))
        } else {
            Ok(HttpResponse::NotFound().body(""))
        }
    } else {
        Err(ApiError::InvalidInput(format!(
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
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    removal_type: web::Query<RemovalType>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let id_option = crate::database::models::User::get_id_from_username_or_id(
        &*info.into_inner().0,
        &**pool,
    )
    .await?;

    if let Some(id) = id_option {
        if !user.role.is_admin() && user.id != id.into() {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to delete this user!".to_string(),
            ));
        }

        let mut transaction = pool.begin().await?;

        let result = if &*removal_type.removal_type == "full" {
            crate::database::models::User::remove_full(id, &mut transaction)
                .await?
        } else {
            crate::database::models::User::remove(id, &mut transaction).await?
        };

        transaction.commit().await?;

        if result.is_some() {
            Ok(HttpResponse::NoContent().body(""))
        } else {
            Ok(HttpResponse::NotFound().body(""))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("{id}/follows")]
pub async fn user_follows(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let id_option = crate::database::models::User::get_id_from_username_or_id(
        &*info.into_inner().0,
        &**pool,
    )
    .await?;

    if let Some(id) = id_option {
        if !user.role.is_admin() && user.id != id.into() {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to see the projects this user follows!".to_string(),
            ));
        }

        use futures::TryStreamExt;

        let project_ids = sqlx::query!(
            "
            SELECT mf.mod_id FROM mod_follows mf
            WHERE mf.follower_id = $1
            ",
            id as crate::database::models::ids::UserId,
        )
        .fetch_many(&**pool)
        .try_filter_map(|e| async {
            Ok(e.right()
                .map(|m| crate::database::models::ProjectId(m.mod_id)))
        })
        .try_collect::<Vec<crate::database::models::ProjectId>>()
        .await?;

        let projects: Vec<_> =
            crate::database::Project::get_many_full(project_ids, &**pool)
                .await?
                .into_iter()
                .map(Project::from)
                .collect();

        Ok(HttpResponse::Ok().json(projects))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("{id}/notifications")]
pub async fn user_notifications(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;
    let id_option = crate::database::models::User::get_id_from_username_or_id(
        &*info.into_inner().0,
        &**pool,
    )
    .await?;

    if let Some(id) = id_option {
        if !user.role.is_admin() && user.id != id.into() {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to see the notifications of this user!".to_string(),
            ));
        }

        let mut notifications: Vec<Notification> =
            crate::database::models::notification_item::Notification::get_many_user(id, &**pool)
                .await?
                .into_iter()
                .map(Into::into)
                .collect();

        notifications.sort_by(|a, b| b.created.cmp(&a.created));

        Ok(HttpResponse::Ok().json(notifications))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
