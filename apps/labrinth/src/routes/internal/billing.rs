use crate::auth::{get_user_from_headers, send_email};
use crate::database::models::charge_item::ChargeItem;
use crate::database::models::{
    generate_charge_id, generate_user_subscription_id, product_item,
    user_subscription_item,
};
use crate::database::redis::RedisPool;
use crate::models::billing::{
    Charge, ChargeStatus, ChargeType, Price, PriceDuration, Product,
    ProductMetadata, ProductPrice, SubscriptionMetadata, SubscriptionStatus,
    UserSubscription,
};
use crate::models::ids::base62_impl::{parse_base62, to_base62};
use crate::models::pats::Scopes;
use crate::models::users::Badges;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use chrono::Utc;
use log::{info, warn};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;
use serde_with::serde_derive::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use stripe::{
    CreateCustomer, CreatePaymentIntent, CreateSetupIntent,
    CreateSetupIntentAutomaticPaymentMethods,
    CreateSetupIntentAutomaticPaymentMethodsAllowRedirects, Currency,
    CustomerId, CustomerInvoiceSettings, CustomerPaymentMethodRetrieval,
    EventObject, EventType, PaymentIntentOffSession,
    PaymentIntentSetupFutureUsage, PaymentMethodId, SetupIntent,
    UpdateCustomer, Webhook,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("billing")
            .service(products)
            .service(subscriptions)
            .service(user_customer)
            .service(edit_subscription)
            .service(payment_methods)
            .service(add_payment_method_flow)
            .service(edit_payment_method)
            .service(remove_payment_method)
            .service(charges)
            .service(initiate_payment)
            .service(stripe_webhook),
    );
}

#[get("products")]
pub async fn products(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let products = product_item::QueryProduct::list(&**pool, &redis).await?;

    let products = products
        .into_iter()
        .map(|x| Product {
            id: x.id.into(),
            metadata: x.metadata,
            prices: x
                .prices
                .into_iter()
                .map(|x| ProductPrice {
                    id: x.id.into(),
                    product_id: x.product_id.into(),
                    currency_code: x.currency_code,
                    prices: x.prices,
                })
                .collect(),
            unitary: x.unitary,
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(products))
}

#[get("subscriptions")]
pub async fn subscriptions(
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
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let subscriptions =
        user_subscription_item::UserSubscriptionItem::get_all_user(
            user.id.into(),
            &**pool,
        )
        .await?
        .into_iter()
        .map(UserSubscription::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(subscriptions))
}

#[derive(Deserialize)]
pub struct SubscriptionEdit {
    pub interval: Option<PriceDuration>,
    pub payment_method: Option<String>,
    pub cancelled: Option<bool>,
    pub product: Option<crate::models::ids::ProductId>,
}

#[patch("subscription/{id}")]
pub async fn edit_subscription(
    req: HttpRequest,
    info: web::Path<(crate::models::ids::UserSubscriptionId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    edit_subscription: web::Json<SubscriptionEdit>,
    stripe_client: web::Data<stripe::Client>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let (id,) = info.into_inner();

    if let Some(subscription) =
        user_subscription_item::UserSubscriptionItem::get(id.into(), &**pool)
            .await?
    {
        if subscription.user_id != user.id.into() && !user.role.is_admin() {
            return Err(ApiError::NotFound);
        }

        let mut transaction = pool.begin().await?;

        let mut open_charge =
            crate::database::models::charge_item::ChargeItem::get_open_subscription(
                subscription.id,
                &mut *transaction,
            )
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "Could not find open charge for this subscription".to_string(),
                )
            })?;

        let current_price = product_item::ProductPriceItem::get(
            subscription.price_id,
            &mut *transaction,
        )
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "Could not find current product price".to_string(),
            )
        })?;

        if let Some(cancelled) = &edit_subscription.cancelled {
            if open_charge.status != ChargeStatus::Open
                && open_charge.status != ChargeStatus::Cancelled
            {
                return Err(ApiError::InvalidInput(
                    "You may not change the status of this subscription!"
                        .to_string(),
                ));
            }

            if *cancelled {
                open_charge.status = ChargeStatus::Cancelled;
            } else {
                open_charge.status = ChargeStatus::Open;
            }
        }

        if let Some(interval) = &edit_subscription.interval {
            if let Price::Recurring { intervals } = &current_price.prices {
                if let Some(price) = intervals.get(interval) {
                    open_charge.subscription_interval = Some(*interval);
                    open_charge.amount = *price as i64;
                } else {
                    return Err(ApiError::InvalidInput(
                        "Interval is not valid for this subscription!"
                            .to_string(),
                    ));
                }
            }
        }

        let intent = if let Some(product_id) = &edit_subscription.product {
            let product_price =
                product_item::ProductPriceItem::get_all_product_prices(
                    (*product_id).into(),
                    &mut *transaction,
                )
                .await?
                .into_iter()
                .find(|x| x.currency_code == current_price.currency_code)
                .ok_or_else(|| {
                    ApiError::InvalidInput(
                        "Could not find a valid price for your currency code!"
                            .to_string(),
                    )
                })?;

            if product_price.id == current_price.id {
                return Err(ApiError::InvalidInput(
                    "You may not change the price of this subscription!"
                        .to_string(),
                ));
            }

            let interval = open_charge.due - Utc::now();
            let duration = PriceDuration::iterator()
                .min_by_key(|x| {
                    (x.duration().num_seconds() - interval.num_seconds()).abs()
                })
                .unwrap_or(PriceDuration::Monthly);

            let current_amount = match &current_price.prices {
                Price::OneTime { price } => *price,
                Price::Recurring { intervals } => *intervals.get(&duration).ok_or_else(|| {
                    ApiError::InvalidInput(
                        "Could not find a valid price for the user's duration".to_string(),
                    )
                })?,
            };

            let amount = match &product_price.prices {
                Price::OneTime { price } => *price,
                Price::Recurring { intervals } => *intervals.get(&duration).ok_or_else(|| {
                    ApiError::InvalidInput(
                        "Could not find a valid price for the user's duration".to_string(),
                    )
                })?,
            };

            let complete = Decimal::from(interval.num_seconds())
                / Decimal::from(duration.duration().num_seconds());
            let proration = (Decimal::from(amount - current_amount) * complete)
                .floor()
                .to_i32()
                .ok_or_else(|| {
                    ApiError::InvalidInput(
                        "Could not convert proration to i32".to_string(),
                    )
                })?;

            // TODO: Add downgrading plans
            if proration <= 0 {
                return Err(ApiError::InvalidInput(
                    "You may not downgrade plans!".to_string(),
                ));
            }

            let charge_id = generate_charge_id(&mut transaction).await?;
            let charge = ChargeItem {
                id: charge_id,
                user_id: user.id.into(),
                price_id: product_price.id,
                amount: proration as i64,
                currency_code: current_price.currency_code.clone(),
                status: ChargeStatus::Processing,
                due: Utc::now(),
                last_attempt: None,
                type_: ChargeType::Proration,
                subscription_id: Some(subscription.id),
                subscription_interval: Some(duration),
            };

            let customer_id = get_or_create_customer(
                user.id,
                user.stripe_customer_id.as_deref(),
                user.email.as_deref(),
                &stripe_client,
                &pool,
                &redis,
            )
            .await?;

            let currency =
                Currency::from_str(&current_price.currency_code.to_lowercase())
                    .map_err(|_| {
                        ApiError::InvalidInput(
                            "Invalid currency code".to_string(),
                        )
                    })?;

            let mut intent =
                CreatePaymentIntent::new(proration as i64, currency);

            let mut metadata = HashMap::new();
            metadata
                .insert("modrinth_user_id".to_string(), to_base62(user.id.0));

            intent.customer = Some(customer_id);
            intent.metadata = Some(metadata);
            intent.receipt_email = user.email.as_deref();
            intent.setup_future_usage =
                Some(PaymentIntentSetupFutureUsage::OffSession);

            if let Some(payment_method) = &edit_subscription.payment_method {
                let payment_method_id =
                    if let Ok(id) = PaymentMethodId::from_str(payment_method) {
                        id
                    } else {
                        return Err(ApiError::InvalidInput(
                            "Invalid payment method id".to_string(),
                        ));
                    };
                intent.payment_method = Some(payment_method_id);
            }

            charge.upsert(&mut transaction).await?;

            Some((
                proration,
                0,
                stripe::PaymentIntent::create(&stripe_client, intent).await?,
            ))
        } else {
            None
        };

        open_charge.upsert(&mut transaction).await?;

        transaction.commit().await?;

        if let Some((amount, tax, payment_intent)) = intent {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "payment_intent_id": payment_intent.id,
                "client_secret": payment_intent.client_secret,
                "tax": tax,
                "total": amount
            })))
        } else {
            Ok(HttpResponse::NoContent().body(""))
        }
    } else {
        Err(ApiError::NotFound)
    }
}

#[get("customer")]
pub async fn user_customer(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    stripe_client: web::Data<stripe::Client>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let customer_id = get_or_create_customer(
        user.id,
        user.stripe_customer_id.as_deref(),
        user.email.as_deref(),
        &stripe_client,
        &pool,
        &redis,
    )
    .await?;
    let customer =
        stripe::Customer::retrieve(&stripe_client, &customer_id, &[]).await?;

    Ok(HttpResponse::Ok().json(customer))
}

#[get("payments")]
pub async fn charges(
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
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let charges =
        crate::database::models::charge_item::ChargeItem::get_from_user(
            user.id.into(),
            &**pool,
        )
        .await?;

    Ok(HttpResponse::Ok().json(
        charges
            .into_iter()
            .map(|x| Charge {
                id: x.id.into(),
                user_id: x.user_id.into(),
                price_id: x.price_id.into(),
                amount: x.amount,
                currency_code: x.currency_code,
                status: x.status,
                due: x.due,
                last_attempt: x.last_attempt,
                type_: x.type_,
                subscription_id: x.subscription_id.map(|x| x.into()),
                subscription_interval: x.subscription_interval,
            })
            .collect::<Vec<_>>(),
    ))
}

#[post("payment_method")]
pub async fn add_payment_method_flow(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    stripe_client: web::Data<stripe::Client>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let customer = get_or_create_customer(
        user.id,
        user.stripe_customer_id.as_deref(),
        user.email.as_deref(),
        &stripe_client,
        &pool,
        &redis,
    )
    .await?;

    let intent = SetupIntent::create(
        &stripe_client,
        CreateSetupIntent {
            customer: Some(customer),
            automatic_payment_methods: Some(CreateSetupIntentAutomaticPaymentMethods {
                allow_redirects: Some(
                    CreateSetupIntentAutomaticPaymentMethodsAllowRedirects::Never,
                ),
                enabled: true,
            }),
            ..Default::default()
        },
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "client_secret": intent.client_secret
    })))
}

#[derive(Deserialize)]
pub struct EditPaymentMethod {
    pub primary: bool,
}

#[patch("payment_method/{id}")]
pub async fn edit_payment_method(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    stripe_client: web::Data<stripe::Client>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let (id,) = info.into_inner();

    let payment_method_id = if let Ok(id) = PaymentMethodId::from_str(&id) {
        id
    } else {
        return Err(ApiError::NotFound);
    };

    let customer = get_or_create_customer(
        user.id,
        user.stripe_customer_id.as_deref(),
        user.email.as_deref(),
        &stripe_client,
        &pool,
        &redis,
    )
    .await?;

    let payment_method = stripe::PaymentMethod::retrieve(
        &stripe_client,
        &payment_method_id,
        &[],
    )
    .await?;

    if payment_method
        .customer
        .map(|x| x.id() == customer)
        .unwrap_or(false)
        || user.role.is_admin()
    {
        stripe::Customer::update(
            &stripe_client,
            &customer,
            UpdateCustomer {
                invoice_settings: Some(CustomerInvoiceSettings {
                    default_payment_method: Some(payment_method.id.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::NotFound)
    }
}

#[delete("payment_method/{id}")]
pub async fn remove_payment_method(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    stripe_client: web::Data<stripe::Client>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let (id,) = info.into_inner();

    let payment_method_id = if let Ok(id) = PaymentMethodId::from_str(&id) {
        id
    } else {
        return Err(ApiError::NotFound);
    };

    let customer = get_or_create_customer(
        user.id,
        user.stripe_customer_id.as_deref(),
        user.email.as_deref(),
        &stripe_client,
        &pool,
        &redis,
    )
    .await?;

    let payment_method = stripe::PaymentMethod::retrieve(
        &stripe_client,
        &payment_method_id,
        &[],
    )
    .await?;

    let user_subscriptions =
        user_subscription_item::UserSubscriptionItem::get_all_user(
            user.id.into(),
            &**pool,
        )
        .await?;

    if user_subscriptions
        .iter()
        .any(|x| x.status != SubscriptionStatus::Unprovisioned)
    {
        let customer =
            stripe::Customer::retrieve(&stripe_client, &customer, &[]).await?;

        if customer
            .invoice_settings
            .and_then(|x| {
                x.default_payment_method
                    .map(|x| x.id() == payment_method_id)
            })
            .unwrap_or(false)
        {
            return Err(ApiError::InvalidInput(
                "You may not remove the default payment method if you have active subscriptions!"
                    .to_string(),
            ));
        }
    }

    if payment_method
        .customer
        .map(|x| x.id() == customer)
        .unwrap_or(false)
        || user.role.is_admin()
    {
        stripe::PaymentMethod::detach(&stripe_client, &payment_method_id)
            .await?;

        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::NotFound)
    }
}

#[get("payment_methods")]
pub async fn payment_methods(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    stripe_client: web::Data<stripe::Client>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    if let Some(customer_id) = user
        .stripe_customer_id
        .as_ref()
        .and_then(|x| stripe::CustomerId::from_str(x).ok())
    {
        let methods = stripe::Customer::retrieve_payment_methods(
            &stripe_client,
            &customer_id,
            CustomerPaymentMethodRetrieval {
                limit: Some(100),
                ..Default::default()
            },
        )
        .await?;

        Ok(HttpResponse::Ok().json(methods.data))
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaymentRequestType {
    PaymentMethod { id: String },
    ConfirmationToken { token: String },
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChargeRequestType {
    Existing {
        id: crate::models::ids::ChargeId,
    },
    New {
        product_id: crate::models::ids::ProductId,
        interval: Option<PriceDuration>,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaymentRequestMetadata {
    Pyro {
        server_name: Option<String>,
        source: serde_json::Value,
    },
}

#[derive(Deserialize)]
pub struct PaymentRequest {
    #[serde(flatten)]
    pub type_: PaymentRequestType,
    pub charge: ChargeRequestType,
    pub existing_payment_intent: Option<stripe::PaymentIntentId>,
    pub metadata: Option<PaymentRequestMetadata>,
}

fn infer_currency_code(country: &str) -> String {
    match country {
        "US" => "USD",
        "GB" => "GBP",
        "EU" => "EUR",
        "AT" => "EUR",
        "BE" => "EUR",
        "CY" => "EUR",
        "EE" => "EUR",
        "FI" => "EUR",
        "FR" => "EUR",
        "DE" => "EUR",
        "GR" => "EUR",
        "IE" => "EUR",
        "IT" => "EUR",
        "LV" => "EUR",
        "LT" => "EUR",
        "LU" => "EUR",
        "MT" => "EUR",
        "NL" => "EUR",
        "PT" => "EUR",
        "SK" => "EUR",
        "SI" => "EUR",
        "RU" => "RUB",
        "BR" => "BRL",
        "JP" => "JPY",
        "ID" => "IDR",
        "MY" => "MYR",
        "PH" => "PHP",
        "TH" => "THB",
        "VN" => "VND",
        "KR" => "KRW",
        "TR" => "TRY",
        "UA" => "UAH",
        "MX" => "MXN",
        "CA" => "CAD",
        "NZ" => "NZD",
        "NO" => "NOK",
        "PL" => "PLN",
        "CH" => "CHF",
        "LI" => "CHF",
        "IN" => "INR",
        "CL" => "CLP",
        "PE" => "PEN",
        "CO" => "COP",
        "ZA" => "ZAR",
        "HK" => "HKD",
        "AR" => "ARS",
        "KZ" => "KZT",
        "UY" => "UYU",
        "CN" => "CNY",
        "AU" => "AUD",
        "TW" => "TWD",
        "SA" => "SAR",
        "QA" => "QAR",
        _ => "USD",
    }
    .to_string()
}

#[post("payment")]
pub async fn initiate_payment(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    stripe_client: web::Data<stripe::Client>,
    payment_request: web::Json<PaymentRequest>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let (user_country, payment_method) = match &payment_request.type_ {
        PaymentRequestType::PaymentMethod { id } => {
            let payment_method_id = stripe::PaymentMethodId::from_str(id)
                .map_err(|_| {
                    ApiError::InvalidInput(
                        "Invalid payment method id".to_string(),
                    )
                })?;

            let payment_method = stripe::PaymentMethod::retrieve(
                &stripe_client,
                &payment_method_id,
                &[],
            )
            .await?;

            let country = payment_method
                .billing_details
                .address
                .as_ref()
                .and_then(|x| x.country.clone());

            (country, payment_method)
        }
        PaymentRequestType::ConfirmationToken { token } => {
            #[derive(Deserialize)]
            struct ConfirmationToken {
                payment_method_preview: Option<stripe::PaymentMethod>,
            }

            let mut confirmation: serde_json::Value = stripe_client
                .get(&format!("confirmation_tokens/{token}"))
                .await?;

            // We patch the JSONs to support the PaymentMethod struct
            let p: json_patch::Patch = serde_json::from_value(serde_json::json!([
                { "op": "add", "path": "/payment_method_preview/id", "value": "pm_1PirTdJygY5LJFfKmPIaM1N1" },
                { "op": "add", "path": "/payment_method_preview/created", "value": 1723183475 },
                { "op": "add", "path": "/payment_method_preview/livemode", "value": false }
            ])).unwrap();
            json_patch::patch(&mut confirmation, &p).unwrap();

            let confirmation: ConfirmationToken =
                serde_json::from_value(confirmation)?;

            let payment_method =
                confirmation.payment_method_preview.ok_or_else(|| {
                    ApiError::InvalidInput(
                        "Confirmation token is missing payment method!"
                            .to_string(),
                    )
                })?;

            let country = payment_method
                .billing_details
                .address
                .as_ref()
                .and_then(|x| x.country.clone());

            (country, payment_method)
        }
    };

    let country = user_country.as_deref().unwrap_or("US");
    let recommended_currency_code = infer_currency_code(country);

    let (price, currency_code, interval, price_id, charge_id) =
        match payment_request.charge {
            ChargeRequestType::Existing { id } => {
                let charge =
                    crate::database::models::charge_item::ChargeItem::get(
                        id.into(),
                        &**pool,
                    )
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInput(
                            "Specified charge could not be found!".to_string(),
                        )
                    })?;

                (
                    charge.amount,
                    charge.currency_code,
                    charge.subscription_interval,
                    charge.price_id,
                    Some(id),
                )
            }
            ChargeRequestType::New {
                product_id,
                interval,
            } => {
                let product =
                    product_item::ProductItem::get(product_id.into(), &**pool)
                        .await?
                        .ok_or_else(|| {
                            ApiError::InvalidInput(
                                "Specified product could not be found!"
                                    .to_string(),
                            )
                        })?;

                let mut product_prices =
                    product_item::ProductPriceItem::get_all_product_prices(
                        product.id, &**pool,
                    )
                    .await?;

                let price_item = if let Some(pos) = product_prices
                    .iter()
                    .position(|x| x.currency_code == recommended_currency_code)
                {
                    product_prices.remove(pos)
                } else if let Some(pos) =
                    product_prices.iter().position(|x| x.currency_code == "USD")
                {
                    product_prices.remove(pos)
                } else {
                    return Err(ApiError::InvalidInput(
                        "Could not find a valid price for the user's country"
                            .to_string(),
                    ));
                };

                let price = match price_item.prices {
                    Price::OneTime { price } => price,
                    Price::Recurring { ref intervals } => {
                        let interval = interval.ok_or_else(|| {
                        ApiError::InvalidInput(
                            "Could not find a valid interval for the user's country".to_string(),
                        )
                    })?;

                        *intervals.get(&interval).ok_or_else(|| {
                        ApiError::InvalidInput(
                            "Could not find a valid price for the user's country".to_string(),
                        )
                    })?
                    }
                };

                if let Price::Recurring { .. } = price_item.prices {
                    if product.unitary {
                        let user_subscriptions =
                        user_subscription_item::UserSubscriptionItem::get_all_user(
                            user.id.into(),
                            &**pool,
                        )
                        .await?;

                        let user_products =
                            product_item::ProductPriceItem::get_many(
                                &user_subscriptions
                                    .iter()
                                    .filter(|x| {
                                        x.status
                                            == SubscriptionStatus::Provisioned
                                    })
                                    .map(|x| x.price_id)
                                    .collect::<Vec<_>>(),
                                &**pool,
                            )
                            .await?;

                        if user_products
                            .into_iter()
                            .any(|x| x.product_id == product.id)
                        {
                            return Err(ApiError::InvalidInput(
                                "You are already subscribed to this product!"
                                    .to_string(),
                            ));
                        }
                    }
                }

                (
                    price as i64,
                    price_item.currency_code,
                    interval,
                    price_item.id,
                    None,
                )
            }
        };

    let customer = get_or_create_customer(
        user.id,
        user.stripe_customer_id.as_deref(),
        user.email.as_deref(),
        &stripe_client,
        &pool,
        &redis,
    )
    .await?;
    let stripe_currency = Currency::from_str(&currency_code.to_lowercase())
        .map_err(|_| {
            ApiError::InvalidInput("Invalid currency code".to_string())
        })?;

    if let Some(payment_intent_id) = &payment_request.existing_payment_intent {
        let mut update_payment_intent = stripe::UpdatePaymentIntent {
            amount: Some(price),
            currency: Some(stripe_currency),
            customer: Some(customer),
            ..Default::default()
        };

        if let PaymentRequestType::PaymentMethod { .. } = payment_request.type_
        {
            update_payment_intent.payment_method =
                Some(payment_method.id.clone());
        }

        stripe::PaymentIntent::update(
            &stripe_client,
            payment_intent_id,
            update_payment_intent,
        )
        .await?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "price_id": to_base62(price_id.0 as u64),
            "tax": 0,
            "total": price,
            "payment_method": payment_method,
        })))
    } else {
        let mut intent = CreatePaymentIntent::new(price, stripe_currency);

        let mut metadata = HashMap::new();
        metadata.insert("modrinth_user_id".to_string(), to_base62(user.id.0));

        if let Some(payment_metadata) = &payment_request.metadata {
            metadata.insert(
                "modrinth_payment_metadata".to_string(),
                serde_json::to_string(&payment_metadata)?,
            );
        }

        if let Some(charge_id) = charge_id {
            metadata.insert(
                "modrinth_charge_id".to_string(),
                to_base62(charge_id.0),
            );
        } else {
            let mut transaction = pool.begin().await?;
            let charge_id = generate_charge_id(&mut transaction).await?;
            let subscription_id =
                generate_user_subscription_id(&mut transaction).await?;

            metadata.insert(
                "modrinth_charge_id".to_string(),
                to_base62(charge_id.0 as u64),
            );
            metadata.insert(
                "modrinth_subscription_id".to_string(),
                to_base62(subscription_id.0 as u64),
            );

            metadata.insert(
                "modrinth_price_id".to_string(),
                to_base62(price_id.0 as u64),
            );

            if let Some(interval) = interval {
                metadata.insert(
                    "modrinth_subscription_interval".to_string(),
                    interval.as_str().to_string(),
                );
            }
        }

        intent.customer = Some(customer);
        intent.metadata = Some(metadata);
        intent.receipt_email = user.email.as_deref();
        intent.setup_future_usage =
            Some(PaymentIntentSetupFutureUsage::OffSession);

        if let PaymentRequestType::PaymentMethod { .. } = payment_request.type_
        {
            intent.payment_method = Some(payment_method.id.clone());
        }

        let payment_intent =
            stripe::PaymentIntent::create(&stripe_client, intent).await?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "payment_intent_id": payment_intent.id,
            "client_secret": payment_intent.client_secret,
            "price_id": to_base62(price_id.0 as u64),
            "tax": 0,
            "total": price,
            "payment_method": payment_method,
        })))
    }
}

#[post("_stripe")]
pub async fn stripe_webhook(
    req: HttpRequest,
    payload: String,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    stripe_client: web::Data<stripe::Client>,
) -> Result<HttpResponse, ApiError> {
    let stripe_signature = req
        .headers()
        .get("Stripe-Signature")
        .and_then(|x| x.to_str().ok())
        .unwrap_or_default();

    if let Ok(event) = Webhook::construct_event(
        &payload,
        stripe_signature,
        &dotenvy::var("STRIPE_WEBHOOK_SECRET")?,
    ) {
        struct PaymentIntentMetadata {
            pub user_item: crate::database::models::user_item::User,
            pub product_price_item: product_item::ProductPriceItem,
            pub product_item: product_item::ProductItem,
            pub charge_item: crate::database::models::charge_item::ChargeItem,
            pub user_subscription_item:
                Option<user_subscription_item::UserSubscriptionItem>,
            pub payment_metadata: Option<PaymentRequestMetadata>,
        }

        async fn get_payment_intent_metadata(
            metadata: HashMap<String, String>,
            pool: &PgPool,
            redis: &RedisPool,
            charge_status: ChargeStatus,
            transaction: &mut Transaction<'_, Postgres>,
        ) -> Result<PaymentIntentMetadata, ApiError> {
            'metadata: {
                let user_id = if let Some(user_id) = metadata
                    .get("modrinth_user_id")
                    .and_then(|x| parse_base62(x).ok())
                    .map(|x| crate::database::models::ids::UserId(x as i64))
                {
                    user_id
                } else {
                    break 'metadata;
                };

                let user = if let Some(user) =
                    crate::database::models::user_item::User::get_id(
                        user_id, pool, redis,
                    )
                    .await?
                {
                    user
                } else {
                    break 'metadata;
                };

                let payment_metadata = metadata
                    .get("modrinth_payment_metadata")
                    .and_then(|x| serde_json::from_str(x).ok());

                let charge_id = if let Some(charge_id) = metadata
                    .get("modrinth_charge_id")
                    .and_then(|x| parse_base62(x).ok())
                    .map(|x| crate::database::models::ids::ChargeId(x as i64))
                {
                    charge_id
                } else {
                    break 'metadata;
                };

                let (charge, price, product, subscription) = if let Some(
                    mut charge,
                ) =
                    crate::database::models::charge_item::ChargeItem::get(
                        charge_id, pool,
                    )
                    .await?
                {
                    let price = if let Some(price) =
                        product_item::ProductPriceItem::get(
                            charge.price_id,
                            pool,
                        )
                        .await?
                    {
                        price
                    } else {
                        break 'metadata;
                    };

                    let product = if let Some(product) =
                        product_item::ProductItem::get(price.product_id, pool)
                            .await?
                    {
                        product
                    } else {
                        break 'metadata;
                    };

                    charge.status = charge_status;
                    charge.last_attempt = Some(Utc::now());
                    charge.upsert(transaction).await?;

                    if let Some(subscription_id) = charge.subscription_id {
                        let mut subscription = if let Some(subscription) =
                            user_subscription_item::UserSubscriptionItem::get(
                                subscription_id,
                                pool,
                            )
                            .await?
                        {
                            subscription
                        } else {
                            break 'metadata;
                        };

                        match charge.type_ {
                            ChargeType::OneTime | ChargeType::Subscription => {
                                if let Some(interval) =
                                    charge.subscription_interval
                                {
                                    subscription.interval = interval;
                                }
                            }
                            ChargeType::Proration => {
                                subscription.price_id = charge.price_id;
                            }
                        }

                        subscription.upsert(transaction).await?;

                        (charge, price, product, Some(subscription))
                    } else {
                        (charge, price, product, None)
                    }
                } else {
                    let price_id = if let Some(price_id) = metadata
                        .get("modrinth_price_id")
                        .and_then(|x| parse_base62(x).ok())
                        .map(|x| {
                            crate::database::models::ids::ProductPriceId(
                                x as i64,
                            )
                        }) {
                        price_id
                    } else {
                        break 'metadata;
                    };

                    let price = if let Some(price) =
                        product_item::ProductPriceItem::get(price_id, pool)
                            .await?
                    {
                        price
                    } else {
                        break 'metadata;
                    };

                    let product = if let Some(product) =
                        product_item::ProductItem::get(price.product_id, pool)
                            .await?
                    {
                        product
                    } else {
                        break 'metadata;
                    };

                    let (amount, subscription) = match &price.prices {
                        Price::OneTime { price } => (*price, None),
                        Price::Recurring { intervals } => {
                            let interval = if let Some(interval) = metadata
                                .get("modrinth_subscription_interval")
                                .map(|x| PriceDuration::from_string(x))
                            {
                                interval
                            } else {
                                break 'metadata;
                            };

                            if let Some(price) = intervals.get(&interval) {
                                let subscription_id = if let Some(subscription_id) = metadata
                                    .get("modrinth_subscription_id")
                                    .and_then(|x| parse_base62(x).ok())
                                    .map(|x| {
                                        crate::database::models::ids::UserSubscriptionId(x as i64)
                                    }) {
                                    subscription_id
                                } else {
                                    break 'metadata;
                                };

                                let subscription = user_subscription_item::UserSubscriptionItem {
                                    id: subscription_id,
                                    user_id,
                                    price_id,
                                    interval,
                                    created: Utc::now(),
                                    status: SubscriptionStatus::Unprovisioned,
                                    metadata: None,
                                };

                                if charge_status != ChargeStatus::Failed {
                                    subscription.upsert(transaction).await?;
                                }

                                (*price, Some(subscription))
                            } else {
                                break 'metadata;
                            }
                        }
                    };

                    let charge = ChargeItem {
                        id: charge_id,
                        user_id,
                        price_id,
                        amount: amount as i64,
                        currency_code: price.currency_code.clone(),
                        status: charge_status,
                        due: Utc::now(),
                        last_attempt: Some(Utc::now()),
                        type_: if subscription.is_some() {
                            ChargeType::Subscription
                        } else {
                            ChargeType::OneTime
                        },
                        subscription_id: subscription.as_ref().map(|x| x.id),
                        subscription_interval: subscription
                            .as_ref()
                            .map(|x| x.interval),
                    };

                    if charge_status != ChargeStatus::Failed {
                        charge.upsert(transaction).await?;
                    }

                    (charge, price, product, subscription)
                };

                return Ok(PaymentIntentMetadata {
                    user_item: user,
                    product_price_item: price,
                    product_item: product,
                    charge_item: charge,
                    user_subscription_item: subscription,
                    payment_metadata,
                });
            }

            Err(ApiError::InvalidInput(
                "Webhook missing required webhook metadata!".to_string(),
            ))
        }

        match event.type_ {
            EventType::PaymentIntentSucceeded => {
                if let EventObject::PaymentIntent(payment_intent) =
                    event.data.object
                {
                    let mut transaction = pool.begin().await?;

                    let mut metadata = get_payment_intent_metadata(
                        payment_intent.metadata,
                        &pool,
                        &redis,
                        ChargeStatus::Succeeded,
                        &mut transaction,
                    )
                    .await?;

                    // Provision subscription
                    match metadata.product_item.metadata {
                        ProductMetadata::Midas => {
                            let badges =
                                metadata.user_item.badges | Badges::MIDAS;

                            sqlx::query!(
                                "
                                UPDATE users
                                SET badges = $1
                                WHERE (id = $2)
                                ",
                                badges.bits() as i64,
                                metadata.user_item.id
                                    as crate::database::models::ids::UserId,
                            )
                            .execute(&mut *transaction)
                            .await?;
                        }
                        ProductMetadata::Pyro {
                            ram,
                            cpu,
                            swap,
                            storage,
                        } => {
                            if let Some(ref subscription) =
                                metadata.user_subscription_item
                            {
                                let client = reqwest::Client::new();

                                if let Some(SubscriptionMetadata::Pyro { id }) =
                                    &subscription.metadata
                                {
                                    client
                                        .post(format!(
                                            "https://archon.pyro.host/modrinth/v0/servers/{}/unsuspend",
                                            id
                                        ))
                                        .header("X-Master-Key", dotenvy::var("PYRO_API_KEY")?)
                                        .send()
                                        .await?
                                        .error_for_status()?;

                                    // TODO: Send plan upgrade request for proration
                                } else {
                                    let (server_name, source) = if let Some(
                                        PaymentRequestMetadata::Pyro {
                                            ref server_name,
                                            ref source,
                                        },
                                    ) =
                                        metadata.payment_metadata
                                    {
                                        (server_name.clone(), source.clone())
                                    } else {
                                        // Create a server with the latest version of Minecraft
                                        let minecraft_versions = crate::database::models::legacy_loader_fields::MinecraftGameVersion::list(
                                            Some("release"),
                                            None,
                                            &**pool,
                                            &redis,
                                        ).await?;

                                        (
                                            None,
                                            serde_json::json!({
                                                "loader": "Vanilla",
                                                "game_version": minecraft_versions.first().map(|x| x.version.clone()),
                                                "loader_version": ""
                                            }),
                                        )
                                    };

                                    let server_name = server_name
                                        .unwrap_or_else(|| {
                                            format!(
                                                "{}'s server",
                                                metadata.user_item.username
                                            )
                                        });

                                    #[derive(Deserialize)]
                                    struct PyroServerResponse {
                                        uuid: String,
                                    }

                                    let res = client
                                        .post("https://archon.pyro.host/modrinth/v0/servers/create")
                                        .header("X-Master-Key", dotenvy::var("PYRO_API_KEY")?)
                                        .json(&serde_json::json!({
                                            "user_id": to_base62(metadata.user_item.id.0 as u64),
                                            "name": server_name,
                                            "specs": {
                                                "memory_mb": ram,
                                                "cpu": cpu,
                                                "swap_mb": swap,
                                                "storage_mb": storage,
                                            },
                                            "source": source,
                                        }))
                                        .send()
                                        .await?
                                        .error_for_status()?
                                        .json::<PyroServerResponse>()
                                        .await?;

                                    if let Some(ref mut subscription) =
                                        metadata.user_subscription_item
                                    {
                                        subscription.metadata =
                                            Some(SubscriptionMetadata::Pyro {
                                                id: res.uuid,
                                            });
                                    }
                                }
                            }
                        }
                    }

                    if let Some(mut subscription) =
                        metadata.user_subscription_item
                    {
                        let open_charge = ChargeItem::get_open_subscription(
                            subscription.id,
                            &mut *transaction,
                        )
                        .await?;

                        let new_price = match metadata.product_price_item.prices {
                            Price::OneTime { price } => price,
                            Price::Recurring { intervals } => {
                                *intervals.get(&subscription.interval).ok_or_else(|| {
                                    ApiError::InvalidInput(
                                        "Could not find a valid price for the user's country"
                                            .to_string(),
                                    )
                                })?
                            }
                        };

                        if let Some(mut charge) = open_charge {
                            charge.price_id = metadata.product_price_item.id;
                            charge.amount = new_price as i64;

                            charge.upsert(&mut transaction).await?;
                        } else if metadata.charge_item.status
                            != ChargeStatus::Cancelled
                        {
                            let charge_id =
                                generate_charge_id(&mut transaction).await?;
                            ChargeItem {
                                id: charge_id,
                                user_id: metadata.user_item.id,
                                price_id: metadata.product_price_item.id,
                                amount: new_price as i64,
                                currency_code: metadata
                                    .product_price_item
                                    .currency_code,
                                status: ChargeStatus::Open,
                                due: if subscription.status
                                    == SubscriptionStatus::Unprovisioned
                                {
                                    Utc::now()
                                        + subscription.interval.duration()
                                } else {
                                    metadata.charge_item.due
                                        + subscription.interval.duration()
                                },
                                last_attempt: None,
                                type_: ChargeType::Subscription,
                                subscription_id: Some(subscription.id),
                                subscription_interval: Some(
                                    subscription.interval,
                                ),
                            }
                            .upsert(&mut transaction)
                            .await?;
                        };

                        subscription.status = SubscriptionStatus::Provisioned;
                        subscription.upsert(&mut transaction).await?;
                    }

                    transaction.commit().await?;
                    crate::database::models::user_item::User::clear_caches(
                        &[(metadata.user_item.id, None)],
                        &redis,
                    )
                    .await?;
                }
            }
            EventType::PaymentIntentProcessing => {
                if let EventObject::PaymentIntent(payment_intent) =
                    event.data.object
                {
                    let mut transaction = pool.begin().await?;
                    get_payment_intent_metadata(
                        payment_intent.metadata,
                        &pool,
                        &redis,
                        ChargeStatus::Processing,
                        &mut transaction,
                    )
                    .await?;
                    transaction.commit().await?;
                }
            }
            EventType::PaymentIntentPaymentFailed => {
                if let EventObject::PaymentIntent(payment_intent) =
                    event.data.object
                {
                    let mut transaction = pool.begin().await?;

                    let metadata = get_payment_intent_metadata(
                        payment_intent.metadata,
                        &pool,
                        &redis,
                        ChargeStatus::Failed,
                        &mut transaction,
                    )
                    .await?;

                    if let Some(email) = metadata.user_item.email {
                        let money = rusty_money::Money::from_minor(
                            metadata.charge_item.amount,
                            rusty_money::iso::find(
                                &metadata.charge_item.currency_code,
                            )
                            .unwrap_or(rusty_money::iso::USD),
                        );

                        let _ = send_email(
                            email,
                            "Payment Failed for Modrinth",
                            &format!("Our attempt to collect payment for {money} from the payment card on file was unsuccessful."),
                            "Please visit the following link below to update your payment method or contact your card provider. If the button does not work, you can copy the link and paste it into your browser.",
                            Some(("Update billing settings", &format!("{}/{}", dotenvy::var("SITE_URL")?,  dotenvy::var("SITE_BILLING_PATH")?))),
                        );
                    }

                    transaction.commit().await?;
                }
            }
            EventType::PaymentMethodAttached => {
                if let EventObject::PaymentMethod(payment_method) =
                    event.data.object
                {
                    if let Some(customer_id) =
                        payment_method.customer.map(|x| x.id())
                    {
                        let customer = stripe::Customer::retrieve(
                            &stripe_client,
                            &customer_id,
                            &[],
                        )
                        .await?;

                        if !customer
                            .invoice_settings
                            .map(|x| x.default_payment_method.is_some())
                            .unwrap_or(false)
                        {
                            stripe::Customer::update(
                                &stripe_client,
                                &customer_id,
                                UpdateCustomer {
                                    invoice_settings: Some(
                                        CustomerInvoiceSettings {
                                            default_payment_method: Some(
                                                payment_method.id.to_string(),
                                            ),
                                            ..Default::default()
                                        },
                                    ),
                                    ..Default::default()
                                },
                            )
                            .await?;
                        }
                    }
                }
            }
            _ => {}
        }
    } else {
        return Err(ApiError::InvalidInput(
            "Webhook signature validation failed!".to_string(),
        ));
    }

    Ok(HttpResponse::Ok().finish())
}

async fn get_or_create_customer(
    user_id: crate::models::ids::UserId,
    stripe_customer_id: Option<&str>,
    user_email: Option<&str>,
    client: &stripe::Client,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<CustomerId, ApiError> {
    if let Some(customer_id) =
        stripe_customer_id.and_then(|x| stripe::CustomerId::from_str(x).ok())
    {
        Ok(customer_id)
    } else {
        let mut metadata = HashMap::new();
        metadata.insert("modrinth_user_id".to_string(), to_base62(user_id.0));

        let customer = stripe::Customer::create(
            client,
            CreateCustomer {
                email: user_email,
                metadata: Some(metadata),
                ..Default::default()
            },
        )
        .await?;

        sqlx::query!(
            "
            UPDATE users
            SET stripe_customer_id = $1
            WHERE id = $2
            ",
            customer.id.as_str(),
            user_id.0 as i64
        )
        .execute(pool)
        .await?;

        crate::database::models::user_item::User::clear_caches(
            &[(user_id.into(), None)],
            redis,
        )
        .await?;

        Ok(customer.id)
    }
}

pub async fn subscription_task(pool: PgPool, redis: RedisPool) {
    loop {
        info!("Indexing subscriptions");

        let res = async {
            let mut transaction = pool.begin().await?;
            let mut clear_cache_users = Vec::new();

            // If an active subscription has a canceled charge OR a failed charge more than two days ago, it should be cancelled
            let all_charges = ChargeItem::get_unprovision(&pool).await?;

            let mut all_subscriptions =
                user_subscription_item::UserSubscriptionItem::get_many(
                    &all_charges
                        .iter()
                        .filter_map(|x| x.subscription_id)
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>(),
                    &pool,
                )
                .await?;
            let subscription_prices = product_item::ProductPriceItem::get_many(
                &all_subscriptions
                    .iter()
                    .map(|x| x.price_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                &pool,
            )
            .await?;
            let subscription_products = product_item::ProductItem::get_many(
                &subscription_prices
                    .iter()
                    .map(|x| x.product_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                &pool,
            )
            .await?;
            let users = crate::database::models::User::get_many_ids(
                &all_subscriptions
                    .iter()
                    .map(|x| x.user_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                &pool,
                &redis,
            )
            .await?;

            for charge in all_charges {
                let subscription = if let Some(subscription) = all_subscriptions
                    .iter_mut()
                    .find(|x| Some(x.id) == charge.subscription_id)
                {
                    subscription
                } else {
                    continue;
                };

                if subscription.status == SubscriptionStatus::Unprovisioned {
                    continue;
                }

                let product_price = if let Some(product_price) =
                    subscription_prices
                        .iter()
                        .find(|x| x.id == subscription.price_id)
                {
                    product_price
                } else {
                    continue;
                };

                let product = if let Some(product) = subscription_products
                    .iter()
                    .find(|x| x.id == product_price.product_id)
                {
                    product
                } else {
                    continue;
                };

                let user = if let Some(user) =
                    users.iter().find(|x| x.id == subscription.user_id)
                {
                    user
                } else {
                    continue;
                };

                let unprovisioned = match product.metadata {
                    ProductMetadata::Midas => {
                        let badges = user.badges - Badges::MIDAS;

                        sqlx::query!(
                            "
                            UPDATE users
                            SET badges = $1
                            WHERE (id = $2)
                            ",
                            badges.bits() as i64,
                            user.id as crate::database::models::ids::UserId,
                        )
                        .execute(&mut *transaction)
                        .await?;

                        true
                    }
                    ProductMetadata::Pyro { .. } => {
                        if let Some(SubscriptionMetadata::Pyro { id }) =
                            &subscription.metadata
                        {
                            let res = reqwest::Client::new()
                                .post(format!(
                                    "https://archon.pyro.host/modrinth/v0/servers/{}/suspend",
                                    id
                                ))
                                .header("X-Master-Key", dotenvy::var("PYRO_API_KEY")?)
                                .json(&serde_json::json!({
                                    "reason": if charge.status == ChargeStatus::Cancelled {
                                        "cancelled"
                                    } else {
                                        "paymentfailed"
                                    }
                                }))
                                .send()
                                .await;

                            if let Err(e) = res {
                                warn!("Error suspending pyro server: {:?}", e);
                                false
                            } else {
                                true
                            }
                        } else {
                            true
                        }
                    }
                };

                if unprovisioned {
                    subscription.status = SubscriptionStatus::Unprovisioned;
                    subscription.upsert(&mut transaction).await?;
                }

                clear_cache_users.push(user.id);
            }

            crate::database::models::User::clear_caches(
                &clear_cache_users
                    .into_iter()
                    .map(|x| (x, None))
                    .collect::<Vec<_>>(),
                &redis,
            )
            .await?;
            transaction.commit().await?;

            Ok::<(), ApiError>(())
        };

        if let Err(e) = res.await {
            warn!("Error indexing billing queue: {:?}", e);
        }

        info!("Done indexing billing queue");

        tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
    }
}

pub async fn task(
    stripe_client: stripe::Client,
    pool: PgPool,
    redis: RedisPool,
) {
    loop {
        info!("Indexing billing queue");
        let res = async {
            // If a charge is open and due or has been attempted more than two days ago, it should be processed
            let charges_to_do =
                crate::database::models::charge_item::ChargeItem::get_chargeable(&pool).await?;

            let prices = product_item::ProductPriceItem::get_many(
                &charges_to_do
                    .iter()
                    .map(|x| x.price_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                &pool,
            )
            .await?;

            let users = crate::database::models::User::get_many_ids(
                &charges_to_do
                    .iter()
                    .map(|x| x.user_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                &pool,
                &redis,
            )
            .await?;

            let mut transaction = pool.begin().await?;

            for mut charge in charges_to_do {
                let product_price =
                    if let Some(price) = prices.iter().find(|x| x.id == charge.price_id) {
                        price
                    } else {
                        continue;
                    };

                let user = if let Some(user) = users.iter().find(|x| x.id == charge.user_id) {
                    user
                } else {
                    continue;
                };

                let price = match &product_price.prices {
                    Price::OneTime { price } => Some(price),
                    Price::Recurring { intervals } => {
                        if let Some(ref interval) = charge.subscription_interval {
                            intervals.get(interval)
                        } else {
                            warn!("Could not find subscription for charge {:?}", charge.id);
                            continue;
                        }
                    }
                };

                if let Some(price) = price {
                    let customer_id = get_or_create_customer(
                        user.id.into(),
                        user.stripe_customer_id.as_deref(),
                        user.email.as_deref(),
                        &stripe_client,
                        &pool,
                        &redis,
                    )
                    .await?;

                    let customer =
                        stripe::Customer::retrieve(&stripe_client, &customer_id, &[]).await?;

                    let currency =
                        match Currency::from_str(&product_price.currency_code.to_lowercase()) {
                            Ok(x) => x,
                            Err(_) => {
                                warn!(
                                    "Could not find currency for {}",
                                    product_price.currency_code
                                );
                                continue;
                            }
                        };

                    let mut intent = CreatePaymentIntent::new(*price as i64, currency);

                    let mut metadata = HashMap::new();
                    metadata.insert(
                        "modrinth_user_id".to_string(),
                        to_base62(charge.user_id.0 as u64),
                    );
                    metadata.insert(
                        "modrinth_charge_id".to_string(),
                        to_base62(charge.id.0 as u64),
                    );

                    intent.metadata = Some(metadata);
                    intent.customer = Some(customer.id);

                    if let Some(payment_method) = customer
                        .invoice_settings
                        .and_then(|x| x.default_payment_method.map(|x| x.id()))
                    {
                        intent.payment_method = Some(payment_method);
                        intent.confirm = Some(true);
                        intent.off_session = Some(PaymentIntentOffSession::Exists(true));

                        charge.status = ChargeStatus::Processing;

                        stripe::PaymentIntent::create(&stripe_client, intent).await?;
                    } else {
                        charge.status = ChargeStatus::Failed;
                        charge.last_attempt = Some(Utc::now());
                    }

                    charge.upsert(&mut transaction).await?;
                }
            }

            transaction.commit().await?;

            Ok::<(), ApiError>(())
        }
        .await;

        if let Err(e) = res {
            warn!("Error indexing billing queue: {:?}", e);
        }

        info!("Done indexing billing queue");

        tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
    }
}
