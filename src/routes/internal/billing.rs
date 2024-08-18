use crate::auth::{get_user_from_headers, send_email};
use crate::database::models::{
    generate_user_subscription_id, product_item, user_subscription_item,
};
use crate::database::redis::RedisPool;
use crate::models::billing::{
    Price, PriceDuration, Product, ProductMetadata, ProductPrice, SubscriptionStatus,
    UserSubscription,
};
use crate::models::ids::base62_impl::{parse_base62, to_base62};
use crate::models::pats::Scopes;
use crate::models::users::Badges;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use log::{info, warn};
use serde_with::serde_derive::Deserialize;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use stripe::{
    CreateCustomer, CreatePaymentIntent, CreatePaymentIntentAutomaticPaymentMethods,
    CreateSetupIntent, CreateSetupIntentAutomaticPaymentMethods,
    CreateSetupIntentAutomaticPaymentMethodsAllowRedirects, Currency, CustomerId,
    CustomerInvoiceSettings, CustomerPaymentMethodRetrieval, EventObject, EventType, ListCharges,
    PaymentIntentOffSession, PaymentIntentSetupFutureUsage, PaymentMethodId, SetupIntent,
    UpdateCustomer, Webhook,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("billing")
            .service(products)
            .service(subscriptions)
            .service(user_customer)
            .service(cancel_subscription)
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
        user_subscription_item::UserSubscriptionItem::get_all_user(user.id.into(), &**pool)
            .await?
            .into_iter()
            .map(|x| UserSubscription {
                id: x.id.into(),
                user_id: x.user_id.into(),
                price_id: x.price_id.into(),
                interval: x.interval,
                status: x.status,
                created: x.created,
                expires: x.expires,
                last_charge: x.last_charge,
            })
            .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(subscriptions))
}

#[delete("subscription/{id}")]
pub async fn cancel_subscription(
    req: HttpRequest,
    info: web::Path<(crate::models::ids::UserSubscriptionId,)>,
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

    let (id,) = info.into_inner();

    if let Some(mut subscription) =
        user_subscription_item::UserSubscriptionItem::get(id.into(), &**pool).await?
    {
        if subscription.user_id != user.id.into() && !user.role.is_admin() {
            return Err(ApiError::NotFound);
        }

        let mut transaction = pool.begin().await?;

        if subscription.expires < Utc::now() {
            sqlx::query!(
                "
                DELETE FROM users_subscriptions
                WHERE id = $1
                ",
                subscription.id.0 as i64
            )
            .execute(&mut *transaction)
            .await?;
        } else {
            subscription.status = SubscriptionStatus::Cancelled;
            subscription.upsert(&mut transaction).await?;
        }

        transaction.commit().await?;

        Ok(HttpResponse::NoContent().body(""))
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
    let customer = stripe::Customer::retrieve(&stripe_client, &customer_id, &[]).await?;

    Ok(HttpResponse::Ok().json(customer))
}

#[get("payments")]
pub async fn charges(
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
        let charges = stripe::Charge::list(
            &stripe_client,
            &ListCharges {
                customer: Some(customer_id),
                limit: Some(100),
                ..Default::default()
            },
        )
        .await?;

        Ok(HttpResponse::Ok().json(charges.data))
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
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

    let payment_method =
        stripe::PaymentMethod::retrieve(&stripe_client, &payment_method_id, &[]).await?;

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

    let payment_method =
        stripe::PaymentMethod::retrieve(&stripe_client, &payment_method_id, &[]).await?;

    let user_subscriptions =
        user_subscription_item::UserSubscriptionItem::get_all_user(user.id.into(), &**pool).await?;

    if user_subscriptions
        .iter()
        .any(|x| x.status != SubscriptionStatus::Cancelled)
    {
        let customer = stripe::Customer::retrieve(&stripe_client, &customer, &[]).await?;

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
        stripe::PaymentMethod::detach(&stripe_client, &payment_method_id).await?;

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
pub struct PaymentRequest {
    pub product_id: crate::models::ids::ProductId,
    pub interval: Option<PriceDuration>,
    #[serde(flatten)]
    pub type_: PaymentRequestType,
    pub existing_payment_intent: Option<stripe::PaymentIntentId>,
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

    let product = product_item::ProductItem::get(payment_request.product_id.into(), &**pool)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("Specified product could not be found!".to_string())
        })?;

    let (user_country, payment_method) = match &payment_request.type_ {
        PaymentRequestType::PaymentMethod { id } => {
            let payment_method_id = stripe::PaymentMethodId::from_str(id)
                .map_err(|_| ApiError::InvalidInput("Invalid payment method id".to_string()))?;

            let payment_method =
                stripe::PaymentMethod::retrieve(&stripe_client, &payment_method_id, &[]).await?;

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

            let confirmation: ConfirmationToken = serde_json::from_value(confirmation)?;

            let payment_method = confirmation.payment_method_preview.ok_or_else(|| {
                ApiError::InvalidInput("Confirmation token is missing payment method!".to_string())
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
    let recommended_currency_code = match country {
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
    };

    let mut product_prices =
        product_item::ProductPriceItem::get_all_product_prices(product.id, &**pool).await?;

    let price_item = if let Some(pos) = product_prices
        .iter()
        .position(|x| x.currency_code == recommended_currency_code)
    {
        product_prices.remove(pos)
    } else if let Some(pos) = product_prices.iter().position(|x| x.currency_code == "USD") {
        product_prices.remove(pos)
    } else {
        return Err(ApiError::InvalidInput(
            "Could not find a valid price for the user's country".to_string(),
        ));
    };

    let price = match price_item.prices {
        Price::OneTime { price } => price,
        Price::Recurring { ref intervals } => {
            let interval = payment_request.interval.ok_or_else(|| {
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

    let customer = get_or_create_customer(
        user.id,
        user.stripe_customer_id.as_deref(),
        user.email.as_deref(),
        &stripe_client,
        &pool,
        &redis,
    )
    .await?;
    let stripe_currency = Currency::from_str(&price_item.currency_code.to_lowercase())
        .map_err(|_| ApiError::InvalidInput("Invalid currency code".to_string()))?;

    if let Some(payment_intent_id) = &payment_request.existing_payment_intent {
        let mut update_payment_intent = stripe::UpdatePaymentIntent {
            amount: Some(price as i64),
            currency: Some(stripe_currency),
            customer: Some(customer),
            ..Default::default()
        };

        let mut metadata = HashMap::new();
        metadata.insert("modrinth_user_id".to_string(), to_base62(user.id.0));
        metadata.insert(
            "modrinth_price_id".to_string(),
            to_base62(price_item.id.0 as u64),
        );
        if let Some(interval) = payment_request.interval {
            metadata.insert(
                "modrinth_subscription_interval".to_string(),
                interval.as_str().to_string(),
            );
        }
        update_payment_intent.metadata = Some(metadata);

        if let PaymentRequestType::PaymentMethod { .. } = payment_request.type_ {
            update_payment_intent.payment_method = Some(payment_method.id.clone());
        }

        stripe::PaymentIntent::update(&stripe_client, payment_intent_id, update_payment_intent)
            .await?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "price_id": to_base62(price_item.id.0 as u64),
            "tax": 0,
            "total": price,
            "payment_method": payment_method,
        })))
    } else {
        let mut intent = CreatePaymentIntent::new(price as i64, stripe_currency);

        let mut transaction = pool.begin().await?;
        let mut metadata = HashMap::new();
        metadata.insert("modrinth_user_id".to_string(), to_base62(user.id.0));
        metadata.insert(
            "modrinth_price_id".to_string(),
            to_base62(price_item.id.0 as u64),
        );

        if let Price::Recurring { .. } = price_item.prices {
            if product.unitary {
                let user_subscriptions =
                    user_subscription_item::UserSubscriptionItem::get_all_user(
                        user.id.into(),
                        &**pool,
                    )
                    .await?;

                let user_products = product_item::ProductPriceItem::get_many(
                    &user_subscriptions
                        .iter()
                        .map(|x| x.price_id)
                        .collect::<Vec<_>>(),
                    &**pool,
                )
                .await?;

                if let Some(product) = user_products
                    .into_iter()
                    .find(|x| x.product_id == product.id)
                {
                    if let Some(subscription) = user_subscriptions
                        .into_iter()
                        .find(|x| x.price_id == product.id)
                    {
                        if subscription.status == SubscriptionStatus::Cancelled
                            || subscription.status == SubscriptionStatus::PaymentFailed
                        {
                            metadata.insert(
                                "modrinth_subscription_id".to_string(),
                                to_base62(subscription.id.0 as u64),
                            );
                        } else {
                            return Err(ApiError::InvalidInput(
                                "You are already subscribed to this product!".to_string(),
                            ));
                        }
                    }
                }
            }

            if !metadata.contains_key("modrinth_subscription_id") {
                let user_subscription_id = generate_user_subscription_id(&mut transaction).await?;

                metadata.insert(
                    "modrinth_subscription_id".to_string(),
                    to_base62(user_subscription_id.0 as u64),
                );
            }

            if let Some(interval) = payment_request.interval {
                metadata.insert(
                    "modrinth_subscription_interval".to_string(),
                    interval.as_str().to_string(),
                );
            }
        }

        intent.customer = Some(customer);
        intent.metadata = Some(metadata);
        intent.automatic_payment_methods = Some(CreatePaymentIntentAutomaticPaymentMethods {
            allow_redirects: None,
            enabled: false,
        });
        intent.receipt_email = user.email.as_deref();
        intent.setup_future_usage = Some(PaymentIntentSetupFutureUsage::OffSession);
        intent.payment_method_types = Some(vec!["card".to_string(), "cashapp".to_string()]);

        if let PaymentRequestType::PaymentMethod { .. } = payment_request.type_ {
            intent.payment_method = Some(payment_method.id.clone());
        }

        let payment_intent = stripe::PaymentIntent::create(&stripe_client, intent).await?;
        transaction.commit().await?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "payment_intent_id": payment_intent.id,
            "client_secret": payment_intent.client_secret,
            "price_id": to_base62(price_item.id.0 as u64),
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
            user: crate::database::models::User,
            user_subscription_data: Option<(
                crate::database::models::ids::UserSubscriptionId,
                PriceDuration,
            )>,
            user_subscription: Option<user_subscription_item::UserSubscriptionItem>,
            product: product_item::ProductItem,
            product_price: product_item::ProductPriceItem,
        }

        async fn get_payment_intent_metadata(
            metadata: HashMap<String, String>,
            pool: &PgPool,
            redis: &RedisPool,
        ) -> Result<PaymentIntentMetadata, ApiError> {
            if let Some(user_id) = metadata
                .get("modrinth_user_id")
                .and_then(|x| parse_base62(x).ok())
                .map(|x| crate::database::models::ids::UserId(x as i64))
            {
                let user =
                    crate::database::models::user_item::User::get_id(user_id, pool, redis).await?;

                if let Some(user) = user {
                    let (user_subscription_data, user_subscription) = if let Some(subscription_id) =
                        metadata
                            .get("modrinth_subscription_id")
                            .and_then(|x| parse_base62(x).ok())
                            .map(|x| crate::database::models::ids::UserSubscriptionId(x as i64))
                    {
                        if let Some(interval) = metadata
                            .get("modrinth_subscription_interval")
                            .map(|x| PriceDuration::from_string(x))
                        {
                            let subscription = user_subscription_item::UserSubscriptionItem::get(
                                subscription_id,
                                pool,
                            )
                            .await?;

                            (Some((subscription_id, interval)), subscription)
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    };

                    if let Some(price_id) = metadata
                        .get("modrinth_price_id")
                        .and_then(|x| parse_base62(x).ok())
                        .map(|x| crate::database::models::ids::ProductPriceId(x as i64))
                    {
                        let price = product_item::ProductPriceItem::get(price_id, pool).await?;

                        if let Some(product_price) = price {
                            let product =
                                product_item::ProductItem::get(product_price.product_id, pool)
                                    .await?;

                            if let Some(product) = product {
                                return Ok(PaymentIntentMetadata {
                                    user,
                                    user_subscription_data,
                                    user_subscription,
                                    product,
                                    product_price,
                                });
                            }
                        }
                    }
                }
            }

            Err(ApiError::InvalidInput(
                "Webhook missing required webhook metadata!".to_string(),
            ))
        }

        match event.type_ {
            EventType::PaymentIntentSucceeded => {
                if let EventObject::PaymentIntent(payment_intent) = event.data.object {
                    let metadata =
                        get_payment_intent_metadata(payment_intent.metadata, &pool, &redis).await?;

                    let mut transaction = pool.begin().await?;

                    if let Some((subscription_id, interval)) = metadata.user_subscription_data {
                        let duration = match interval {
                            PriceDuration::Monthly => Duration::days(30),
                            PriceDuration::Yearly => Duration::days(365),
                        };

                        if let Some(mut user_subscription) = metadata.user_subscription {
                            user_subscription.expires += duration;
                            user_subscription.status = SubscriptionStatus::Active;
                            user_subscription.interval = interval;
                            user_subscription.price_id = metadata.product_price.id;
                            user_subscription.upsert(&mut transaction).await?;
                        } else {
                            user_subscription_item::UserSubscriptionItem {
                                id: subscription_id,
                                user_id: metadata.user.id,
                                price_id: metadata.product_price.id,
                                interval,
                                created: Utc::now(),
                                expires: Utc::now() + duration,
                                last_charge: None,
                                status: SubscriptionStatus::Active,
                            }
                            .upsert(&mut transaction)
                            .await?;
                        }
                    }

                    // Provision subscription
                    match metadata.product.metadata {
                        ProductMetadata::Midas => {
                            let badges = metadata.user.badges | Badges::MIDAS;

                            sqlx::query!(
                                "
                                UPDATE users
                                SET badges = $1
                                WHERE (id = $2)
                                ",
                                badges.bits() as i64,
                                metadata.user.id as crate::database::models::ids::UserId,
                            )
                            .execute(&mut *transaction)
                            .await?;
                        }
                    }

                    transaction.commit().await?;
                    crate::database::models::user_item::User::clear_caches(
                        &[(metadata.user.id, None)],
                        &redis,
                    )
                    .await?;
                }
            }
            EventType::PaymentIntentProcessing => {
                if let EventObject::PaymentIntent(payment_intent) = event.data.object {
                    let metadata =
                        get_payment_intent_metadata(payment_intent.metadata, &pool, &redis).await?;

                    let mut transaction = pool.begin().await?;

                    if let Some((subscription_id, interval)) = metadata.user_subscription_data {
                        if let Some(mut user_subscription) = metadata.user_subscription {
                            user_subscription.status = SubscriptionStatus::PaymentProcessing;
                            user_subscription.interval = interval;
                            user_subscription.price_id = metadata.product_price.id;
                            user_subscription.upsert(&mut transaction).await?;
                        } else {
                            user_subscription_item::UserSubscriptionItem {
                                id: subscription_id,
                                user_id: metadata.user.id,
                                price_id: metadata.product_price.id,
                                interval,
                                created: Utc::now(),
                                expires: Utc::now(),
                                last_charge: None,
                                status: SubscriptionStatus::PaymentProcessing,
                            }
                            .upsert(&mut transaction)
                            .await?;
                        }
                    }

                    transaction.commit().await?;
                }
            }
            EventType::PaymentIntentPaymentFailed => {
                if let EventObject::PaymentIntent(payment_intent) = event.data.object {
                    let metadata =
                        get_payment_intent_metadata(payment_intent.metadata, &pool, &redis).await?;

                    let mut transaction = pool.begin().await?;

                    let price = match metadata.product_price.prices {
                        Price::OneTime { price } => Some(price),
                        Price::Recurring { intervals } => {
                            if let Some((subscription_id, interval)) =
                                metadata.user_subscription_data
                            {
                                if let Some(mut user_subscription) = metadata.user_subscription {
                                    user_subscription.last_charge = Some(Utc::now());
                                    user_subscription.status = SubscriptionStatus::PaymentFailed;
                                    user_subscription.price_id = metadata.product_price.id;
                                    user_subscription.interval = interval;
                                    user_subscription.upsert(&mut transaction).await?;

                                    intervals.get(&interval).copied()
                                } else {
                                    // We don't create a new subscription for a failed payment, so we return None here so no email is sent
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    };

                    if let Some(price) = price {
                        if let Some(email) = metadata.user.email {
                            let money = rusty_money::Money::from_minor(
                                price as i64,
                                rusty_money::iso::find(&metadata.product_price.currency_code)
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
                    }

                    transaction.commit().await?;
                }
            }
            EventType::PaymentMethodAttached => {
                if let EventObject::PaymentMethod(payment_method) = event.data.object {
                    if let Some(customer_id) = payment_method.customer.map(|x| x.id()) {
                        let customer =
                            stripe::Customer::retrieve(&stripe_client, &customer_id, &[]).await?;

                        if !customer
                            .invoice_settings
                            .map(|x| x.default_payment_method.is_some())
                            .unwrap_or(false)
                        {
                            stripe::Customer::update(
                                &stripe_client,
                                &customer_id,
                                UpdateCustomer {
                                    invoice_settings: Some(CustomerInvoiceSettings {
                                        default_payment_method: Some(payment_method.id.to_string()),
                                        ..Default::default()
                                    }),
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
    if let Some(customer_id) = stripe_customer_id.and_then(|x| stripe::CustomerId::from_str(x).ok())
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

        crate::database::models::user_item::User::clear_caches(&[(user_id.into(), None)], redis)
            .await?;

        Ok(customer.id)
    }
}

pub async fn task(stripe_client: stripe::Client, pool: PgPool, redis: RedisPool) {
    // if subscription is cancelled and expired, unprovision and remove
    // if subscription is payment failed and last attempt is > 2 days ago, try again to charge and unprovision
    // if subscription is active and expired, attempt to charge and set as processing
    loop {
        info!("Indexing billing queue");
        let res = async {
            let expired =
                user_subscription_item::UserSubscriptionItem::get_all_expired(&pool).await?;
            let subscription_prices = product_item::ProductPriceItem::get_many(
                &expired
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
                &expired
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
            let mut clear_cache_users = Vec::new();

            for mut subscription in expired {
                let user = users.iter().find(|x| x.id == subscription.user_id);

                if let Some(user) = user {
                    let product_price = subscription_prices
                        .iter()
                        .find(|x| x.id == subscription.price_id);

                    if let Some(product_price) = product_price {
                        let product = subscription_products
                            .iter()
                            .find(|x| x.id == product_price.product_id);

                        if let Some(product) = product {
                            let price = match &product_price.prices {
                                Price::OneTime { price } => Some(price),
                                Price::Recurring { intervals } => {
                                    intervals.get(&subscription.interval)
                                }
                            };

                            if let Some(price) = price {
                                let cancelled =
                                    subscription.status == SubscriptionStatus::Cancelled;
                                let payment_failed = subscription
                                    .last_charge
                                    .map(|y| {
                                        subscription.status == SubscriptionStatus::PaymentFailed
                                            && Utc::now() - y > Duration::days(2)
                                    })
                                    .unwrap_or(false);
                                let active = subscription.status == SubscriptionStatus::Active;

                                // Unprovision subscription
                                if cancelled || payment_failed {
                                    match product.metadata {
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
                                        }
                                    }

                                    clear_cache_users.push(user.id);
                                }

                                if cancelled {
                                    user_subscription_item::UserSubscriptionItem::remove(
                                        subscription.id,
                                        &mut transaction,
                                    )
                                    .await?;
                                } else if payment_failed || active {
                                    let customer_id = get_or_create_customer(
                                        user.id.into(),
                                        user.stripe_customer_id.as_deref(),
                                        user.email.as_deref(),
                                        &stripe_client,
                                        &pool,
                                        &redis,
                                    )
                                    .await?;

                                    let customer = stripe::Customer::retrieve(
                                        &stripe_client,
                                        &customer_id,
                                        &[],
                                    )
                                    .await?;

                                    let mut intent = CreatePaymentIntent::new(
                                        *price as i64,
                                        Currency::from_str(&product_price.currency_code)
                                            .unwrap_or(Currency::USD),
                                    );

                                    let mut metadata = HashMap::new();
                                    metadata.insert(
                                        "modrinth_user_id".to_string(),
                                        to_base62(user.id.0 as u64),
                                    );
                                    metadata.insert(
                                        "modrinth_price_id".to_string(),
                                        to_base62(product_price.id.0 as u64),
                                    );
                                    metadata.insert(
                                        "modrinth_subscription_id".to_string(),
                                        to_base62(subscription.id.0 as u64),
                                    );
                                    metadata.insert(
                                        "modrinth_subscription_interval".to_string(),
                                        subscription.interval.as_str().to_string(),
                                    );

                                    intent.metadata = Some(metadata);
                                    intent.customer = Some(customer_id);

                                    if let Some(payment_method) = customer
                                        .invoice_settings
                                        .and_then(|x| x.default_payment_method.map(|x| x.id()))
                                    {
                                        intent.payment_method = Some(payment_method);
                                        intent.confirm = Some(true);
                                        intent.off_session =
                                            Some(PaymentIntentOffSession::Exists(true));

                                        subscription.status = SubscriptionStatus::PaymentProcessing;
                                        stripe::PaymentIntent::create(&stripe_client, intent)
                                            .await?;
                                    } else {
                                        subscription.status = SubscriptionStatus::PaymentFailed;
                                    }

                                    subscription.upsert(&mut transaction).await?;
                                }
                            }
                        }
                    }
                }
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
        }
        .await;

        if let Err(e) = res {
            warn!("Error indexing billing queue: {:?}", e);
        }

        info!("Done indexing billing queue");

        tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
    }
}
