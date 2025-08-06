use actix_web::{HttpResponse, post, web};
use ariadne::ids::UserId;
use ariadne::ids::base62_impl::to_base62;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::database::models::charge_item::DBCharge;
use crate::database::models::product_item;
use crate::database::models::user_subscription_item::DBUserSubscription;
use crate::database::models::users_redeemals::{
    Offer, RedeemalLookupFields, Status, UserRedeemal,
};
use crate::database::models::{
    generate_charge_id, generate_user_subscription_id,
};
use crate::models::v3::billing::{
    ChargeStatus, ChargeType, PaymentPlatform, Price, PriceDuration,
    ProductMetadata, SubscriptionMetadata, SubscriptionStatus,
};
use crate::routes::ApiError;
use crate::util::guards::medal_key_guard;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("medal").service(verify).service(redeem));
}

#[derive(Deserialize)]
struct MedalQuery {
    username: String,
}

#[post("verify", guard = "medal_key_guard")]
pub async fn verify(
    pool: web::Data<PgPool>,
    web::Query(MedalQuery { username }): web::Query<MedalQuery>,
) -> Result<HttpResponse, ApiError> {
    let maybe_fields =
        RedeemalLookupFields::redeemal_status_by_username_and_offer(
            &**pool,
            &username,
            Offer::Medal,
        )
        .await?;

    #[derive(Serialize)]
    struct VerifyResponse {
        user_id: UserId,
        redeemed: bool,
    }

    match maybe_fields {
        None => Err(ApiError::NotFound),
        Some(fields) => Ok(HttpResponse::Ok().json(VerifyResponse {
            user_id: fields.user_id.into(),
            redeemed: fields.redeemal_status.is_some(),
        })),
    }
}

#[post("redeem", guard = "medal_key_guard")]
pub async fn redeem(
    pool: web::Data<PgPool>,
    web::Query(MedalQuery { username }): web::Query<MedalQuery>,
) -> Result<HttpResponse, ApiError> {
    // Check the offer hasn't been redeemed yet, then insert into the table.

    let maybe_fields =
        RedeemalLookupFields::redeemal_status_by_username_and_offer(
            &**pool,
            &username,
            Offer::Medal,
        )
        .await?;

    let user_id = match maybe_fields {
        None => return Err(ApiError::NotFound),
        Some(fields) => {
            if fields.redeemal_status.is_some() {
                return Err(ApiError::Conflict(
                    "User already redeemed this offer".to_string(),
                ));
            }

            fields.user_id
        }
    };

    let client = reqwest::Client::new();

    // Find the Medal product price
    let mut medal_products =
        product_item::QueryProductWithPrices::list_by_product_type(
            &**pool, "medal",
        )
        .await?;

    let Some(product_item::QueryProductWithPrices {
        id: _product_id,
        metadata,
        mut prices,
        unitary: _,
    }) = medal_products.pop()
    else {
        return Ok(HttpResponse::NotImplemented()
            .body("Missing Medal subscription product"));
    };

    let ProductMetadata::Medal {
        cpu,
        ram,
        swap,
        storage,
        region,
    } = metadata
    else {
        return Ok(HttpResponse::NotImplemented()
            .body("Missing or incorrect metadata for Medal subscription"));
    };

    let Some(medal_price) = prices.pop() else {
        return Ok(HttpResponse::NotImplemented()
            .body("Missing price for Medal subscription"));
    };

    let (price_duration, price_amount) = match medal_price.prices {
        Price::OneTime { price: _ } => {
            return Ok(HttpResponse::NotImplemented()
                .body("Unexpected metadata for Medal subscription price"));
        }

        Price::Recurring { intervals } => {
            let Some((price_duration, price_amount)) =
                intervals.into_iter().next()
            else {
                return Ok(HttpResponse::NotImplemented()
                    .body("Missing price interval for Medal subscription"));
            };

            (price_duration, price_amount)
        }
    };

    let price_id = medal_price.id;

    #[derive(Deserialize)]
    struct PyroServerResponse {
        uuid: uuid::Uuid,
    }

    // TODO: archon-client module
    let pyro_response = client
        .post(format!(
            "{}/modrinth/v0/servers/create",
            dotenvy::var("ARCHON_URL")?,
        ))
        .header("X-Master-Key", dotenvy::var("PYRO_API_KEY")?)
        .json(&serde_json::json!({
            "user_id": to_base62(user_id.0 as u64),
            "name": format!("{}'s Medal server", username),
            "specs": {
                "memory_mb": ram,
                "cpu": cpu,
                "swap_mb": swap,
                "storage_mb": storage,
            },
            "region": region,
            "source": {}, // Don't install anything by default (field is ignored on Archon anyways)
            "payment_interval": 1, // Doesn't matter, not used on Archon anymore anyways
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<PyroServerResponse>()
        .await?;

    let mut txn = pool.begin().await?;

    // Build a subscription using this price ID.
    let subscription = DBUserSubscription {
        id: generate_user_subscription_id(&mut txn).await?,
        user_id,
        price_id,
        interval: PriceDuration::FiveDays,
        created: Utc::now(),
        status: SubscriptionStatus::Provisioned,
        metadata: Some(SubscriptionMetadata::Medal {
            id: pyro_response.uuid.to_string(),
        }),
    };

    subscription.upsert(&mut txn).await?;

    // Insert an expiring charge, `index_subscriptions` will unprovision the
    // subscription when expired.
    DBCharge {
        id: generate_charge_id(&mut txn).await?,
        user_id,
        price_id,
        amount: price_amount.into(),
        currency_code: medal_price.currency_code,
        status: ChargeStatus::Expiring,
        due: Utc::now() + price_duration.duration(),
        last_attempt: None,
        type_: ChargeType::Subscription,
        subscription_id: Some(subscription.id),
        subscription_interval: Some(subscription.interval),
        payment_platform: PaymentPlatform::None,
        payment_platform_id: None,
        parent_charge_id: None,
        net: None,
    }
    .upsert(&mut txn)
    .await?;

    // Link user to offer redeemal.
    let mut redeemal = UserRedeemal {
        id: 0,
        user_id,
        offer: Offer::Medal,
        redeemed: Utc::now(),
        status: Status::Redeemed,
    };

    redeemal.insert(&mut *txn).await?;

    txn.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
