use crate::models::users::UserId;
use crate::routes::ApiError;
use crate::util::auth::get_user_from_headers;
use actix_web::{post, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use hmac::{Hmac, Mac, NewMac};
use itertools::Itertools;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct CheckoutData {
    pub price_id: String,
}

#[post("/_stripe-init-checkout")]
pub async fn init_checkout(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<CheckoutData>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let client = reqwest::Client::new();

    #[derive(Deserialize)]
    struct Session {
        url: Option<String>,
    }

    let session = client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .header(
            "Authorization",
            format!("Bearer {}", dotenvy::var("STRIPE_TOKEN")?),
        )
        .form(&[
            ("mode", "subscription"),
            ("line_items[0][price]", &*data.price_id),
            ("line_items[0][quantity]", "1"),
            ("success_url", "https://modrinth.com/welcome-to-midas"),
            ("cancel_url", "https://modrinth.com/midas"),
            ("metadata[user_id]", &user.id.to_string()),
        ])
        .send()
        .await
        .map_err(|_| {
            ApiError::Payments(
                "Error while creating checkout session!".to_string(),
            )
        })?
        .json::<Session>()
        .await
        .map_err(|_| {
            ApiError::Payments(
                "Error while deserializing checkout response!".to_string(),
            )
        })?;

    Ok(HttpResponse::Ok().json(json!(
        {
           "url": session.url
        }
    )))
}

#[post("/_stripe-init-portal")]
pub async fn init_customer_portal(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let customer_id = sqlx::query!(
        "
        SELECT u.stripe_customer_id
        FROM users u
        WHERE u.id = $1
        ",
        user.id.0 as i64,
    )
    .fetch_optional(&**pool)
    .await?
    .and_then(|x| x.stripe_customer_id)
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "User is not linked to stripe account!".to_string(),
        )
    })?;

    let client = reqwest::Client::new();

    #[derive(Deserialize)]
    struct Session {
        url: Option<String>,
    }

    let session = client
        .post("https://api.stripe.com/v1/billing_portal/sessions")
        .header(
            "Authorization",
            format!("Bearer {}", dotenvy::var("STRIPE_TOKEN")?),
        )
        .form(&[
            ("customer", &*customer_id),
            ("return_url", "https://modrinth.com/settings/billing"),
        ])
        .send()
        .await
        .map_err(|_| {
            ApiError::Payments(
                "Error while creating billing session!".to_string(),
            )
        })?
        .json::<Session>()
        .await
        .map_err(|_| {
            ApiError::Payments(
                "Error while deserializing billing response!".to_string(),
            )
        })?;

    Ok(HttpResponse::Ok().json(json!(
        {
           "url": session.url
        }
    )))
}

#[post("/_stripe-webook")]
pub async fn handle_stripe_webhook(
    body: String,
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    if let Some(signature_raw) = req
        .headers()
        .get("Stripe-Signature")
        .and_then(|x| x.to_str().ok())
    {
        let mut timestamp = None;
        let mut signature = None;
        for val in signature_raw.split(',') {
            let key_val = val.split('=').collect_vec();

            if key_val.len() == 2 {
                if key_val[0] == "v1" {
                    signature = hex::decode(key_val[1]).ok()
                } else if key_val[0] == "t" {
                    timestamp = key_val[1].parse::<i64>().ok()
                }
            }
        }

        if let Some(timestamp) = timestamp {
            if let Some(signature) = signature {
                type HmacSha256 = Hmac<sha2::Sha256>;

                let mut key = HmacSha256::new_from_slice(dotenvy::var("STRIPE_WEBHOOK_SECRET")?.as_bytes()).map_err(|_| {
                    ApiError::Crypto(
                        "Unable to initialize HMAC instance due to invalid key length!".to_string(),
                    )
                })?;

                key.update(format!("{timestamp}.{body}").as_bytes());

                key.verify(&signature).map_err(|_| {
                    ApiError::Crypto(
                        "Unable to verify webhook signature!".to_string(),
                    )
                })?;

                if timestamp < (Utc::now() - Duration::minutes(5)).timestamp()
                    || timestamp
                        > (Utc::now() + Duration::minutes(5)).timestamp()
                {
                    return Err(ApiError::Crypto(
                        "Webhook signature expired!".to_string(),
                    ));
                }
            } else {
                return Err(ApiError::Crypto("Missing signature!".to_string()));
            }
        } else {
            return Err(ApiError::Crypto("Missing timestamp!".to_string()));
        }
    } else {
        return Err(ApiError::Crypto("Missing signature header!".to_string()));
    }

    #[derive(Deserialize)]
    struct StripeWebhookBody {
        #[serde(rename = "type")]
        type_: String,
        data: StripeWebhookObject,
    }

    #[derive(Deserialize)]
    struct StripeWebhookObject {
        object: Value,
    }

    let webhook: StripeWebhookBody = serde_json::from_str(&body)?;

    #[derive(Deserialize)]
    struct CheckoutSession {
        customer: String,
        metadata: SessionMetadata,
    }

    #[derive(Deserialize)]
    struct SessionMetadata {
        user_id: UserId,
    }

    #[derive(Deserialize)]
    struct Invoice {
        customer: String,
        // paid: bool,
        lines: InvoiceLineItems,
    }

    #[derive(Deserialize)]
    struct InvoiceLineItems {
        pub data: Vec<InvoiceLineItem>,
    }

    #[derive(Deserialize)]
    struct InvoiceLineItem {
        period: Period,
    }

    #[derive(Deserialize)]
    struct Period {
        // start: i64,
        end: i64,
    }

    #[derive(Deserialize)]
    struct Subscription {
        customer: String,
    }

    let mut transaction = pool.begin().await?;

    // TODO: Currently hardcoded to midas-only. When we add more stuff should include price IDs
    match &*webhook.type_ {
        "checkout.session.completed" => {
            let session: CheckoutSession =
                serde_json::from_value(webhook.data.object)?;

            sqlx::query!(
                "
                UPDATE users
                SET stripe_customer_id = $1
                WHERE (id = $2)
                ",
                session.customer,
                session.metadata.user_id.0 as i64,
            )
            .execute(&mut *transaction)
            .await?;
        }
        "invoice.paid" => {
            let invoice: Invoice = serde_json::from_value(webhook.data.object)?;

            if let Some(item) = invoice.lines.data.first() {
                let expires: DateTime<Utc> = DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(item.period.end, 0)
                        .unwrap_or_default(),
                    Utc,
                ) + Duration::days(1);

                sqlx::query!(
                    "
                    UPDATE users
                    SET midas_expires = $1, is_overdue = FALSE
                    WHERE (stripe_customer_id = $2)
                    ",
                    expires,
                    invoice.customer,
                )
                .execute(&mut *transaction)
                .await?;
            }
        }
        "invoice.payment_failed" => {
            let invoice: Invoice = serde_json::from_value(webhook.data.object)?;

            let customer_id = sqlx::query!(
                "
                SELECT u.id
                FROM users u
                WHERE u.stripe_customer_id = $1
                ",
                invoice.customer,
            )
            .fetch_optional(&**pool)
            .await?
            .map(|x| x.id);

            if let Some(user_id) = customer_id {
                sqlx::query!(
                    "
                    UPDATE users
                    SET is_overdue = TRUE
                    WHERE (id = $1)
                    ",
                    user_id,
                )
                .execute(&mut *transaction)
                .await?;
            }
        }
        "customer.subscription.deleted" => {
            let session: Subscription =
                serde_json::from_value(webhook.data.object)?;

            sqlx::query!(
                "
                UPDATE users
                SET stripe_customer_id = NULL, midas_expires = NULL, is_overdue = NULL
                WHERE (stripe_customer_id = $1)
                ",
                session.customer,
            )
                .execute(&mut *transaction)
                .await?;
        }
        _ => {}
    };

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}
