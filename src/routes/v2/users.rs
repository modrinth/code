use crate::auth::{get_user_from_headers, AuthenticationError};
use crate::database::models::User;
use crate::file_hosting::FileHost;
use crate::models::collections::{Collection, CollectionStatus};
use crate::models::notifications::Notification;
use crate::models::pats::Scopes;
use crate::models::projects::Project;
use crate::models::users::{Badges, RecipientType, RecipientWallet, Role, UserId};
use crate::queue::payouts::{PayoutAmount, PayoutItem, PayoutsQueue};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::routes::read_from_payload;
use crate::util::validate::validation_errors_to_string;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(user_auth_get);
    cfg.service(users_get);

    cfg.service(
        web::scope("user")
            .service(user_get)
            .service(projects_list)
            .service(collections_list)
            .service(user_delete)
            .service(user_edit)
            .service(user_icon_edit)
            .service(user_notifications)
            .service(user_follows)
            .service(user_payouts)
            .service(user_payouts_request),
    );
}

#[get("user")]
pub async fn user_auth_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<deadpool_redis::Pool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (scopes, mut user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::USER_READ]),
    )
    .await?;

    if !scopes.contains(Scopes::USER_READ_EMAIL) {
        user.email = None;
    }

    if !scopes.contains(Scopes::PAYOUTS_READ) {
        user.payout_data = None;
    }

    Ok(HttpResponse::Ok().json(
        get_user_from_headers(
            &req,
            &**pool,
            &redis,
            &session_queue,
            Some(&[Scopes::USER_READ]),
        )
        .await?
        .1,
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
    redis: web::Data<deadpool_redis::Pool>,
) -> Result<HttpResponse, ApiError> {
    let user_ids = serde_json::from_str::<Vec<String>>(&ids.ids)?;

    let users_data = User::get_many(&user_ids, &**pool, &redis).await?;

    let users: Vec<crate::models::users::User> = users_data.into_iter().map(From::from).collect();

    Ok(HttpResponse::Ok().json(users))
}

#[get("{id}")]
pub async fn user_get(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<deadpool_redis::Pool>,
) -> Result<HttpResponse, ApiError> {
    let user_data = User::get(&info.into_inner().0, &**pool, &redis).await?;

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
    redis: web::Data<deadpool_redis::Pool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let id_option = User::get(&info.into_inner().0, &**pool, &redis).await?;

    if let Some(id) = id_option.map(|x| x.id) {
        let user_id: UserId = id.into();

        let can_view_private = user
            .map(|y| y.role.is_mod() || y.id == user_id)
            .unwrap_or(false);

        let project_data = User::get_projects(id, &**pool).await?;

        let response: Vec<_> =
            crate::database::Project::get_many_ids(&project_data, &**pool, &redis)
                .await?
                .into_iter()
                .filter(|x| can_view_private || x.inner.status.is_searchable())
                .map(Project::from)
                .collect();

        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("{user_id}/collections")]
pub async fn collections_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<deadpool_redis::Pool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::COLLECTION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let id_option = User::get(&info.into_inner().0, &**pool, &redis).await?;

    if let Some(id) = id_option.map(|x| x.id) {
        let user_id: UserId = id.into();

        let can_view_private = user
            .map(|y| y.role.is_mod() || y.id == user_id)
            .unwrap_or(false);

        let project_data = User::get_collections(id, &**pool).await?;

        let response: Vec<_> =
            crate::database::models::Collection::get_many(&project_data, &**pool, &redis)
                .await?
                .into_iter()
                .filter(|x| can_view_private || matches!(x.status, CollectionStatus::Listed))
                .map(Collection::from)
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
    #[validate(length(min = 1, max = 39), regex = "RE_URL_SAFE")]
    pub username: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 64), regex = "RE_URL_SAFE")]
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 160))]
    pub bio: Option<Option<String>>,
    pub role: Option<Role>,
    pub badges: Option<Badges>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate]
    pub payout_data: Option<Option<EditPayoutData>>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct EditPayoutData {
    pub payout_wallet: RecipientWallet,
    pub payout_wallet_type: RecipientType,
    #[validate(length(max = 128))]
    pub payout_address: String,
}

#[patch("{id}")]
pub async fn user_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    new_user: web::Json<EditUser>,
    pool: web::Data<PgPool>,
    redis: web::Data<deadpool_redis::Pool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (scopes, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::USER_WRITE]),
    )
    .await?;

    new_user
        .validate()
        .map_err(|err| ApiError::Validation(validation_errors_to_string(err, None)))?;

    let id_option = User::get(&info.into_inner().0, &**pool, &redis).await?;

    if let Some(actual_user) = id_option {
        let id = actual_user.id;
        let user_id: UserId = id.into();

        if user.id == user_id || user.role.is_mod() {
            let mut transaction = pool.begin().await?;

            if let Some(username) = &new_user.username {
                let existing_user_id_option = User::get(username, &**pool, &redis).await?;

                if existing_user_id_option
                    .map(|x| UserId::from(x.id))
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
                        "Username {username} is taken!"
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

            if let Some(badges) = &new_user.badges {
                if !user.role.is_admin() {
                    return Err(ApiError::CustomAuthentication(
                        "You do not have the permissions to edit the badges of this user!"
                            .to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE users
                    SET badges = $1
                    WHERE (id = $2)
                    ",
                    badges.bits() as i64,
                    id as crate::database::models::ids::UserId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(payout_data) = &new_user.payout_data {
                if let Some(payout_data) = payout_data {
                    if payout_data.payout_wallet_type == RecipientType::UserHandle
                        && payout_data.payout_wallet == RecipientWallet::Paypal
                    {
                        return Err(ApiError::InvalidInput(
                            "You cannot use a paypal wallet with a user handle!".to_string(),
                        ));
                    }

                    if !scopes.contains(Scopes::PAYOUTS_WRITE) {
                        return Err(ApiError::Authentication(
                            AuthenticationError::InvalidCredentials,
                        ));
                    }

                    if !match payout_data.payout_wallet_type {
                        RecipientType::Email => {
                            validator::validate_email(&payout_data.payout_address)
                        }
                        RecipientType::Phone => {
                            validator::validate_phone(&payout_data.payout_address)
                        }
                        RecipientType::UserHandle => true,
                    } {
                        return Err(ApiError::InvalidInput(
                            "Invalid wallet specified!".to_string(),
                        ));
                    }

                    let results = sqlx::query!(
                        "
                        SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND email IS NULL)
                        ",
                        id as crate::database::models::ids::UserId,
                    )
                    .fetch_one(&mut *transaction)
                    .await?;

                    if results.exists.unwrap_or(false) {
                        return Err(ApiError::InvalidInput(
                            "You must have an email set on your Modrinth account to enroll in the monetization program!"
                                .to_string(),
                        ));
                    }

                    sqlx::query!(
                        "
                        UPDATE users
                        SET payout_wallet = $1, payout_wallet_type = $2, payout_address = $3
                        WHERE (id = $4)
                        ",
                        payout_data.payout_wallet.as_str(),
                        payout_data.payout_wallet_type.as_str(),
                        payout_data.payout_address,
                        id as crate::database::models::ids::UserId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                } else {
                    sqlx::query!(
                        "
                        UPDATE users
                        SET payout_wallet = NULL, payout_wallet_type = NULL, payout_address = NULL
                        WHERE (id = $1)
                        ",
                        id as crate::database::models::ids::UserId,
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }

            User::clear_caches(&[(id, Some(actual_user.username))], &redis).await?;
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
#[allow(clippy::too_many_arguments)]
pub async fn user_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<deadpool_redis::Pool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    if let Some(content_type) = crate::util::ext::get_image_content_type(&ext.ext) {
        let cdn_url = dotenvy::var("CDN_URL")?;
        let user = get_user_from_headers(
            &req,
            &**pool,
            &redis,
            &session_queue,
            Some(&[Scopes::USER_WRITE]),
        )
        .await?
        .1;
        let id_option = User::get(&info.into_inner().0, &**pool, &redis).await?;

        if let Some(actual_user) = id_option {
            if user.id != actual_user.id.into() && !user.role.is_mod() {
                return Err(ApiError::CustomAuthentication(
                    "You don't have permission to edit this user's icon.".to_string(),
                ));
            }

            let icon_url = actual_user.avatar_url;
            let user_id: UserId = actual_user.id.into();

            if let Some(icon) = icon_url {
                let name = icon.split(&format!("{cdn_url}/")).nth(1);

                if let Some(icon_path) = name {
                    file_host.delete_file_version("", icon_path).await?;
                }
            }

            let bytes =
                read_from_payload(&mut payload, 2097152, "Icons must be smaller than 2MiB").await?;

            let hash = sha1::Sha1::from(&bytes).hexdigest();
            let upload_data = file_host
                .upload_file(
                    content_type,
                    &format!("user/{}/{}.{}", user_id, hash, ext.ext),
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
                actual_user.id as crate::database::models::ids::UserId,
            )
            .execute(&**pool)
            .await?;
            User::clear_caches(&[(actual_user.id, None)], &redis).await?;

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
    redis: web::Data<deadpool_redis::Pool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::USER_DELETE]),
    )
    .await?
    .1;
    let id_option = User::get(&info.into_inner().0, &**pool, &redis).await?;

    if let Some(id) = id_option.map(|x| x.id) {
        if !user.role.is_admin() && user.id != id.into() {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to delete this user!".to_string(),
            ));
        }

        let mut transaction = pool.begin().await?;

        let result = User::remove(
            id,
            removal_type.removal_type == "full",
            &mut transaction,
            &redis,
        )
        .await?;

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
    redis: web::Data<deadpool_redis::Pool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::USER_READ]),
    )
    .await?
    .1;
    let id_option = User::get(&info.into_inner().0, &**pool, &redis).await?;

    if let Some(id) = id_option.map(|x| x.id) {
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
            crate::database::Project::get_many_ids(&project_ids, &**pool, &redis)
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
    redis: web::Data<deadpool_redis::Pool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::NOTIFICATION_READ]),
    )
    .await?
    .1;
    let id_option = User::get(&info.into_inner().0, &**pool, &redis).await?;

    if let Some(id) = id_option.map(|x| x.id) {
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

#[derive(Serialize)]
pub struct Payout {
    pub created: DateTime<Utc>,
    pub amount: Decimal,
    pub status: String,
}

#[get("{id}/payouts")]
pub async fn user_payouts(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<deadpool_redis::Pool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PAYOUTS_READ]),
    )
    .await?
    .1;
    let id_option = User::get(&info.into_inner().0, &**pool, &redis).await?;

    if let Some(id) = id_option.map(|x| x.id) {
        if !user.role.is_admin() && user.id != id.into() {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to see the payouts of this user!".to_string(),
            ));
        }

        let (all_time, last_month, payouts) = futures::future::try_join3(
            sqlx::query!(
                "
                SELECT SUM(pv.amount) amount
                FROM payouts_values pv
                WHERE pv.user_id = $1
                ",
                id as crate::database::models::UserId
            )
            .fetch_one(&**pool),
            sqlx::query!(
                "
                SELECT SUM(pv.amount) amount
                FROM payouts_values pv
                WHERE pv.user_id = $1 AND created > NOW() - '1 month'::interval
                ",
                id as crate::database::models::UserId
            )
            .fetch_one(&**pool),
            sqlx::query!(
                "
                SELECT hp.created, hp.amount, hp.status
                FROM historical_payouts hp
                WHERE hp.user_id = $1
                ORDER BY hp.created DESC
                ",
                id as crate::database::models::UserId
            )
            .fetch_many(&**pool)
            .try_filter_map(|e| async {
                Ok(e.right().map(|row| Payout {
                    created: row.created,
                    amount: row.amount,
                    status: row.status,
                }))
            })
            .try_collect::<Vec<Payout>>(),
        )
        .await?;

        use futures::TryStreamExt;

        Ok(HttpResponse::Ok().json(json!({
            "all_time": all_time.amount,
            "last_month": last_month.amount,
            "payouts": payouts,
        })))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Deserialize)]
pub struct PayoutData {
    amount: Decimal,
}

#[post("{id}/payouts")]
pub async fn user_payouts_request(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    data: web::Json<PayoutData>,
    payouts_queue: web::Data<Mutex<PayoutsQueue>>,
    redis: web::Data<deadpool_redis::Pool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let mut payouts_queue = payouts_queue.lock().await;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PAYOUTS_WRITE]),
    )
    .await?
    .1;
    let id_option = User::get(&info.into_inner().0, &**pool, &redis).await?;

    if let Some(id) = id_option.map(|x| x.id) {
        if !user.role.is_admin() && user.id != id.into() {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to request payouts of this user!".to_string(),
            ));
        }

        if let Some(payouts_data) = user.payout_data {
            if let Some(payout_address) = payouts_data.payout_address {
                if let Some(payout_wallet_type) = payouts_data.payout_wallet_type {
                    if let Some(payout_wallet) = payouts_data.payout_wallet {
                        return if data.amount < payouts_data.balance {
                            let mut transaction = pool.begin().await?;

                            let leftover = payouts_queue
                                .send_payout(PayoutItem {
                                    amount: PayoutAmount {
                                        currency: "USD".to_string(),
                                        value: data.amount,
                                    },
                                    receiver: payout_address,
                                    note: "Payment from Modrinth creator monetization program"
                                        .to_string(),
                                    recipient_type: payout_wallet_type.to_string().to_uppercase(),
                                    recipient_wallet: payout_wallet.as_str_api().to_string(),
                                    sender_item_id: format!(
                                        "{}-{}",
                                        UserId::from(id),
                                        Utc::now().timestamp()
                                    ),
                                })
                                .await?;

                            sqlx::query!(
                                "
                                INSERT INTO historical_payouts (user_id, amount, status)
                                VALUES ($1, $2, $3)
                                ",
                                id as crate::database::models::ids::UserId,
                                data.amount,
                                "success"
                            )
                            .execute(&mut *transaction)
                            .await?;

                            sqlx::query!(
                                "
                                UPDATE users
                                SET balance = balance - $1
                                WHERE id = $2
                                ",
                                data.amount - leftover,
                                id as crate::database::models::ids::UserId
                            )
                            .execute(&mut *transaction)
                            .await?;
                            User::clear_caches(&[(id, None)], &redis).await?;

                            transaction.commit().await?;

                            Ok(HttpResponse::NoContent().body(""))
                        } else {
                            Err(ApiError::InvalidInput(
                                "You do not have enough funds to make this payout!".to_string(),
                            ))
                        };
                    }
                }
            }
        }

        Err(ApiError::InvalidInput(
            "You are not enrolled in the payouts program yet!".to_string(),
        ))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
