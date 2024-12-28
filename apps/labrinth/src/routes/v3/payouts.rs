use crate::auth::validate::get_user_record_from_bearer_token;
use crate::auth::{get_user_from_headers, AuthenticationError};
use crate::database::models::generate_payout_id;
use crate::database::redis::RedisPool;
use crate::models::ids::PayoutId;
use crate::models::pats::Scopes;
use crate::models::payouts::{PayoutMethodType, PayoutStatus};
use crate::queue::payouts::{make_aditude_request, PayoutsQueue};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use chrono::{Datelike, Duration, TimeZone, Utc, Weekday};
use hex::ToHex;
use hmac::{Hmac, Mac, NewMac};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use sqlx::PgPool;
use std::collections::HashMap;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("payout")
            .service(paypal_webhook)
            .service(tremendous_webhook)
            .service(user_payouts)
            .service(create_payout)
            .service(cancel_payout)
            .service(payment_methods)
            .service(get_balance)
            .service(platform_revenue),
    );
}

#[post("_paypal")]
pub async fn paypal_webhook(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    payouts: web::Data<PayoutsQueue>,
    body: String,
) -> Result<HttpResponse, ApiError> {
    let auth_algo = req
        .headers()
        .get("PAYPAL-AUTH-ALGO")
        .and_then(|x| x.to_str().ok())
        .ok_or_else(|| {
            ApiError::InvalidInput("missing auth algo".to_string())
        })?;
    let cert_url = req
        .headers()
        .get("PAYPAL-CERT-URL")
        .and_then(|x| x.to_str().ok())
        .ok_or_else(|| {
            ApiError::InvalidInput("missing cert url".to_string())
        })?;
    let transmission_id = req
        .headers()
        .get("PAYPAL-TRANSMISSION-ID")
        .and_then(|x| x.to_str().ok())
        .ok_or_else(|| {
            ApiError::InvalidInput("missing transmission ID".to_string())
        })?;
    let transmission_sig = req
        .headers()
        .get("PAYPAL-TRANSMISSION-SIG")
        .and_then(|x| x.to_str().ok())
        .ok_or_else(|| {
            ApiError::InvalidInput("missing transmission sig".to_string())
        })?;
    let transmission_time = req
        .headers()
        .get("PAYPAL-TRANSMISSION-TIME")
        .and_then(|x| x.to_str().ok())
        .ok_or_else(|| {
            ApiError::InvalidInput("missing transmission time".to_string())
        })?;

    #[derive(Deserialize)]
    struct WebHookResponse {
        verification_status: String,
    }

    let webhook_res = payouts
        .make_paypal_request::<(), WebHookResponse>(
            Method::POST,
            "notifications/verify-webhook-signature",
            None,
            // This is needed as serde re-orders fields, which causes the validation to fail for PayPal.
            Some(format!(
                "{{
                    \"auth_algo\": \"{auth_algo}\",
                    \"cert_url\": \"{cert_url}\",
                    \"transmission_id\": \"{transmission_id}\",
                    \"transmission_sig\": \"{transmission_sig}\",
                    \"transmission_time\": \"{transmission_time}\",
                    \"webhook_id\": \"{}\",
                    \"webhook_event\": {body}
                }}",
                dotenvy::var("PAYPAL_WEBHOOK_ID")?
            )),
            None,
        )
        .await?;

    if &webhook_res.verification_status != "SUCCESS" {
        return Err(ApiError::InvalidInput(
            "Invalid webhook signature".to_string(),
        ));
    }

    #[derive(Deserialize)]
    struct PayPalResource {
        pub payout_item_id: String,
    }

    #[derive(Deserialize)]
    struct PayPalWebhook {
        pub event_type: String,
        pub resource: PayPalResource,
    }

    let webhook = serde_json::from_str::<PayPalWebhook>(&body)?;

    match &*webhook.event_type {
        "PAYMENT.PAYOUTS-ITEM.BLOCKED"
        | "PAYMENT.PAYOUTS-ITEM.DENIED"
        | "PAYMENT.PAYOUTS-ITEM.REFUNDED"
        | "PAYMENT.PAYOUTS-ITEM.RETURNED"
        | "PAYMENT.PAYOUTS-ITEM.CANCELED" => {
            let mut transaction = pool.begin().await?;

            let result = sqlx::query!(
                "SELECT user_id, amount, fee FROM payouts WHERE platform_id = $1 AND status = $2",
                webhook.resource.payout_item_id,
                PayoutStatus::InTransit.as_str()
            )
            .fetch_optional(&mut *transaction)
            .await?;

            if let Some(result) = result {
                sqlx::query!(
                    "
                    UPDATE payouts
                    SET status = $1
                    WHERE platform_id = $2
                    ",
                    if &*webhook.event_type == "PAYMENT.PAYOUTS-ITEM.CANCELED" {
                        PayoutStatus::Cancelled
                    } else {
                        PayoutStatus::Failed
                    }
                    .as_str(),
                    webhook.resource.payout_item_id
                )
                .execute(&mut *transaction)
                .await?;

                transaction.commit().await?;

                crate::database::models::user_item::User::clear_caches(
                    &[(crate::database::models::UserId(result.user_id), None)],
                    &redis,
                )
                .await?;
            }
        }
        "PAYMENT.PAYOUTS-ITEM.SUCCEEDED" => {
            let mut transaction = pool.begin().await?;
            sqlx::query!(
                "
                UPDATE payouts
                SET status = $1
                WHERE platform_id = $2
                ",
                PayoutStatus::Success.as_str(),
                webhook.resource.payout_item_id
            )
            .execute(&mut *transaction)
            .await?;
            transaction.commit().await?;
        }
        _ => {}
    }

    Ok(HttpResponse::NoContent().finish())
}

#[post("_tremendous")]
pub async fn tremendous_webhook(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    body: String,
) -> Result<HttpResponse, ApiError> {
    let signature = req
        .headers()
        .get("Tremendous-Webhook-Signature")
        .and_then(|x| x.to_str().ok())
        .and_then(|x| x.split('=').next_back())
        .ok_or_else(|| {
            ApiError::InvalidInput("missing webhook signature".to_string())
        })?;

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(
        dotenvy::var("TREMENDOUS_PRIVATE_KEY")?.as_bytes(),
    )
    .map_err(|_| ApiError::Payments("error initializing HMAC".to_string()))?;
    mac.update(body.as_bytes());
    let request_signature = mac.finalize().into_bytes().encode_hex::<String>();

    if &*request_signature != signature {
        return Err(ApiError::InvalidInput(
            "Invalid webhook signature".to_string(),
        ));
    }

    #[derive(Deserialize)]
    pub struct TremendousResource {
        pub id: String,
    }

    #[derive(Deserialize)]
    struct TremendousPayload {
        pub resource: TremendousResource,
    }

    #[derive(Deserialize)]
    struct TremendousWebhook {
        pub event: String,
        pub payload: TremendousPayload,
    }

    let webhook = serde_json::from_str::<TremendousWebhook>(&body)?;

    match &*webhook.event {
        "REWARDS.CANCELED" | "REWARDS.DELIVERY.FAILED" => {
            let mut transaction = pool.begin().await?;

            let result = sqlx::query!(
                "SELECT user_id, amount, fee FROM payouts WHERE platform_id = $1 AND status = $2",
                webhook.payload.resource.id,
                PayoutStatus::InTransit.as_str()
            )
            .fetch_optional(&mut *transaction)
            .await?;

            if let Some(result) = result {
                sqlx::query!(
                    "
                    UPDATE payouts
                    SET status = $1
                    WHERE platform_id = $2
                    ",
                    if &*webhook.event == "REWARDS.CANCELED" {
                        PayoutStatus::Cancelled
                    } else {
                        PayoutStatus::Failed
                    }
                    .as_str(),
                    webhook.payload.resource.id
                )
                .execute(&mut *transaction)
                .await?;

                transaction.commit().await?;

                crate::database::models::user_item::User::clear_caches(
                    &[(crate::database::models::UserId(result.user_id), None)],
                    &redis,
                )
                .await?;
            }
        }
        "REWARDS.DELIVERY.SUCCEEDED" => {
            let mut transaction = pool.begin().await?;
            sqlx::query!(
                "
                UPDATE payouts
                SET status = $1
                WHERE platform_id = $2
                ",
                PayoutStatus::Success.as_str(),
                webhook.payload.resource.id
            )
            .execute(&mut *transaction)
            .await?;
            transaction.commit().await?;
        }
        _ => {}
    }

    Ok(HttpResponse::NoContent().finish())
}

#[get("")]
pub async fn user_payouts(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
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

    let payout_ids =
        crate::database::models::payout_item::Payout::get_all_for_user(
            user.id.into(),
            &**pool,
        )
        .await?;
    let payouts = crate::database::models::payout_item::Payout::get_many(
        &payout_ids,
        &**pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(
        payouts
            .into_iter()
            .map(crate::models::payouts::Payout::from)
            .collect::<Vec<_>>(),
    ))
}

#[derive(Deserialize)]
pub struct Withdrawal {
    #[serde(with = "rust_decimal::serde::float")]
    amount: Decimal,
    method: PayoutMethodType,
    method_id: String,
}

#[post("")]
pub async fn create_payout(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    body: web::Json<Withdrawal>,
    session_queue: web::Data<AuthQueue>,
    payouts_queue: web::Data<PayoutsQueue>,
) -> Result<HttpResponse, ApiError> {
    let (scopes, user) = get_user_record_from_bearer_token(
        &req,
        None,
        &**pool,
        &redis,
        &session_queue,
    )
    .await?
    .ok_or_else(|| {
        ApiError::Authentication(AuthenticationError::InvalidCredentials)
    })?;

    if !scopes.contains(Scopes::PAYOUTS_WRITE) {
        return Err(ApiError::Authentication(
            AuthenticationError::InvalidCredentials,
        ));
    }

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        SELECT balance FROM users WHERE id = $1 FOR UPDATE
        ",
        user.id.0
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let balance = get_user_balance(user.id, &pool).await?;
    if balance.available < body.amount || body.amount < Decimal::ZERO {
        return Err(ApiError::InvalidInput(
            "You do not have enough funds to make this payout!".to_string(),
        ));
    }

    let payout_method = payouts_queue
        .get_payout_methods()
        .await?
        .into_iter()
        .find(|x| x.id == body.method_id)
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "Invalid payment method specified!".to_string(),
            )
        })?;

    let fee = std::cmp::min(
        std::cmp::max(
            payout_method.fee.min,
            payout_method.fee.percentage * body.amount,
        ),
        payout_method.fee.max.unwrap_or(Decimal::MAX),
    );

    let transfer = (body.amount - fee).round_dp(2);
    if transfer <= Decimal::ZERO {
        return Err(ApiError::InvalidInput(
            "You need to withdraw more to cover the fee!".to_string(),
        ));
    }

    let payout_id = generate_payout_id(&mut transaction).await?;

    let payout_item = match body.method {
        PayoutMethodType::Venmo | PayoutMethodType::PayPal => {
            let (wallet, wallet_type, address, display_address) = if body.method
                == PayoutMethodType::Venmo
            {
                if let Some(venmo) = user.venmo_handle {
                    ("Venmo", "user_handle", venmo.clone(), venmo)
                } else {
                    return Err(ApiError::InvalidInput(
                        "Venmo address has not been set for account!"
                            .to_string(),
                    ));
                }
            } else if let Some(paypal_id) = user.paypal_id {
                if let Some(paypal_country) = user.paypal_country {
                    if &*paypal_country == "US"
                        && &*body.method_id != "paypal_us"
                    {
                        return Err(ApiError::InvalidInput(
                            "Please use the US PayPal transfer option!"
                                .to_string(),
                        ));
                    } else if &*paypal_country != "US"
                        && &*body.method_id == "paypal_us"
                    {
                        return Err(ApiError::InvalidInput(
                                "Please use the International PayPal transfer option!".to_string(),
                            ));
                    }

                    (
                        "PayPal",
                        "paypal_id",
                        paypal_id.clone(),
                        user.paypal_email.unwrap_or(paypal_id),
                    )
                } else {
                    return Err(ApiError::InvalidInput(
                        "Please re-link your PayPal account!".to_string(),
                    ));
                }
            } else {
                return Err(ApiError::InvalidInput(
                    "You have not linked a PayPal account!".to_string(),
                ));
            };

            #[derive(Deserialize)]
            struct PayPalLink {
                href: String,
            }

            #[derive(Deserialize)]
            struct PayoutsResponse {
                pub links: Vec<PayPalLink>,
            }

            let mut payout_item =
                crate::database::models::payout_item::Payout {
                    id: payout_id,
                    user_id: user.id,
                    created: Utc::now(),
                    status: PayoutStatus::InTransit,
                    amount: transfer,
                    fee: Some(fee),
                    method: Some(body.method),
                    method_address: Some(display_address),
                    platform_id: None,
                };

            let res: PayoutsResponse = payouts_queue.make_paypal_request(
                Method::POST,
                "payments/payouts",
                Some(
                    json! ({
                        "sender_batch_header": {
                            "sender_batch_id": format!("{}-payouts", Utc::now().to_rfc3339()),
                            "email_subject": "You have received a payment from Modrinth!",
                            "email_message": "Thank you for creating projects on Modrinth. Please claim this payment within 30 days.",
                        },
                        "items": [{
                            "amount": {
                                "currency": "USD",
                                "value": transfer.to_string()
                            },
                            "receiver": address,
                            "note": "Payment from Modrinth creator monetization program",
                            "recipient_type": wallet_type,
                            "recipient_wallet": wallet,
                            "sender_item_id": crate::models::ids::PayoutId::from(payout_id),
                        }]
                    })
                ),
                None,
                None
            ).await?;

            if let Some(link) = res.links.first() {
                #[derive(Deserialize)]
                struct PayoutItem {
                    pub payout_item_id: String,
                }

                #[derive(Deserialize)]
                struct PayoutData {
                    pub items: Vec<PayoutItem>,
                }

                if let Ok(res) = payouts_queue
                    .make_paypal_request::<(), PayoutData>(
                        Method::GET,
                        &link.href,
                        None,
                        None,
                        Some(true),
                    )
                    .await
                {
                    if let Some(data) = res.items.first() {
                        payout_item.platform_id =
                            Some(data.payout_item_id.clone());
                    }
                }
            }

            payout_item
        }
        PayoutMethodType::Tremendous => {
            if let Some(email) = user.email {
                if user.email_verified {
                    let mut payout_item =
                        crate::database::models::payout_item::Payout {
                            id: payout_id,
                            user_id: user.id,
                            created: Utc::now(),
                            status: PayoutStatus::InTransit,
                            amount: transfer,
                            fee: Some(fee),
                            method: Some(PayoutMethodType::Tremendous),
                            method_address: Some(email.clone()),
                            platform_id: None,
                        };

                    #[derive(Deserialize)]
                    struct Reward {
                        pub id: String,
                    }

                    #[derive(Deserialize)]
                    struct Order {
                        pub rewards: Vec<Reward>,
                    }

                    #[derive(Deserialize)]
                    struct TremendousResponse {
                        pub order: Order,
                    }

                    let res: TremendousResponse = payouts_queue
                        .make_tremendous_request(
                            Method::POST,
                            "orders",
                            Some(json! ({
                                "payment": {
                                    "funding_source_id": "BALANCE",
                                },
                                "rewards": [{
                                    "value": {
                                        "denomination": transfer
                                    },
                                    "delivery": {
                                        "method": "EMAIL"
                                    },
                                    "recipient": {
                                        "name": user.username,
                                        "email": email
                                    },
                                    "products": [
                                        &body.method_id,
                                    ],
                                    "campaign_id": dotenvy::var("TREMENDOUS_CAMPAIGN_ID")?,
                                }]
                            })),
                        )
                        .await?;

                    if let Some(reward) = res.order.rewards.first() {
                        payout_item.platform_id = Some(reward.id.clone())
                    }

                    payout_item
                } else {
                    return Err(ApiError::InvalidInput(
                        "You must verify your account email to proceed!"
                            .to_string(),
                    ));
                }
            } else {
                return Err(ApiError::InvalidInput(
                    "You must add an email to your account to proceed!"
                        .to_string(),
                ));
            }
        }
        PayoutMethodType::Unknown => {
            return Err(ApiError::Payments(
                "Invalid payment method specified!".to_string(),
            ))
        }
    };

    payout_item.insert(&mut transaction).await?;

    transaction.commit().await?;
    crate::database::models::User::clear_caches(&[(user.id, None)], &redis)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[delete("{id}")]
pub async fn cancel_payout(
    info: web::Path<(PayoutId,)>,
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    payouts: web::Data<PayoutsQueue>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PAYOUTS_WRITE]),
    )
    .await?
    .1;

    let id = info.into_inner().0;
    let payout =
        crate::database::models::payout_item::Payout::get(id.into(), &**pool)
            .await?;

    if let Some(payout) = payout {
        if payout.user_id != user.id.into() && !user.role.is_admin() {
            return Ok(HttpResponse::NotFound().finish());
        }

        if let Some(platform_id) = payout.platform_id {
            if let Some(method) = payout.method {
                if payout.status != PayoutStatus::InTransit {
                    return Err(ApiError::InvalidInput(
                        "Payout cannot be cancelled!".to_string(),
                    ));
                }

                match method {
                    PayoutMethodType::Venmo | PayoutMethodType::PayPal => {
                        payouts
                            .make_paypal_request::<(), ()>(
                                Method::POST,
                                &format!(
                                    "payments/payouts-item/{}/cancel",
                                    platform_id
                                ),
                                None,
                                None,
                                None,
                            )
                            .await?;
                    }
                    PayoutMethodType::Tremendous => {
                        payouts
                            .make_tremendous_request::<(), ()>(
                                Method::POST,
                                &format!("rewards/{}/cancel", platform_id),
                                None,
                            )
                            .await?;
                    }
                    PayoutMethodType::Unknown => {
                        return Err(ApiError::InvalidInput(
                            "Payout cannot be cancelled!".to_string(),
                        ))
                    }
                }

                let mut transaction = pool.begin().await?;
                sqlx::query!(
                    "
                    UPDATE payouts
                    SET status = $1
                    WHERE platform_id = $2
                    ",
                    PayoutStatus::Cancelling.as_str(),
                    platform_id
                )
                .execute(&mut *transaction)
                .await?;
                transaction.commit().await?;

                Ok(HttpResponse::NoContent().finish())
            } else {
                Err(ApiError::InvalidInput(
                    "Payout cannot be cancelled!".to_string(),
                ))
            }
        } else {
            Err(ApiError::InvalidInput(
                "Payout cannot be cancelled!".to_string(),
            ))
        }
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[derive(Deserialize)]
pub struct MethodFilter {
    pub country: Option<String>,
}

#[get("methods")]
pub async fn payment_methods(
    payouts_queue: web::Data<PayoutsQueue>,
    filter: web::Query<MethodFilter>,
) -> Result<HttpResponse, ApiError> {
    let methods = payouts_queue
        .get_payout_methods()
        .await?
        .into_iter()
        .filter(|x| {
            let mut val = true;

            if let Some(country) = &filter.country {
                val &= x.supported_countries.contains(country);
            }

            val
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(methods))
}

#[derive(Serialize)]
pub struct UserBalance {
    pub available: Decimal,
    pub pending: Decimal,
}

#[get("balance")]
pub async fn get_balance(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
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

    let balance = get_user_balance(user.id.into(), &pool).await?;

    Ok(HttpResponse::Ok().json(balance))
}

async fn get_user_balance(
    user_id: crate::database::models::ids::UserId,
    pool: &PgPool,
) -> Result<UserBalance, sqlx::Error> {
    let available = sqlx::query!(
        "
        SELECT SUM(amount)
        FROM payouts_values
        WHERE user_id = $1 AND date_available <= NOW()
        ",
        user_id.0
    )
    .fetch_optional(pool)
    .await?;

    let pending = sqlx::query!(
        "
        SELECT SUM(amount)
        FROM payouts_values
        WHERE user_id = $1 AND date_available > NOW()
        ",
        user_id.0
    )
    .fetch_optional(pool)
    .await?;

    let withdrawn = sqlx::query!(
        "
        SELECT SUM(amount) amount, SUM(fee) fee
        FROM payouts
        WHERE user_id = $1 AND (status = 'success' OR status = 'in-transit')
        ",
        user_id.0
    )
    .fetch_optional(pool)
    .await?;

    let available = available
        .map(|x| x.sum.unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO);
    let pending = pending
        .map(|x| x.sum.unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO);
    let (withdrawn, fees) = withdrawn
        .map(|x| {
            (
                x.amount.unwrap_or(Decimal::ZERO),
                x.fee.unwrap_or(Decimal::ZERO),
            )
        })
        .unwrap_or((Decimal::ZERO, Decimal::ZERO));

    Ok(UserBalance {
        available: available.round_dp(16)
            - withdrawn.round_dp(16)
            - fees.round_dp(16),
        pending,
    })
}

#[derive(Serialize, Deserialize)]
pub struct RevenueResponse {
    pub all_time: Decimal,
    pub data: Vec<RevenueData>,
}

#[derive(Serialize, Deserialize)]
pub struct RevenueData {
    pub time: u64,
    pub revenue: Decimal,
    pub creator_revenue: Decimal,
}

#[get("platform_revenue")]
pub async fn platform_revenue(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let mut redis = redis.connect().await?;

    const PLATFORM_REVENUE_NAMESPACE: &str = "platform_revenue";

    let res: Option<RevenueResponse> = redis
        .get_deserialized_from_json(PLATFORM_REVENUE_NAMESPACE, "0")
        .await?;

    if let Some(res) = res {
        return Ok(HttpResponse::Ok().json(res));
    }

    let all_time_payouts = sqlx::query!(
        "
        SELECT SUM(amount) from payouts_values
        ",
    )
    .fetch_optional(&**pool)
    .await?
    .and_then(|x| x.sum)
    .unwrap_or(Decimal::ZERO);

    let points = make_aditude_request(
        &["METRIC_REVENUE", "METRIC_IMPRESSIONS"],
        "30d",
        "1d",
    )
    .await?;

    let mut points_map = HashMap::new();

    for point in points {
        for point in point.points_list {
            let entry =
                points_map.entry(point.time.seconds).or_insert((None, None));

            if let Some(revenue) = point.metric.revenue {
                entry.0 = Some(revenue);
            }

            if let Some(impressions) = point.metric.impressions {
                entry.1 = Some(impressions);
            }
        }
    }

    let mut revenue_data = Vec::new();
    let now = Utc::now();

    for i in 1..=30 {
        let time = now - Duration::days(i);
        let start = time
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();

        if let Some((revenue, impressions)) = points_map.remove(&(start as u64))
        {
            // Before 9/5/24, when legacy payouts were in effect.
            if start >= 1725494400 {
                let revenue = revenue.unwrap_or(Decimal::ZERO);
                let impressions = impressions.unwrap_or(0);

                // Modrinth's share of ad revenue
                let modrinth_cut = Decimal::from(1) / Decimal::from(4);
                // Clean.io fee (ad antimalware). Per 1000 impressions.
                let clean_io_fee = Decimal::from(8) / Decimal::from(1000);

                let net_revenue = revenue
                    - (clean_io_fee * Decimal::from(impressions)
                        / Decimal::from(1000));

                let payout = net_revenue * (Decimal::from(1) - modrinth_cut);

                revenue_data.push(RevenueData {
                    time: start as u64,
                    revenue: net_revenue,
                    creator_revenue: payout,
                });

                continue;
            }
        }

        revenue_data.push(get_legacy_data_point(start as u64));
    }

    let res = RevenueResponse {
        all_time: all_time_payouts,
        data: revenue_data,
    };

    redis
        .set_serialized_to_json(
            PLATFORM_REVENUE_NAMESPACE,
            0,
            &res,
            Some(60 * 60),
        )
        .await?;

    Ok(HttpResponse::Ok().json(res))
}

fn get_legacy_data_point(timestamp: u64) -> RevenueData {
    let start = Utc.timestamp_opt(timestamp as i64, 0).unwrap();

    let old_payouts_budget = Decimal::from(10_000);

    let days = Decimal::from(28);
    let weekdays = Decimal::from(20);
    let weekend_bonus = Decimal::from(5) / Decimal::from(4);

    let weekday_amount =
        old_payouts_budget / (weekdays + (weekend_bonus) * (days - weekdays));
    let weekend_amount = weekday_amount * weekend_bonus;

    let payout = match start.weekday() {
        Weekday::Sat | Weekday::Sun => weekend_amount,
        _ => weekday_amount,
    };

    RevenueData {
        time: timestamp,
        revenue: payout,
        creator_revenue: payout * (Decimal::from(9) / Decimal::from(10)),
    }
}
