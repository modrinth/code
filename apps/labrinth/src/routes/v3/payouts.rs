use crate::auth::validate::get_user_record_from_bearer_token;
use crate::auth::{AuthenticationError, get_user_from_headers};
use crate::database::models::DBUserId;
use crate::database::models::{generate_payout_id, users_compliance};
use crate::database::redis::RedisPool;
use crate::models::ids::PayoutId;
use crate::models::pats::Scopes;
use crate::models::payouts::{PayoutMethodType, PayoutStatus, Withdrawal};
use crate::queue::payouts::PayoutsQueue;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::avalara1099;
use crate::util::error::Context;
use crate::util::gotenberg::GotenbergClient;
use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use chrono::{DateTime, Duration, Utc};
use hex::ToHex;
use hmac::{Hmac, Mac};
use modrinth_util::decimal::Decimal2dp;
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio_stream::StreamExt;
use tracing::error;

const COMPLIANCE_CHECK_DEBOUNCE: chrono::Duration =
    chrono::Duration::seconds(15);

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(paypal_webhook)
        .service(tremendous_webhook)
        .service(transaction_history)
        .service(calculate_fees)
        .service(create_payout)
        .service(cancel_payout)
        .service(payment_methods)
        .service(get_balance)
        .service(platform_revenue)
        .service(post_compliance_form);
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct RequestForm {
    form_type: users_compliance::FormType,
}

#[utoipa::path]
#[post("/compliance")]
pub async fn post_compliance_form(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    body: web::Json<RequestForm>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PAYOUTS_WRITE,
    )
    .await?
    .1;

    let user_id = DBUserId(user.id.0 as i64);

    let mut txn = pool.begin().await?;

    let maybe_compliance =
        users_compliance::UserCompliance::get_by_user_id(&mut *txn, user_id)
            .await?;

    let mut compliance = match maybe_compliance {
        Some(c) => {
            if c.signed.is_some()
                && c.form_type.is_some_and(|f| f.requires_domestic_tin_match())
                && !c.tin_matched
            {
                return Err(ApiError::InvalidInput(
                    "Your TIN/SSN did not match the IRS records. Please contact support https://support.modrinth.com".to_owned(),
                ));
            }

            c
        }
        None => users_compliance::UserCompliance {
            id: 0,
            user_id,
            requested: Utc::now(),
            signed: None,
            last_checked: Utc::now() - COMPLIANCE_CHECK_DEBOUNCE,
            external_request_id: String::new(),
            reference_id: String::new(),
            e_delivery_consented: false,
            tin_matched: false,
            form_type: Some(body.0.form_type),
            requires_manual_review: false,
        },
    };

    let result = avalara1099::request_form(user_id, body.0.form_type).await?;

    match result {
        Ok(
            ref toplevel @ avalara1099::DataWrapper {
                data:
                    avalara1099::Data {
                        r#type: _,
                        id: Some(ref request_id),
                        ref attributes,
                        links: _,
                    },
            },
        ) => {
            compliance.external_request_id = request_id.clone();
            compliance.reference_id = attributes.reference_id.clone();
            compliance.requested = Utc::now();
            compliance.e_delivery_consented = false;
            compliance.tin_matched = false;
            compliance.signed = None;
            compliance.form_type = Some(body.0.form_type);
            compliance.last_checked = Utc::now() - COMPLIANCE_CHECK_DEBOUNCE;

            compliance.upsert_partial(&mut *txn).await?;
            txn.commit().await?;

            Ok(HttpResponse::Ok().json(toplevel))
        }

        Ok(_) => {
            error!("Missing form request ID in Avalara response");
            Err(ApiError::TaxComplianceApi)
        }

        Err(json_error) => {
            error!(
                "Error sending request to Avalara: {}",
                serde_json::to_string_pretty(&json_error).unwrap()
            );
            Err(ApiError::TaxComplianceApi)
        }
    }
}

#[utoipa::path]
#[post("/_paypal")]
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

                crate::database::models::user_item::DBUser::clear_caches(
                    &[(
                        crate::database::models::DBUserId(result.user_id),
                        None,
                    )],
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

#[utoipa::path]
#[post("/_tremendous")]
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

                crate::database::models::user_item::DBUser::clear_caches(
                    &[(
                        crate::database::models::DBUserId(result.user_id),
                        None,
                    )],
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

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawalFees {
    pub net_usd: Decimal2dp,
    pub fee: Decimal2dp,
    pub exchange_rate: Option<Decimal>,
}

#[utoipa::path]
#[post("/fees")]
pub async fn calculate_fees(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    body: web::Json<Withdrawal>,
    session_queue: web::Data<AuthQueue>,
    payouts_queue: web::Data<PayoutsQueue>,
) -> Result<web::Json<WithdrawalFees>, ApiError> {
    // even though we don't use the user, we ensure they're logged in to make API calls
    let (_, _user) = get_user_record_from_bearer_token(
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

    let payout_flow = payouts_queue.create_payout_flow(body.0).await?;

    Ok(web::Json(WithdrawalFees {
        net_usd: payout_flow.net_usd,
        fee: payout_flow.total_fee_usd,
        exchange_rate: payout_flow.forex_usd_to_currency,
    }))
}

#[utoipa::path]
#[post("")]
pub async fn create_payout(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    body: web::Json<Withdrawal>,
    session_queue: web::Data<AuthQueue>,
    payouts_queue: web::Data<PayoutsQueue>,
    gotenberg: web::Data<GotenbergClient>,
) -> Result<(), ApiError> {
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
    .await
    .wrap_internal_err("failed to fetch user balance")?;

    let balance = get_user_balance(user.id, &pool)
        .await
        .wrap_internal_err("failed to calculate user balance")?;
    if balance.available < body.amount || body.amount < Decimal::ZERO {
        return Err(ApiError::InvalidInput(
            "You do not have enough funds to make this payout!".to_string(),
        ));
    }

    let requires_manual_review;

    if let Some(threshold) = tax_compliance_payout_threshold() {
        let maybe_compliance = update_compliance_status(&pool, user.id).await?;

        let (tin_matched, signed, requested, api_check_failed) =
            match maybe_compliance {
                Some(ComplianceCheck {
                    model,
                    compliance_api_check_failed,
                }) => {
                    let tin = model.tin_matched;
                    let signed = model.signed.is_some();

                    requires_manual_review = Some(model.requires_manual_review);

                    (tin, signed, true, compliance_api_check_failed)
                }
                None => {
                    requires_manual_review = None;
                    (false, false, false, false)
                }
            };

        if !(tin_matched && signed)
            && balance.withdrawn_ytd + body.amount >= threshold
        {
            // We propagate the error this way because we don't want to block payouts
            // that would be acceptable regardless of the tax form submission status
            // if the compliance API is down.

            // In this case the payout is going to be blocked, so do return that we hit an
            // error with the API, as this is more accurate than saying the form wasn't completed
            // properly as this might be wrong!
            if api_check_failed {
                return Err(ApiError::TaxComplianceApi);
            }

            return Err(ApiError::InvalidInput(match (tin_matched, signed, requested) {
                (_, false, true) => "Tax form isn't signed yet!",
                (false, true, true) => "Tax form is signed, but the Tax Identification Number/SSN didn't match the IRS records. Withdrawals are blocked until the TIN/SSN matches.",
                _ => "Tax compliance form is required to withdraw more!",
            }.to_owned()));
        }
    } else {
        requires_manual_review = None;
    }

    let requires_manual_review = if let Some(r) = requires_manual_review {
        r
    } else {
        users_compliance::UserCompliance::get_by_user_id(&**pool, user.id)
            .await?
            .is_some_and(|x| x.requires_manual_review)
    };

    if requires_manual_review {
        return Err(ApiError::InvalidInput(
            "More information is required to proceed. Please contact support (https://support.modrinth.com, support@modrinth.com)".to_string(),
        ));
    }

    let payout_flow = payouts_queue.create_payout_flow(body.0).await?;
    let payout_flow = match payout_flow.validate(balance.available) {
        Ok(flow) => flow,
        Err(err) => return Err(ApiError::InvalidInput(err.to_string())),
    };

    let payout_id = generate_payout_id(&mut transaction)
        .await
        .wrap_internal_err("failed to generate payout ID")?;

    payout_flow
        .execute(&payouts_queue, &user, payout_id, transaction, &gotenberg)
        .await?;

    crate::database::models::DBUser::clear_caches(&[(user.id, None)], &redis)
        .await
        .wrap_internal_err("failed to clear user caches")?;

    Ok(())
}

/// User performing a payout-related action.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransactionItem {
    /// User withdrew some of their available payout.
    Withdrawal {
        /// ID of the payout.
        id: PayoutId,
        /// Status of this payout.
        status: PayoutStatus,
        /// When the payout was created.
        created: DateTime<Utc>,
        /// How much the user got from this payout, excluding fees.
        amount: Decimal,
        /// How much the user paid in fees for this payout, on top of `amount`.
        fee: Option<Decimal>,
        /// What payout method type was used for this.
        method_type: Option<PayoutMethodType>,
        /// Payout-method-specific ID for the type of payout the user got.
        ///
        /// - Tremendous: the rewarded gift card ID.
        /// - Mural: the payment rail code used.
        ///   - Blockchain: `blockchain-usdc-polygon`.
        ///   - Fiat: see [`muralpay::FiatAndRailCode`].
        /// - PayPal: `paypal_us`.
        /// - Venmo: `venmo`.
        ///
        /// For legacy transactions, this may be [`None`] as we did not always
        /// store this payout info.
        method_id: Option<String>,
        /// Payout-method-specific address which the payout was sent to, like
        /// an email address.
        method_address: Option<String>,
    },
    /// User got a payout available for them to withdraw.
    PayoutAvailable {
        /// When this payout was made available for the user to withdraw.
        created: DateTime<Utc>,
        /// Where this payout came from.
        payout_source: PayoutSource,
        /// How much the payout was worth.
        amount: Decimal,
    },
}

impl TransactionItem {
    pub fn created(&self) -> DateTime<Utc> {
        match self {
            Self::Withdrawal { created, .. } => *created,
            Self::PayoutAvailable { created, .. } => *created,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PayoutSource {
    CreatorRewards,
    Affilites,
}

/// Get the history of when the authorized user got payouts available, and when
/// the user withdrew their payouts.
#[utoipa::path(responses((status = OK, body = Vec<TransactionItem>)))]
#[get("/history")]
pub async fn transaction_history(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<TransactionItem>>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PAYOUTS_READ,
    )
    .await?;

    let payout_ids =
        crate::database::models::payout_item::DBPayout::get_all_for_user(
            user.id.into(),
            &**pool,
        )
        .await?;
    let payouts = crate::database::models::payout_item::DBPayout::get_many(
        &payout_ids,
        &**pool,
    )
    .await?;
    let withdrawals =
        payouts
            .into_iter()
            .map(|payout| TransactionItem::Withdrawal {
                id: payout.id.into(),
                status: payout.status,
                created: payout.created,
                amount: payout.amount,
                fee: payout.fee,
                method_type: payout.method,
                method_id: payout.method_id,
                method_address: payout.method_address,
            });

    let mut payouts_available = sqlx::query!(
        "
        SELECT date_available, SUM(amount) AS amount
        FROM payouts_values
        WHERE user_id = $1
        AND NOW() >= date_available
        GROUP BY date_available
        ",
        DBUserId::from(user.id) as DBUserId
    )
    .fetch(&**pool)
    .map(|record| {
        let record = record
            .wrap_internal_err("failed to fetch available payout record")?;
        let amount = record.amount.unwrap_or_default();
        if amount > Decimal::ZERO {
            Ok(Some(TransactionItem::PayoutAvailable {
                created: record.date_available,
                payout_source: PayoutSource::CreatorRewards,
                amount,
            }))
        } else {
            Ok(None)
        }
    })
    .collect::<Result<Vec<_>, ApiError>>()
    .await
    .wrap_internal_err("failed to fetch available payouts")?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let mut txn_items = Vec::new();
    txn_items.extend(withdrawals);
    txn_items.append(&mut payouts_available);
    txn_items.sort_by_key(|item| item.created());

    Ok(web::Json(txn_items))
}

#[utoipa::path]
#[delete("/{id}")]
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
        Scopes::PAYOUTS_WRITE,
    )
    .await?
    .1;

    let id = info.into_inner().0;
    let payout =
        crate::database::models::payout_item::DBPayout::get(id.into(), &**pool)
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
                                    "payments/payouts-item/{platform_id}/cancel"
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
                                &format!("rewards/{platform_id}/cancel"),
                                None,
                            )
                            .await?;
                    }
                    PayoutMethodType::MuralPay => {
                        let payout_request_id = platform_id
                            .parse::<muralpay::PayoutRequestId>()
                            .wrap_request_err("invalid payout request ID")?;
                        payouts
                            .cancel_mural_payout_request(payout_request_id)
                            .await
                            .wrap_internal_err(
                                "failed to cancel payout request",
                            )?;
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FormCompletionStatus {
    Unknown,
    Unrequested,
    Unsigned,
    TinMismatch,
    Complete,
}

#[utoipa::path]
#[get("/methods")]
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
    pub withdrawn_lifetime: Decimal,
    pub withdrawn_ytd: Decimal,
    pub pending: Decimal,
    pub dates: HashMap<DateTime<Utc>, Decimal>,
}

#[utoipa::path]
#[get("/balance")]
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
        Scopes::PAYOUTS_READ,
    )
    .await?
    .1;

    #[derive(Serialize)]
    struct Response {
        #[serde(flatten)]
        balance: UserBalance,
        requested_form_type: Option<users_compliance::FormType>,
        form_completion_status: Option<FormCompletionStatus>,
    }

    let balance = get_user_balance(user.id.into(), &pool).await?;

    let mut requested_form_type = None;
    let mut form_completion_status = None;

    // Only check compliance status if the compliance check is enabled (by having a value set for it)
    if tax_compliance_payout_threshold().is_some() {
        form_completion_status = Some(
            update_compliance_status(&pool, user.id.into())
                .await?
                .filter(|x| x.model.form_type.is_some())
                .map_or(FormCompletionStatus::Unrequested, |compliance| {
                    requested_form_type = compliance.model.form_type;

                    if compliance.compliance_api_check_failed {
                        FormCompletionStatus::Unknown
                    } else if compliance.model.signed.is_some() {
                        if compliance.model.tin_matched
                            || compliance.model.form_type.is_some_and(|x| {
                                !x.requires_domestic_tin_match()
                            })
                        {
                            FormCompletionStatus::Complete
                        } else {
                            FormCompletionStatus::TinMismatch
                        }
                    } else {
                        FormCompletionStatus::Unsigned
                    }
                }),
        );
    }

    Ok(HttpResponse::Ok().json(Response {
        balance,
        requested_form_type,
        form_completion_status,
    }))
}

async fn get_user_balance(
    user_id: crate::database::models::ids::DBUserId,
    pool: &PgPool,
) -> Result<UserBalance, ApiError> {
    let payouts = sqlx::query!(
        "
        SELECT date_available, SUM(amount) sum
        FROM payouts_values
        WHERE user_id = $1
        GROUP BY date_available
        ORDER BY date_available DESC
        ",
        user_id.0
    )
    .fetch_all(pool)
    .await?;

    let available = payouts
        .iter()
        .filter(|x| x.date_available <= Utc::now())
        .fold(Decimal::ZERO, |acc, x| acc + x.sum.unwrap_or(Decimal::ZERO));
    let pending = payouts
        .iter()
        .filter(|x| x.date_available > Utc::now())
        .fold(Decimal::ZERO, |acc, x| acc + x.sum.unwrap_or(Decimal::ZERO));

    let withdrawn = sqlx::query!(
        "
        SELECT
          SUM(amount) amount,
          SUM(fee) fee,
          SUM(amount) FILTER (WHERE created >= DATE_TRUNC('year', NOW())) amount_this_year
        FROM payouts
        WHERE user_id = $1 AND (status = 'success' OR status = 'in-transit')
        ",
        user_id.0
    )
    .fetch_optional(pool)
    .await?;

    let (withdrawn, fees, withdrawn_this_year) =
        withdrawn.map_or((Decimal::ZERO, Decimal::ZERO, Decimal::ZERO), |x| {
            (
                x.amount.unwrap_or(Decimal::ZERO),
                x.fee.unwrap_or(Decimal::ZERO),
                x.amount_this_year.unwrap_or(Decimal::ZERO),
            )
        });

    Ok(UserBalance {
        available: available.round_dp(16)
            - withdrawn.round_dp(16)
            - fees.round_dp(16),
        withdrawn_lifetime: withdrawn.round_dp(16),
        withdrawn_ytd: withdrawn_this_year.round_dp(16),
        pending,
        dates: payouts
            .iter()
            .map(|x| (x.date_available, x.sum.unwrap_or(Decimal::ZERO)))
            .collect(),
    })
}

struct ComplianceCheck {
    model: users_compliance::UserCompliance,
    compliance_api_check_failed: bool,
}

async fn update_compliance_status(
    pg: &PgPool,
    user_id: crate::database::models::ids::DBUserId,
) -> Result<Option<ComplianceCheck>, ApiError> {
    let maybe_compliance =
        users_compliance::UserCompliance::get_by_user_id(pg, user_id)
            .await
            .wrap_internal_err("failed to fetch user tax compliance")?;

    let Some(mut compliance) = maybe_compliance else {
        return Ok(None);
    };

    if (compliance.signed.is_some() && compliance.tin_matched)
        || Utc::now().signed_duration_since(compliance.last_checked)
            < COMPLIANCE_CHECK_DEBOUNCE
        || compliance.form_type.is_none()
    {
        Ok(Some(ComplianceCheck {
            model: compliance,
            compliance_api_check_failed: false,
        }))
    } else {
        let result = avalara1099::check_form(&compliance.reference_id)
            .await
            .wrap_internal_err("failed to check form using Track1099")?;
        let mut compliance_api_check_failed = false;

        compliance.last_checked = Utc::now();

        match result {
            Ok(None) => {
                // Means the form wasn't signed yet
                compliance.signed = None;
                compliance.e_delivery_consented = false;
                compliance.tin_matched = false;
            }

            Ok(Some(avalara1099::DataWrapper {
                data: avalara1099::Data { attributes, .. },
            })) => {
                compliance.signed =
                    (&attributes.entry_status == "signed").then(Utc::now);
                compliance.e_delivery_consented =
                    attributes.e_delivery_consented_at.is_some();

                if compliance
                    .form_type
                    .is_some_and(|x| x.requires_domestic_tin_match())
                {
                    compliance.tin_matched = attributes
                        .tin_match_status
                        .as_ref()
                        .is_some_and(|x| x == "matched");
                } else {
                    compliance.tin_matched = true;
                }
            }

            Err(json_error) => {
                error!(
                    "Error sending request to Avalara: {}",
                    serde_json::to_string_pretty(&json_error).unwrap()
                );
                compliance_api_check_failed = true;
            }
        }

        compliance.update(pg).await?;

        Ok(Some(ComplianceCheck {
            model: compliance,
            compliance_api_check_failed,
        }))
    }
}

fn tax_compliance_payout_threshold() -> Option<Decimal> {
    dotenvy::var("COMPLIANCE_PAYOUT_THRESHOLD")
        .ok()
        .and_then(|s| s.parse().ok())
}

#[derive(Deserialize)]
pub struct RevenueQuery {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct RevenueResponse {
    pub all_time: Decimal,
    pub all_time_available: Decimal,
    pub data: Vec<RevenueData>,
}

#[derive(Serialize, Deserialize)]
pub struct RevenueData {
    pub time: u64,
    pub revenue: Decimal,
    pub creator_revenue: Decimal,
}

#[utoipa::path]
#[get("/platform_revenue")]
pub async fn platform_revenue(
    query: web::Query<RevenueQuery>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let all_time_payouts = sqlx::query!(
        "
        SELECT SUM(amount) from payouts_values
        ",
    )
    .fetch_optional(&**pool)
    .await?
    .and_then(|x| x.sum)
    .unwrap_or(Decimal::ZERO);

    let all_available = sqlx::query!(
        "
        SELECT SUM(amount) from payouts_values WHERE date_available <= NOW()
        ",
    )
    .fetch_optional(&**pool)
    .await?
    .and_then(|x| x.sum)
    .unwrap_or(Decimal::ZERO);

    let utc = Utc::now();
    let start = query.start.unwrap_or(utc - Duration::days(30));
    let end = query.end.unwrap_or(utc);

    let revenue_data = sqlx::query!(
        "
        SELECT created, SUM(amount) sum
        FROM payouts_values
        WHERE created BETWEEN $1 AND $2
        GROUP BY created
        ORDER BY created DESC
        ",
        start,
        end
    )
    .fetch_all(&**pool)
    .await?
    .into_iter()
    .map(|x| RevenueData {
        time: x.created.timestamp() as u64,
        revenue: x.sum.unwrap_or(Decimal::ZERO) * Decimal::from(25)
            / Decimal::from(75),
        creator_revenue: x.sum.unwrap_or(Decimal::ZERO),
    })
    .collect();

    let res = RevenueResponse {
        all_time: all_time_payouts,
        all_time_available: all_available,
        data: revenue_data,
    };

    Ok(HttpResponse::Ok().json(res))
}
