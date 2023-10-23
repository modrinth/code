use crate::auth::validate::get_user_record_from_bearer_token;
use crate::database::models::User;
use crate::database::redis::RedisPool;
use crate::models::analytics::Download;
use crate::models::ids::ProjectId;
use crate::models::pats::Scopes;
use crate::models::users::{PayoutStatus, RecipientStatus};
use crate::queue::analytics::AnalyticsQueue;
use crate::queue::download::DownloadQueue;
use crate::queue::maxmind::MaxMindIndexer;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::date::get_current_tenths_of_ms;
use crate::util::guards::admin_key_guard;
use crate::util::routes::read_from_payload;
use actix_web::{patch, post, web, HttpRequest, HttpResponse};
use hex::ToHex;
use hmac::{Hmac, Mac, NewMac};
use serde::Deserialize;
use sha2::Sha256;
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use uuid::Uuid;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("admin")
            .service(count_download)
            .service(trolley_webhook),
    );
}

#[derive(Deserialize)]
pub struct DownloadBody {
    pub url: String,
    pub project_id: ProjectId,
    pub version_name: String,

    pub ip: String,
    pub headers: HashMap<String, String>,
}

// This is an internal route, cannot be used without key
#[patch("/_count-download", guard = "admin_key_guard")]
#[allow(clippy::too_many_arguments)]
pub async fn count_download(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    maxmind: web::Data<Arc<MaxMindIndexer>>,
    analytics_queue: web::Data<Arc<AnalyticsQueue>>,
    session_queue: web::Data<AuthQueue>,
    download_body: web::Json<DownloadBody>,
    download_queue: web::Data<DownloadQueue>,
) -> Result<HttpResponse, ApiError> {
    let token = download_body
        .headers
        .iter()
        .find(|x| x.0.to_lowercase() == "authorization")
        .map(|x| &**x.1);

    let user = get_user_record_from_bearer_token(&req, token, &**pool, &redis, &session_queue)
        .await
        .ok()
        .flatten();

    let project_id: crate::database::models::ids::ProjectId = download_body.project_id.into();

    let id_option = crate::models::ids::base62_impl::parse_base62(&download_body.version_name)
        .ok()
        .map(|x| x as i64);

    let (version_id, project_id, file_type) = if let Some(version) = sqlx::query!(
        "
            SELECT v.id id, v.mod_id mod_id, file_type FROM files f
            INNER JOIN versions v ON v.id = f.version_id
            WHERE f.url = $1
            ",
        download_body.url,
    )
    .fetch_optional(pool.as_ref())
    .await?
    {
        (version.id, version.mod_id, version.file_type)
    } else if let Some(version) = sqlx::query!(
        "
        SELECT id, mod_id FROM versions
        WHERE ((version_number = $1 OR id = $3) AND mod_id = $2)
        ",
        download_body.version_name,
        project_id as crate::database::models::ids::ProjectId,
        id_option
    )
    .fetch_optional(pool.as_ref())
    .await?
    {
        (version.id, version.mod_id, None)
    } else {
        return Err(ApiError::InvalidInput(
            "Specified version does not exist!".to_string(),
        ));
    };

    if file_type.is_none() {
        download_queue
            .add(
                crate::database::models::ProjectId(project_id),
                crate::database::models::VersionId(version_id),
            )
            .await;
    }

    let url = url::Url::parse(&download_body.url)
        .map_err(|_| ApiError::InvalidInput("invalid download URL specified!".to_string()))?;

    let ip = crate::routes::analytics::convert_to_ip_v6(&download_body.ip)
        .unwrap_or_else(|_| Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped());

    analytics_queue.add_download(Download {
        id: Uuid::new_v4(),
        recorded: get_current_tenths_of_ms(),
        domain: url.host_str().unwrap_or_default().to_string(),
        site_path: url.path().to_string(),
        user_id: user
            .and_then(|(scopes, x)| {
                if scopes.contains(Scopes::PERFORM_ANALYTICS) {
                    Some(x.id.0 as u64)
                } else {
                    None
                }
            })
            .unwrap_or(0),
        project_id: project_id as u64,
        version_id: version_id as u64,
        ip,
        country: maxmind.query(ip).await.unwrap_or_default(),
        user_agent: download_body
            .headers
            .get("user-agent")
            .cloned()
            .unwrap_or_default(),
        headers: download_body
            .headers
            .clone()
            .into_iter()
            .filter(|x| !crate::routes::analytics::FILTERED_HEADERS.contains(&&*x.0.to_lowercase()))
            .collect(),
    });

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Deserialize)]
pub struct TrolleyWebhook {
    model: String,
    action: String,
    body: HashMap<String, serde_json::Value>,
}

#[post("/_trolley")]
#[allow(clippy::too_many_arguments)]
pub async fn trolley_webhook(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    mut payload: web::Payload,
) -> Result<HttpResponse, ApiError> {
    if let Some(signature) = req.headers().get("X-PaymentRails-Signature") {
        let payload = read_from_payload(
            &mut payload,
            1 << 20,
            "Webhook payload exceeds the maximum of 1MiB.",
        )
        .await?;

        let mut signature = signature.to_str().ok().unwrap_or_default().split(',');
        let timestamp = signature
            .next()
            .and_then(|x| x.split('=').nth(1))
            .unwrap_or_default();
        let v1 = signature
            .next()
            .and_then(|x| x.split('=').nth(1))
            .unwrap_or_default();

        let mut mac: Hmac<Sha256> =
            Hmac::new_from_slice(dotenvy::var("TROLLEY_WEBHOOK_SIGNATURE")?.as_bytes())
                .map_err(|_| ApiError::Payments("error initializing HMAC".to_string()))?;
        mac.update(timestamp.as_bytes());
        mac.update(&payload);
        let request_signature = mac.finalize().into_bytes().encode_hex::<String>();

        if &*request_signature == v1 {
            let webhook = serde_json::from_slice::<TrolleyWebhook>(&payload)?;

            if webhook.model == "recipient" {
                #[derive(Deserialize)]
                struct Recipient {
                    pub id: String,
                    pub email: Option<String>,
                    pub status: Option<RecipientStatus>,
                }

                if let Some(body) = webhook.body.get("recipient") {
                    if let Ok(recipient) = serde_json::from_value::<Recipient>(body.clone()) {
                        let value = sqlx::query!(
                            "SELECT id FROM users WHERE trolley_id = $1",
                            recipient.id
                        )
                        .fetch_optional(&**pool)
                        .await?;

                        if let Some(user) = value {
                            let user = User::get_id(
                                crate::database::models::UserId(user.id),
                                &**pool,
                                &redis,
                            )
                            .await?;

                            if let Some(user) = user {
                                let mut transaction = pool.begin().await?;

                                if webhook.action == "deleted" {
                                    sqlx::query!(
                                        "
                                        UPDATE users
                                        SET trolley_account_status = NULL, trolley_id = NULL
                                        WHERE id = $1
                                        ",
                                        user.id.0
                                    )
                                    .execute(&mut *transaction)
                                    .await?;
                                } else {
                                    sqlx::query!(
                                        "
                                        UPDATE users
                                        SET email = $1, email_verified = $2, trolley_account_status = $3
                                        WHERE id = $4
                                        ",
                                        recipient.email.clone(),
                                        user.email_verified && recipient.email == user.email,
                                        recipient.status.map(|x| x.as_str()),
                                        user.id.0
                                    )
                                        .execute(&mut *transaction).await?;
                                }

                                transaction.commit().await?;
                                User::clear_caches(&[(user.id, None)], &redis).await?;
                            }
                        }
                    }
                }
            }

            if webhook.model == "payment" {
                #[derive(Deserialize)]
                struct Payment {
                    pub id: String,
                    pub status: PayoutStatus,
                }

                if let Some(body) = webhook.body.get("payment") {
                    if let Ok(payment) = serde_json::from_value::<Payment>(body.clone()) {
                        let value = sqlx::query!(
                            "SELECT id, amount, user_id, status FROM historical_payouts WHERE payment_id = $1",
                            payment.id
                        )
                            .fetch_optional(&**pool)
                            .await?;

                        if let Some(payout) = value {
                            let mut transaction = pool.begin().await?;

                            if payment.status.is_failed()
                                && !PayoutStatus::from_string(&payout.status).is_failed()
                            {
                                sqlx::query!(
                                    "
                                    UPDATE users
                                    SET balance = balance + $1
                                    WHERE id = $2
                                    ",
                                    payout.amount,
                                    payout.user_id,
                                )
                                .execute(&mut *transaction)
                                .await?;
                            }

                            sqlx::query!(
                                "
                                UPDATE historical_payouts
                                SET status = $1
                                WHERE payment_id = $2
                                ",
                                payment.status.as_str(),
                                payment.id,
                            )
                            .execute(&mut *transaction)
                            .await?;

                            transaction.commit().await?;
                            User::clear_caches(
                                &[(crate::database::models::UserId(payout.user_id), None)],
                                &redis,
                            )
                            .await?;
                        }
                    }
                }
            }
        }
    }

    Ok(HttpResponse::NoContent().finish())
}
