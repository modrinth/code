use crate::auth::validate::get_user_record_from_bearer_token;
use crate::auth::{AuthenticationError, get_user_from_headers};
use crate::database::models::payout_item::DBPayout;
use crate::database::models::{DBPayoutId, DBUser, DBUserId};
use crate::database::models::{generate_payout_id, users_compliance};
use crate::database::redis::RedisPool;
use crate::models::ids::PayoutId;
use crate::models::pats::Scopes;
use crate::models::payouts::{
    MuralPayDetails, PayoutMethodRequest, PayoutMethodType, PayoutStatus,
    TremendousDetails, TremendousForexResponse,
};
use crate::queue::payouts::mural::MuralPayoutRequest;
use crate::queue::payouts::{PayoutFees, PayoutsQueue};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::avalara1099;
use crate::util::error::Context;
use crate::util::gotenberg::GotenbergClient;
use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use chrono::{DateTime, Duration, Utc};
use eyre::eyre;
use hex::ToHex;
use hmac::{Hmac, Mac};
use modrinth_util::decimal::Decimal2dp;
use reqwest::Method;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use sqlx::{PgPool, PgTransaction};
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

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Withdrawal {
    amount: Decimal2dp,
    #[serde(flatten)]
    method: PayoutMethodRequest,
    method_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawalFees {
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

    let fees = payouts_queue
        .calculate_fees(&body.method, &body.method_id, body.amount)
        .await?;

    Ok(web::Json(WithdrawalFees {
        fee: fees.total_fee(),
        exchange_rate: fees.exchange_rate,
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

    let fees = payouts_queue
        .calculate_fees(&body.method, &body.method_id, body.amount)
        .await
        .wrap_internal_err("failed to compute fees")?;

    // fees are a bit complicated here, since we have 2 types:
    // - method fees - this is what Tremendous, Mural, etc. will take from us
    //   without us having a say in it
    // - platform fees - this is what we deliberately keep for ourselves
    // - total fees - method fees + platform fees
    //
    // we first make sure that `amount - total fees` is greater than zero,
    // then we issue a payout request with `amount - platform fees`

    let amount_minus_fee = body.amount - fees.total_fee();
    if amount_minus_fee <= Decimal::ZERO {
        return Err(ApiError::InvalidInput(
            "You need to withdraw more to cover the fee!".to_string(),
        ));
    }

    let sent_to_method = body.amount - fees.platform_fee;
    if sent_to_method <= Decimal::ZERO {
        return Err(ApiError::InvalidInput(
            "You need to withdraw more to cover the fee!".to_string(),
        ));
    }

    let payout_id = generate_payout_id(&mut transaction)
        .await
        .wrap_internal_err("failed to generate payout ID")?;

    let payout_cx = PayoutContext {
        body: &body,
        user: &user,
        payout_id,
        gross_amount: body.amount,
        fees,
        amount_minus_fee,
        total_fee: fees.total_fee(),
        sent_to_method,
        payouts_queue: &payouts_queue,
        db: PgPool::clone(&pool),
        transaction,
    };

    match &body.method {
        PayoutMethodRequest::PayPal | PayoutMethodRequest::Venmo => {
            paypal_payout(payout_cx).await?;
        }
        PayoutMethodRequest::Tremendous { method_details } => {
            tremendous_payout(payout_cx, method_details).await?;
        }
        PayoutMethodRequest::MuralPay { method_details } => {
            mural_pay_payout(payout_cx, method_details, &gotenberg).await?;
        }
    }

    crate::database::models::DBUser::clear_caches(&[(user.id, None)], &redis)
        .await
        .wrap_internal_err("failed to clear user caches")?;

    Ok(())
}

struct PayoutContext<'a> {
    body: &'a Withdrawal,
    user: &'a DBUser,
    payout_id: DBPayoutId,
    gross_amount: Decimal2dp,
    fees: PayoutFees,
    /// Set as the [`DBPayout::amount`] field.
    amount_minus_fee: Decimal2dp,
    /// Set as the [`DBPayout::fee`] field.
    total_fee: Decimal2dp,
    sent_to_method: Decimal2dp,
    payouts_queue: &'a PayoutsQueue,
    db: PgPool,
    transaction: PgTransaction<'a>,
}

fn get_verified_email(user: &DBUser) -> Result<&str, ApiError> {
    let email = user.email.as_ref().wrap_request_err(
        "you must add an email to your account to withdraw",
    )?;
    if !user.email_verified {
        return Err(ApiError::Request(eyre!(
            "you must verify your email to withdraw"
        )));
    }

    Ok(email)
}

async fn tremendous_payout(
    PayoutContext {
        body,
        user,
        payout_id,
        gross_amount: _,
        fees: _,
        amount_minus_fee,
        total_fee,
        sent_to_method,
        payouts_queue,
        db: _,
        mut transaction,
    }: PayoutContext<'_>,
    TremendousDetails {
        delivery_email,
        currency,
    }: &TremendousDetails,
) -> Result<(), ApiError> {
    let user_email = get_verified_email(user)?;

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

    let forex: TremendousForexResponse = payouts_queue
        .make_tremendous_request(Method::GET, "forex", None::<()>)
        .await
        .wrap_internal_err("failed to fetch Tremendous forex data")?;

    let (denomination, currency_code) = if let Some(currency) = currency {
        let currency_code = currency.to_string();
        let exchange_rate =
            forex.forex.get(&currency_code).wrap_internal_err_with(|| {
                eyre!("no Tremendous forex data for {currency}")
            })?;
        (
            sent_to_method.mul_round(*exchange_rate, RoundingStrategy::ToZero),
            Some(currency_code),
        )
    } else {
        (sent_to_method, None)
    };

    let reward_value = if let Some(currency_code) = currency_code {
        json!({
            "denomination": denomination,
            "currency_code": currency_code,
        })
    } else {
        json!({
            "denomination": denomination,
        })
    };

    let res: TremendousResponse = payouts_queue
        .make_tremendous_request(
            Method::POST,
            "orders",
            Some(json! ({
                "payment": {
                    "funding_source_id": "BALANCE",
                },
                "rewards": [{
                    "value": reward_value,
                    "delivery": {
                        "method": "EMAIL"
                    },
                    "recipient": {
                        "name": user.username,
                        "email": delivery_email
                    },
                    "products": [
                        &body.method_id,
                    ],
                    "campaign_id": dotenvy::var("TREMENDOUS_CAMPAIGN_ID")?,
                }]
            })),
        )
        .await?;

    let platform_id = res.order.rewards.first().map(|reward| reward.id.clone());

    DBPayout {
        id: payout_id,
        user_id: user.id,
        created: Utc::now(),
        status: PayoutStatus::InTransit,
        amount: amount_minus_fee.get(),
        fee: Some(total_fee.get()),
        method: Some(PayoutMethodType::Tremendous),
        method_id: Some(body.method_id.clone()),
        method_address: Some(user_email.to_string()),
        platform_id,
    }
    .insert(&mut transaction)
    .await
    .wrap_internal_err("failed to insert payout")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}

async fn mural_pay_payout(
    PayoutContext {
        body: _,
        user,
        payout_id,
        gross_amount,
        fees,
        amount_minus_fee,
        total_fee,
        sent_to_method: _,
        payouts_queue,
        db,
        mut transaction,
    }: PayoutContext<'_>,
    details: &MuralPayDetails,
    gotenberg: &GotenbergClient,
) -> Result<(), ApiError> {
    let user_email = get_verified_email(user)?;

    let method_id = match &details.payout_details {
        MuralPayoutRequest::Blockchain { .. } => {
            "blockchain-usdc-polygon".to_string()
        }
        MuralPayoutRequest::Fiat {
            fiat_and_rail_details,
            ..
        } => fiat_and_rail_details.code().to_string(),
    };

    // Once the Mural payout request has been created successfully,
    // then we *must* commit the payout into the DB,
    // to link the Mural payout request to the `payout` row.
    // Even if we can't execute the payout.
    // For this, we immediately insert and commit the txn.
    // Otherwise if we don't put it into the DB, we've got a ghost Mural
    // payout with no related database entry.
    //
    // However, this doesn't mean that the payout will definitely go through.
    // For this, we need to execute it, and handle errors.

    let payout_request = payouts_queue
        .create_muralpay_payout_request(
            payout_id,
            user.id.into(),
            gross_amount,
            fees,
            details.payout_details.clone(),
            details.recipient_info.clone(),
            gotenberg,
        )
        .await?;

    let payout = DBPayout {
        id: payout_id,
        user_id: user.id,
        created: Utc::now(),
        // after the payout has been successfully executed,
        // we wait for Mural's confirmation that the funds have been delivered
        // done in `SyncPayoutStatuses` background task
        status: PayoutStatus::InTransit,
        amount: amount_minus_fee.get(),
        fee: Some(total_fee.get()),
        method: Some(PayoutMethodType::MuralPay),
        method_id: Some(method_id),
        method_address: Some(user_email.to_string()),
        platform_id: Some(payout_request.id.to_string()),
    };
    payout
        .insert(&mut transaction)
        .await
        .wrap_internal_err("failed to insert payout")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit payout insert transaction")?;

    // try to immediately execute the payout request...
    // use a poor man's try/catch block using this `async move {}`
    // to catch any errors within this block
    let result = async move {
        payouts_queue
            .execute_mural_payout_request(payout_request.id)
            .await
            .wrap_internal_err("failed to execute payout request")?;
        eyre::Ok(())
    }
    .await;

    // and if it fails, make sure to immediately cancel it -
    // we don't want floating payout requests
    if let Err(err) = result {
        if let Err(err) = sqlx::query!(
            "
            UPDATE payouts
            SET status = $1
            WHERE id = $2
            ",
            PayoutStatus::Failed.as_str(),
            payout.id as _,
        )
        .execute(&db)
        .await
        {
            error!(
                "Created a Mural payout request, but failed to execute it, \
                and failed to mark the payout as failed: {err:#?}"
            );
        }

        payouts_queue
            .cancel_mural_payout_request(payout_request.id)
            .await
            .wrap_internal_err_with(|| {
                eyre!("failed to cancel unexecuted payout request\noriginal error: {err:#?}")
            })?;

        return Err(ApiError::Internal(err));
    }

    Ok(())
}

async fn paypal_payout(
    PayoutContext {
        body,
        user,
        payout_id,
        gross_amount: _,
        fees: _,
        amount_minus_fee,
        total_fee,
        sent_to_method,
        payouts_queue,
        db: _,
        mut transaction,
    }: PayoutContext<'_>,
) -> Result<(), ApiError> {
    let (wallet, wallet_type, address, display_address) =
        if matches!(body.method, PayoutMethodRequest::Venmo) {
            if let Some(venmo) = &user.venmo_handle {
                ("Venmo", "user_handle", venmo.clone(), venmo)
            } else {
                return Err(ApiError::InvalidInput(
                    "Venmo address has not been set for account!".to_string(),
                ));
            }
        } else if let Some(paypal_id) = &user.paypal_id {
            if let Some(paypal_country) = &user.paypal_country {
                if paypal_country == "US" && &*body.method_id != "paypal_us" {
                    return Err(ApiError::InvalidInput(
                        "Please use the US PayPal transfer option!".to_string(),
                    ));
                } else if paypal_country != "US"
                    && &*body.method_id == "paypal_us"
                {
                    return Err(ApiError::InvalidInput(
                        "Please use the International PayPal transfer option!"
                            .to_string(),
                    ));
                }

                (
                    "PayPal",
                    "paypal_id",
                    paypal_id.clone(),
                    user.paypal_email.as_ref().unwrap_or(paypal_id),
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

    let res: PayoutsResponse = payouts_queue.make_paypal_request(
        Method::POST,
        "payments/payouts",
        Some(
            json!({
                "sender_batch_header": {
                    "sender_batch_id": format!("{}-payouts", Utc::now().to_rfc3339()),
                    "email_subject": "You have received a payment from Modrinth!",
                    "email_message": "Thank you for creating projects on Modrinth. Please claim this payment within 30 days.",
                },
                "items": [{
                    "amount": {
                        "currency": "USD",
                        "value": sent_to_method.to_string()
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

    let link = res
        .links
        .first()
        .wrap_request_err("no PayPal links available")?;

    #[derive(Deserialize)]
    struct PayoutItem {
        pub payout_item_id: String,
    }

    #[derive(Deserialize)]
    struct PayoutData {
        pub items: Vec<PayoutItem>,
    }

    let res = payouts_queue
        .make_paypal_request::<(), PayoutData>(
            Method::GET,
            &link.href,
            None,
            None,
            Some(true),
        )
        .await
        .wrap_internal_err("failed to make PayPal request")?;
    let data = res
        .items
        .first()
        .wrap_internal_err("no payout items returned from PayPal request")?;

    let platform_id = Some(data.payout_item_id.clone());

    DBPayout {
        id: payout_id,
        user_id: user.id,
        created: Utc::now(),
        status: PayoutStatus::InTransit,
        amount: amount_minus_fee.get(),
        fee: Some(total_fee.get()),
        method: Some(body.method.method_type()),
        method_id: Some(body.method_id.clone()),
        method_address: Some(display_address.clone()),
        platform_id,
    }
    .insert(&mut transaction)
    .await
    .wrap_internal_err("failed to insert payout")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

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
