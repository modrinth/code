use crate::auth::{get_user_from_headers, send_email};
use crate::database::models::charge_item::DBCharge;
use crate::database::models::{
    generate_charge_id, generate_user_subscription_id, product_item,
    user_subscription_item,
};
use crate::database::redis::RedisPool;
use crate::models::billing::{
    Charge, ChargeStatus, ChargeType, PaymentPlatform, Price, PriceDuration,
    Product, ProductMetadata, ProductPrice, SubscriptionMetadata,
    SubscriptionStatus, UserSubscription,
};
use crate::models::pats::Scopes;
use crate::models::users::Badges;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use ariadne::ids::base62_impl::{parse_base62, to_base62};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use serde_with::serde_derive::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use stripe::{
    CreateCustomer, CreatePaymentIntent, CreateRefund, CreateSetupIntent,
    CreateSetupIntentAutomaticPaymentMethods,
    CreateSetupIntentAutomaticPaymentMethodsAllowRedirects, Currency,
    CustomerId, CustomerInvoiceSettings, CustomerPaymentMethodRetrieval,
    EventObject, EventType, PaymentIntentId, PaymentIntentOffSession,
    PaymentIntentSetupFutureUsage, PaymentMethodId, SetupIntent,
    UpdateCustomer, Webhook,
};
use tracing::{info, warn};

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
            .service(active_servers)
            .service(initiate_payment)
            .service(stripe_webhook)
            .service(refund_charge),
    );
}

#[get("products")]
pub async fn products(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let products =
        product_item::QueryProductWithPrices::list(&**pool, &redis).await?;

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

#[derive(Deserialize)]
struct SubscriptionsQuery {
    pub user_id: Option<ariadne::ids::UserId>,
}

#[get("subscriptions")]
pub async fn subscriptions(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    query: web::Query<SubscriptionsQuery>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?
    .1;

    let subscriptions =
        user_subscription_item::DBUserSubscription::get_all_user(
            if let Some(user_id) = query.user_id {
                if user.role.is_admin() {
                    user_id.into()
                } else {
                    return Err(ApiError::InvalidInput(
                        "You cannot see the subscriptions of other users!"
                            .to_string(),
                    ));
                }
            } else {
                user.id.into()
            },
            &**pool,
        )
        .await?
        .into_iter()
        .map(UserSubscription::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(subscriptions))
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChargeRefundAmount {
    Full,
    Partial { amount: u64 },
    None,
}

#[derive(Deserialize)]
pub struct ChargeRefund {
    #[serde(flatten)]
    pub amount: ChargeRefundAmount,
    pub unprovision: Option<bool>,
}

#[post("charge/{id}/refund")]
pub async fn refund_charge(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    info: web::Path<(crate::models::ids::ChargeId,)>,
    body: web::Json<ChargeRefund>,
    stripe_client: web::Data<stripe::Client>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?
    .1;

    let (id,) = info.into_inner();

    if !user.role.is_admin() {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to refund a subscription!".to_string(),
        ));
    }

    if let Some(charge) = DBCharge::get(id.into(), &**pool).await? {
        let refunds = DBCharge::get_children(id.into(), &**pool).await?;
        let refunds = -refunds
            .into_iter()
            .filter_map(|x| match x.status {
                ChargeStatus::Open
                | ChargeStatus::Processing
                | ChargeStatus::Succeeded => Some(x.amount),
                ChargeStatus::Failed | ChargeStatus::Cancelled => None,
            })
            .sum::<i64>();

        let refundable = charge.amount - refunds;

        let refund_amount = match body.0.amount {
            ChargeRefundAmount::Full => refundable,
            ChargeRefundAmount::Partial { amount } => amount as i64,
            ChargeRefundAmount::None => 0,
        };

        if charge.status != ChargeStatus::Succeeded {
            return Err(ApiError::InvalidInput(
                "This charge cannot be refunded!".to_string(),
            ));
        }

        if (refundable - refund_amount) < 0 {
            return Err(ApiError::InvalidInput(
                "You cannot refund more than the amount of the charge!"
                    .to_string(),
            ));
        }

        let (id, net) = if refund_amount == 0 {
            (None, None)
        } else {
            match charge.payment_platform {
                PaymentPlatform::Stripe => {
                    if let Some(payment_platform_id) =
                        charge.payment_platform_id.and_then(|x| {
                            stripe::PaymentIntentId::from_str(&x).ok()
                        })
                    {
                        let mut metadata = HashMap::new();

                        metadata.insert(
                            "modrinth_user_id".to_string(),
                            to_base62(user.id.0),
                        );
                        metadata.insert(
                            "modrinth_charge_id".to_string(),
                            to_base62(charge.id.0 as u64),
                        );

                        let refund = stripe::Refund::create(
                            &stripe_client,
                            CreateRefund {
                                amount: Some(refund_amount),
                                metadata: Some(metadata),
                                payment_intent: Some(payment_platform_id),

                                expand: &["balance_transaction"],

                                ..Default::default()
                            },
                        )
                        .await?;

                        (
                            Some(refund.id.to_string()),
                            refund
                                .balance_transaction
                                .and_then(|x| x.into_object())
                                .map(|x| x.net),
                        )
                    } else {
                        return Err(ApiError::InvalidInput(
                            "Charge does not have attached payment id!"
                                .to_string(),
                        ));
                    }
                }
            }
        };

        let mut transaction = pool.begin().await?;

        let charge_id = generate_charge_id(&mut transaction).await?;
        DBCharge {
            id: charge_id,
            user_id: charge.user_id,
            price_id: charge.price_id,
            amount: -refund_amount,
            currency_code: charge.currency_code,
            status: ChargeStatus::Succeeded,
            due: Utc::now(),
            last_attempt: None,
            type_: ChargeType::Refund,
            subscription_id: charge.subscription_id,
            subscription_interval: charge.subscription_interval,
            payment_platform: charge.payment_platform,
            payment_platform_id: id,
            parent_charge_id: Some(charge.id),
            net,
        }
        .upsert(&mut transaction)
        .await?;

        if body.0.unprovision.unwrap_or(false) {
            if let Some(subscription_id) = charge.subscription_id {
                let open_charge =
                    DBCharge::get_open_subscription(subscription_id, &**pool)
                        .await?;
                if let Some(mut open_charge) = open_charge {
                    open_charge.status = ChargeStatus::Cancelled;
                    open_charge.due = Utc::now();

                    open_charge.upsert(&mut transaction).await?;
                }
            }
        }

        transaction.commit().await?;
    }

    Ok(HttpResponse::NoContent().body(""))
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
        Scopes::SESSION_ACCESS,
    )
    .await?
    .1;

    let (id,) = info.into_inner();

    if let Some(subscription) =
        user_subscription_item::DBUserSubscription::get(id.into(), &**pool)
            .await?
    {
        if subscription.user_id != user.id.into() && !user.role.is_admin() {
            return Err(ApiError::NotFound);
        }

        let mut transaction = pool.begin().await?;

        let mut open_charge =
            crate::database::models::charge_item::DBCharge::get_open_subscription(
                subscription.id,
                &mut *transaction,
            )
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "Could not find open charge for this subscription".to_string(),
                )
            })?;

        let current_price = product_item::DBProductPrice::get(
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
            if !matches!(
                open_charge.status,
                ChargeStatus::Open
                    | ChargeStatus::Cancelled
                    | ChargeStatus::Failed
            ) {
                return Err(ApiError::InvalidInput(
                    "You may not change the status of this subscription!"
                        .to_string(),
                ));
            }

            if *cancelled {
                open_charge.status = ChargeStatus::Cancelled;
            } else if open_charge.status == ChargeStatus::Failed {
                // Force another resubscription attempt
                open_charge.last_attempt = Some(Utc::now() - Duration::days(2));
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
                product_item::DBProductPrice::get_all_product_prices(
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
            let duration = PriceDuration::Monthly;

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

            // First branch: Plan downgrade, update future charge
            // Second branch: For small transactions (under 30 cents), we make a loss on the
            // proration due to fees. In these situations, just give it to them for free, because
            // their next charge will be in a day or two anyway.
            if current_amount > amount || proration < 30 {
                open_charge.price_id = product_price.id;
                open_charge.amount = amount as i64;

                None
            } else {
                let charge_id = generate_charge_id(&mut transaction).await?;

                let customer_id = get_or_create_customer(
                    user.id,
                    user.stripe_customer_id.as_deref(),
                    user.email.as_deref(),
                    &stripe_client,
                    &pool,
                    &redis,
                )
                .await?;

                let currency = Currency::from_str(
                    &current_price.currency_code.to_lowercase(),
                )
                .map_err(|_| {
                    ApiError::InvalidInput("Invalid currency code".to_string())
                })?;

                let mut intent =
                    CreatePaymentIntent::new(proration as i64, currency);

                let mut metadata = HashMap::new();
                metadata.insert(
                    "modrinth_user_id".to_string(),
                    to_base62(user.id.0),
                );
                metadata.insert(
                    "modrinth_charge_id".to_string(),
                    to_base62(charge_id.0 as u64),
                );
                metadata.insert(
                    "modrinth_subscription_id".to_string(),
                    to_base62(subscription.id.0 as u64),
                );
                metadata.insert(
                    "modrinth_price_id".to_string(),
                    to_base62(product_price.id.0 as u64),
                );
                metadata.insert(
                    "modrinth_subscription_interval".to_string(),
                    open_charge
                        .subscription_interval
                        .unwrap_or(PriceDuration::Monthly)
                        .as_str()
                        .to_string(),
                );
                metadata.insert(
                    "modrinth_charge_type".to_string(),
                    ChargeType::Proration.as_str().to_string(),
                );

                intent.customer = Some(customer_id);
                intent.metadata = Some(metadata);
                intent.receipt_email = user.email.as_deref();
                intent.setup_future_usage =
                    Some(PaymentIntentSetupFutureUsage::OffSession);

                if let Some(payment_method) = &edit_subscription.payment_method
                {
                    let Ok(payment_method_id) =
                        PaymentMethodId::from_str(payment_method)
                    else {
                        return Err(ApiError::InvalidInput(
                            "Invalid payment method id".to_string(),
                        ));
                    };
                    intent.payment_method = Some(payment_method_id);
                }

                let intent =
                    stripe::PaymentIntent::create(&stripe_client, intent)
                        .await?;

                Some((proration, 0, intent))
            }
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
        Scopes::SESSION_ACCESS,
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

#[derive(Deserialize)]
pub struct ChargesQuery {
    pub user_id: Option<ariadne::ids::UserId>,
}

#[get("payments")]
pub async fn charges(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    query: web::Query<ChargesQuery>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?
    .1;

    let charges =
        crate::database::models::charge_item::DBCharge::get_from_user(
            if let Some(user_id) = query.user_id {
                if user.role.is_admin() {
                    user_id.into()
                } else {
                    return Err(ApiError::InvalidInput(
                        "You cannot see the subscriptions of other users!"
                            .to_string(),
                    ));
                }
            } else {
                user.id.into()
            },
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
                platform: x.payment_platform,
                parent_charge_id: x.parent_charge_id.map(|x| x.into()),
                net: if user.role.is_admin() { x.net } else { None },
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
        Scopes::SESSION_ACCESS,
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
        Scopes::SESSION_ACCESS,
    )
    .await?
    .1;

    let (id,) = info.into_inner();

    let Ok(payment_method_id) = PaymentMethodId::from_str(&id) else {
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

    if payment_method.customer.is_some_and(|x| x.id() == customer)
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
        Scopes::SESSION_ACCESS,
    )
    .await?
    .1;

    let (id,) = info.into_inner();

    let Ok(payment_method_id) = PaymentMethodId::from_str(&id) else {
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
        user_subscription_item::DBUserSubscription::get_all_user(
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

    if payment_method.customer.is_some_and(|x| x.id() == customer)
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
        Scopes::SESSION_ACCESS,
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
pub struct ActiveServersQuery {
    pub subscription_status: Option<SubscriptionStatus>,
}

#[get("active_servers")]
pub async fn active_servers(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<ActiveServersQuery>,
) -> Result<HttpResponse, ApiError> {
    let master_key = dotenvy::var("PYRO_API_KEY")?;

    if req
        .head()
        .headers()
        .get("X-Master-Key")
        .is_none_or(|it| it.as_bytes() != master_key.as_bytes())
    {
        return Err(ApiError::CustomAuthentication(
            "Invalid master key".to_string(),
        ));
    }

    let servers = user_subscription_item::DBUserSubscription::get_all_servers(
        query.subscription_status,
        &**pool,
    )
    .await?;

    #[derive(Serialize)]
    struct ActiveServer {
        pub user_id: ariadne::ids::UserId,
        pub server_id: String,
        pub price_id: crate::models::ids::ProductPriceId,
        pub interval: PriceDuration,
        pub region: Option<String>,
    }

    let server_ids = servers
        .into_iter()
        .filter_map(|x| {
            x.metadata.as_ref().map(|metadata| match metadata {
                SubscriptionMetadata::Pyro { id, region } => ActiveServer {
                    user_id: x.user_id.into(),
                    server_id: id.clone(),
                    price_id: x.price_id.into(),
                    interval: x.interval,
                    region: region.clone(),
                },
            })
        })
        .collect::<Vec<ActiveServer>>();

    Ok(HttpResponse::Ok().json(server_ids))
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
        server_region: Option<String>,
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
        "ES" => "EUR",
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
        "SG" => "SGD",
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
        Scopes::SESSION_ACCESS,
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

    let (price, currency_code, interval, price_id, charge_id, charge_type) =
        match payment_request.charge {
            ChargeRequestType::Existing { id } => {
                let charge =
                    crate::database::models::charge_item::DBCharge::get(
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
                    charge.type_,
                )
            }
            ChargeRequestType::New {
                product_id,
                interval,
            } => {
                let product =
                    product_item::DBProduct::get(product_id.into(), &**pool)
                        .await?
                        .ok_or_else(|| {
                            ApiError::InvalidInput(
                                "Specified product could not be found!"
                                    .to_string(),
                            )
                        })?;

                let mut product_prices =
                    product_item::DBProductPrice::get_all_product_prices(
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
                        user_subscription_item::DBUserSubscription::get_all_user(
                            user.id.into(),
                            &**pool,
                        )
                        .await?;

                        let user_products =
                            product_item::DBProductPrice::get_many(
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
                    if let Price::Recurring { .. } = price_item.prices {
                        ChargeType::Subscription
                    } else {
                        ChargeType::OneTime
                    },
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
            metadata: interval.map(|interval| {
                HashMap::from([(
                    "modrinth_subscription_interval".to_string(),
                    interval.as_str().to_string(),
                )])
            }),
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

        metadata.insert(
            "modrinth_charge_type".to_string(),
            charge_type.as_str().to_string(),
        );

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
            pub user_item: crate::database::models::user_item::DBUser,
            pub product_price_item: product_item::DBProductPrice,
            pub product_item: product_item::DBProduct,
            pub charge_item: crate::database::models::charge_item::DBCharge,
            pub user_subscription_item:
                Option<user_subscription_item::DBUserSubscription>,
            pub payment_metadata: Option<PaymentRequestMetadata>,
        }

        #[allow(clippy::too_many_arguments)]
        async fn get_payment_intent_metadata(
            payment_intent_id: PaymentIntentId,
            amount: i64,
            currency: String,
            metadata: HashMap<String, String>,
            pool: &PgPool,
            redis: &RedisPool,
            charge_status: ChargeStatus,
            transaction: &mut Transaction<'_, Postgres>,
        ) -> Result<PaymentIntentMetadata, ApiError> {
            'metadata: {
                let Some(user_id) = metadata
                    .get("modrinth_user_id")
                    .and_then(|x| parse_base62(x).ok())
                    .map(|x| crate::database::models::ids::DBUserId(x as i64))
                else {
                    break 'metadata;
                };

                let Some(user) =
                    crate::database::models::user_item::DBUser::get_id(
                        user_id, pool, redis,
                    )
                    .await?
                else {
                    break 'metadata;
                };

                let payment_metadata = metadata
                    .get("modrinth_payment_metadata")
                    .and_then(|x| serde_json::from_str(x).ok());

                let Some(charge_id) = metadata
                    .get("modrinth_charge_id")
                    .and_then(|x| parse_base62(x).ok())
                    .map(|x| {
                        crate::database::models::ids::DBChargeId(x as i64)
                    })
                else {
                    break 'metadata;
                };

                let Some(charge_type) = metadata
                    .get("modrinth_charge_type")
                    .map(|x| ChargeType::from_string(x))
                else {
                    break 'metadata;
                };

                let (charge, price, product, subscription) = if let Some(
                    mut charge,
                ) =
                    crate::database::models::charge_item::DBCharge::get(
                        charge_id, pool,
                    )
                    .await?
                {
                    let Some(price) = product_item::DBProductPrice::get(
                        charge.price_id,
                        pool,
                    )
                    .await?
                    else {
                        break 'metadata;
                    };

                    let Some(product) =
                        product_item::DBProduct::get(price.product_id, pool)
                            .await?
                    else {
                        break 'metadata;
                    };

                    charge.status = charge_status;
                    charge.last_attempt = Some(Utc::now());
                    charge.payment_platform_id =
                        Some(payment_intent_id.to_string());
                    charge.upsert(transaction).await?;

                    if let Some(subscription_id) = charge.subscription_id {
                        let Some(mut subscription) =
                            user_subscription_item::DBUserSubscription::get(
                                subscription_id,
                                pool,
                            )
                            .await?
                        else {
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
                            ChargeType::Refund => {
                                return Err(ApiError::InvalidInput(
                                    "Invalid charge type: Refund".to_string(),
                                ));
                            }
                        }

                        subscription.upsert(transaction).await?;

                        (charge, price, product, Some(subscription))
                    } else {
                        (charge, price, product, None)
                    }
                } else {
                    let Some(price_id) = metadata
                        .get("modrinth_price_id")
                        .and_then(|x| parse_base62(x).ok())
                        .map(|x| {
                            crate::database::models::ids::DBProductPriceId(
                                x as i64,
                            )
                        })
                    else {
                        break 'metadata;
                    };

                    let Some(price) =
                        product_item::DBProductPrice::get(price_id, pool)
                            .await?
                    else {
                        break 'metadata;
                    };

                    let Some(product) =
                        product_item::DBProduct::get(price.product_id, pool)
                            .await?
                    else {
                        break 'metadata;
                    };

                    let subscription = match &price.prices {
                        Price::OneTime { .. } => None,
                        Price::Recurring { intervals } => {
                            let Some(interval) = metadata
                                .get("modrinth_subscription_interval")
                                .map(|x| PriceDuration::from_string(x))
                            else {
                                break 'metadata;
                            };

                            if intervals.get(&interval).is_some() {
                                let Some(subscription_id) = metadata
                                    .get("modrinth_subscription_id")
                                    .and_then(|x| parse_base62(x).ok())
                                    .map(|x| {
                                        crate::database::models::ids::DBUserSubscriptionId(x as i64)
                                    }) else {
                                    break 'metadata;
                                };

                                let subscription = if let Some(mut subscription) = user_subscription_item::DBUserSubscription::get(subscription_id, pool).await? {
                                    subscription.status = SubscriptionStatus::Unprovisioned;
                                    subscription.price_id = price_id;
                                    subscription.interval = interval;

                                    subscription
                                } else {
                                    user_subscription_item::DBUserSubscription {
                                        id: subscription_id,
                                        user_id,
                                        price_id,
                                        interval,
                                        created: Utc::now(),
                                        status: SubscriptionStatus::Unprovisioned,
                                        metadata: None,
                                    }
                                };

                                if charge_status != ChargeStatus::Failed {
                                    subscription.upsert(transaction).await?;
                                }

                                Some(subscription)
                            } else {
                                break 'metadata;
                            }
                        }
                    };

                    let charge = DBCharge {
                        id: charge_id,
                        user_id,
                        price_id,
                        amount,
                        currency_code: currency,
                        status: charge_status,
                        due: Utc::now(),
                        last_attempt: Some(Utc::now()),
                        type_: charge_type,
                        subscription_id: subscription.as_ref().map(|x| x.id),
                        subscription_interval: subscription
                            .as_ref()
                            .map(|x| x.interval),
                        payment_platform: PaymentPlatform::Stripe,
                        payment_platform_id: Some(
                            payment_intent_id.to_string(),
                        ),
                        parent_charge_id: None,
                        net: None,
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
                        payment_intent.id,
                        payment_intent.amount,
                        payment_intent.currency.to_string().to_uppercase(),
                        payment_intent.metadata,
                        &pool,
                        &redis,
                        ChargeStatus::Succeeded,
                        &mut transaction,
                    )
                    .await?;

                    if let Some(latest_charge) = payment_intent.latest_charge {
                        let charge = stripe::Charge::retrieve(
                            &stripe_client,
                            &latest_charge.id(),
                            &["balance_transaction"],
                        )
                        .await?;

                        if let Some(balance_transaction) = charge
                            .balance_transaction
                            .and_then(|x| x.into_object())
                        {
                            metadata.charge_item.net =
                                Some(balance_transaction.net);
                            metadata
                                .charge_item
                                .upsert(&mut transaction)
                                .await?;
                        }
                    }

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
                                    as crate::database::models::ids::DBUserId,
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

                                if let Some(SubscriptionMetadata::Pyro {
                                    id,
                                    region: _,
                                }) = &subscription.metadata
                                {
                                    client
                                        .post(format!(
                                            "{}/modrinth/v0/servers/{}/unsuspend",
                                            dotenvy::var("ARCHON_URL")?,
                                            id
                                        ))
                                        .header("X-Master-Key", dotenvy::var("PYRO_API_KEY")?)
                                        .send()
                                        .await?
                                        .error_for_status()?;

                                    client
                                        .post(format!(
                                        "{}/modrinth/v0/servers/{}/reallocate",
                                        dotenvy::var("ARCHON_URL")?,
                                        id
                                    ))
                                        .header(
                                            "X-Master-Key",
                                            dotenvy::var("PYRO_API_KEY")?,
                                        )
                                        .json(&serde_json::json!({
                                            "memory_mb": ram,
                                            "cpu": cpu,
                                            "swap_mb": swap,
                                            "storage_mb": storage,
                                        }))
                                        .send()
                                        .await?
                                        .error_for_status()?;
                                } else {
                                    let (server_name, server_region, source) =
                                        if let Some(
                                            PaymentRequestMetadata::Pyro {
                                                ref server_name,
                                                ref server_region,
                                                ref source,
                                            },
                                        ) = metadata.payment_metadata
                                        {
                                            (
                                                server_name.clone(),
                                                server_region.clone(),
                                                source.clone(),
                                            )
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
                                        .post(format!(
                                            "{}/modrinth/v0/servers/create",
                                            dotenvy::var("ARCHON_URL")?,
                                        ))
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
                                            "region": server_region,
                                            "source": source,
                                            "payment_interval": metadata.charge_item.subscription_interval.map(|x| match x {
                                                PriceDuration::Monthly => 1,
                                                PriceDuration::Quarterly => 3,
                                                PriceDuration::Yearly => 12,
                                            })
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
                                                region: server_region,
                                            });
                                    }
                                }
                            }
                        }
                    }

                    if let Some(mut subscription) =
                        metadata.user_subscription_item
                    {
                        let open_charge = DBCharge::get_open_subscription(
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
                            DBCharge {
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
                                payment_platform: PaymentPlatform::Stripe,
                                payment_platform_id: None,
                                parent_charge_id: None,
                                net: None,
                            }
                            .upsert(&mut transaction)
                            .await?;
                        };

                        subscription.status = SubscriptionStatus::Provisioned;
                        subscription.upsert(&mut transaction).await?;
                    }

                    transaction.commit().await?;
                    crate::database::models::user_item::DBUser::clear_caches(
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
                        payment_intent.id,
                        payment_intent.amount,
                        payment_intent.currency.to_string().to_uppercase(),
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
                        payment_intent.id,
                        payment_intent.amount,
                        payment_intent.currency.to_string().to_uppercase(),
                        payment_intent.metadata,
                        &pool,
                        &redis,
                        ChargeStatus::Failed,
                        &mut transaction,
                    )
                    .await?;

                    if let Some(email) = metadata.user_item.email {
                        let money = rusty_money::Money::from_minor(
                            metadata.charge_item.amount as i64,
                            rusty_money::iso::find(
                                &metadata.charge_item.currency_code,
                            )
                            .unwrap_or(rusty_money::iso::USD),
                        );

                        let _ = send_email(
                            email,
                            "Payment Failed for Modrinth",
                            &format!(
                                "Our attempt to collect payment for {money} from the payment card on file was unsuccessful."
                            ),
                            "Please visit the following link below to update your payment method or contact your card provider. If the button does not work, you can copy the link and paste it into your browser.",
                            Some((
                                "Update billing settings",
                                &format!(
                                    "{}/{}",
                                    dotenvy::var("SITE_URL")?,
                                    dotenvy::var("SITE_BILLING_PATH")?
                                ),
                            )),
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

                        if customer
                            .invoice_settings
                            .is_none_or(|x| x.default_payment_method.is_none())
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
    user_id: ariadne::ids::UserId,
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

        crate::database::models::user_item::DBUser::clear_caches(
            &[(user_id.into(), None)],
            redis,
        )
        .await?;

        Ok(customer.id)
    }
}

pub async fn index_subscriptions(pool: PgPool, redis: RedisPool) {
    info!("Indexing subscriptions");

    let res = async {
        let mut transaction = pool.begin().await?;
        let mut clear_cache_users = Vec::new();

        // If an active subscription has a canceled charge OR a failed charge more than two days ago, it should be cancelled
        let all_charges = DBCharge::get_unprovision(&pool).await?;

        let mut all_subscriptions =
            user_subscription_item::DBUserSubscription::get_many(
                &all_charges
                    .iter()
                    .filter_map(|x| x.subscription_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                &pool,
            )
            .await?;
        let subscription_prices = product_item::DBProductPrice::get_many(
            &all_subscriptions
                .iter()
                .map(|x| x.price_id)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
            &pool,
        )
        .await?;
        let subscription_products = product_item::DBProduct::get_many(
            &subscription_prices
                .iter()
                .map(|x| x.product_id)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
            &pool,
        )
        .await?;
        let users = crate::database::models::DBUser::get_many_ids(
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
            let Some(subscription) = all_subscriptions
                .iter_mut()
                .find(|x| Some(x.id) == charge.subscription_id)
            else {
                continue;
            };

            if subscription.status == SubscriptionStatus::Unprovisioned {
                continue;
            }

            let Some(product_price) = subscription_prices
                .iter()
                .find(|x| x.id == subscription.price_id)
            else {
                continue;
            };

            let Some(product) = subscription_products
                .iter()
                .find(|x| x.id == product_price.product_id)
            else {
                continue;
            };

            let Some(user) =
                users.iter().find(|x| x.id == subscription.user_id)
            else {
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
                        user.id as crate::database::models::ids::DBUserId,
                    )
                    .execute(&mut *transaction)
                    .await?;

                    true
                }
                ProductMetadata::Pyro { .. } => {
                    if let Some(SubscriptionMetadata::Pyro { id, region: _ }) =
                        &subscription.metadata
                    {
                        let res = reqwest::Client::new()
                            .post(format!(
                                "{}/modrinth/v0/servers/{}/suspend",
                                dotenvy::var("ARCHON_URL")?,
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

        crate::database::models::DBUser::clear_caches(
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
        warn!("Error indexing subscriptions: {:?}", e);
    }

    info!("Done indexing subscriptions");
}

pub async fn index_billing(
    stripe_client: stripe::Client,
    pool: PgPool,
    redis: RedisPool,
) {
    info!("Indexing billing queue");
    let res = async {
        // If a charge has continuously failed for more than a month, it should be cancelled
        let charges_to_cancel = DBCharge::get_cancellable(&pool).await?;

        for mut charge in charges_to_cancel {
            charge.status = ChargeStatus::Cancelled;

            let mut transaction = pool.begin().await?;
            charge.upsert(&mut transaction).await?;
            transaction.commit().await?;
        }

        // If a charge is open and due or has been attempted more than two days ago, it should be processed
        let charges_to_do = DBCharge::get_chargeable(&pool).await?;

        let prices = product_item::DBProductPrice::get_many(
            &charges_to_do
                .iter()
                .map(|x| x.price_id)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
            &pool,
        )
        .await?;

        let users = crate::database::models::DBUser::get_many_ids(
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

        for mut charge in charges_to_do {
            let Some(product_price) =
                prices.iter().find(|x| x.id == charge.price_id)
            else {
                continue;
            };

            let Some(user) = users.iter().find(|x| x.id == charge.user_id)
            else {
                continue;
            };

            let price = match &product_price.prices {
                Price::OneTime { price } => Some(price),
                Price::Recurring { intervals } => {
                    if let Some(ref interval) = charge.subscription_interval {
                        intervals.get(interval)
                    } else {
                        warn!(
                            "Could not find subscription for charge {:?}",
                            charge.id
                        );
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

                let customer = stripe::Customer::retrieve(
                    &stripe_client,
                    &customer_id,
                    &[],
                )
                .await?;

                let Ok(currency) = Currency::from_str(
                    &product_price.currency_code.to_lowercase(),
                ) else {
                    warn!(
                        "Could not find currency for {}",
                        product_price.currency_code
                    );
                    continue;
                };

                let mut intent =
                    CreatePaymentIntent::new(*price as i64, currency);

                let mut metadata = HashMap::new();
                metadata.insert(
                    "modrinth_user_id".to_string(),
                    to_base62(charge.user_id.0 as u64),
                );
                metadata.insert(
                    "modrinth_charge_id".to_string(),
                    to_base62(charge.id.0 as u64),
                );
                metadata.insert(
                    "modrinth_charge_type".to_string(),
                    charge.type_.as_str().to_string(),
                );

                intent.metadata = Some(metadata);
                intent.customer = Some(customer.id);

                if let Some(payment_method) = customer
                    .invoice_settings
                    .and_then(|x| x.default_payment_method.map(|x| x.id()))
                {
                    intent.payment_method = Some(payment_method);
                    intent.confirm = Some(true);
                    intent.off_session =
                        Some(PaymentIntentOffSession::Exists(true));

                    charge.status = ChargeStatus::Processing;

                    if let Err(e) =
                        stripe::PaymentIntent::create(&stripe_client, intent)
                            .await
                    {
                        tracing::error!(
                            "Failed to create payment intent: {:?}",
                            e
                        );
                        charge.status = ChargeStatus::Failed;
                        charge.last_attempt = Some(Utc::now());
                    }
                } else {
                    charge.status = ChargeStatus::Failed;
                    charge.last_attempt = Some(Utc::now());
                }

                let mut transaction = pool.begin().await?;
                charge.upsert(&mut transaction).await?;
                transaction.commit().await?;
            }
        }

        Ok::<(), ApiError>(())
    }
    .await;

    if let Err(e) = res {
        warn!("Error indexing billing queue: {:?}", e);
    }

    info!("Done indexing billing queue");
}
