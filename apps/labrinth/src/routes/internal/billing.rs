use self::payments::*;
use crate::auth::get_user_from_headers;
use crate::database::models::charge_item::DBCharge;
use crate::database::models::ids::DBUserSubscriptionId;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::products_tax_identifier_item::product_info_by_product_price_id;
use crate::database::models::users_subscriptions_affiliations::DBUsersSubscriptionsAffiliations;
use crate::database::models::users_subscriptions_credits::DBUserSubscriptionCredit;
use crate::database::models::{
    DBAffiliateCodeId, charge_item, generate_charge_id, product_item,
    user_subscription_item,
};
use crate::database::redis::RedisPool;
use crate::models::billing::{
    Charge, ChargeStatus, ChargeType, PaymentPlatform, Price, PriceDuration,
    Product, ProductMetadata, ProductPrice, SubscriptionMetadata,
    SubscriptionStatus, UserSubscription,
};
use crate::models::ids::AffiliateCodeId;
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::users::Badges;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::anrok;
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use ariadne::ids::base62_impl::{parse_base62, to_base62};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use std::str::FromStr;
use stripe::{
    CreateRefund, CreateSetupIntent, CreateSetupIntentAutomaticPaymentMethods,
    CreateSetupIntentAutomaticPaymentMethodsAllowRedirects,
    CustomerInvoiceSettings, CustomerPaymentMethodRetrieval, EventObject,
    EventType, PaymentIntentId, PaymentMethodId, SetupIntent, UpdateCustomer,
    Webhook,
};
use tracing::warn;

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
            .service(credit)
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
    let products = product_item::QueryProductWithPrices::list_purchaseable(
        &**pool, &redis,
    )
    .await?;

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
#[allow(clippy::too_many_arguments)]
pub async fn refund_charge(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    info: web::Path<(crate::models::ids::ChargeId,)>,
    body: web::Json<ChargeRefund>,
    stripe_client: web::Data<stripe::Client>,
    anrok_client: web::Data<anrok::Client>,
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
                | ChargeStatus::Succeeded => Some(x.amount + x.tax_amount),
                ChargeStatus::Failed
                | ChargeStatus::Cancelled
                | ChargeStatus::Expiring => None,
            })
            .sum::<i64>();

        let refundable = charge.amount + charge.tax_amount - refunds;

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

        let (id, net, anrok_result) = if refund_amount == 0 {
            (None, None, None)
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
                            MODRINTH_USER_ID.to_owned(),
                            to_base62(user.id.0),
                        );
                        metadata.insert(
                            MODRINTH_CHARGE_ID.to_owned(),
                            to_base62(charge.id.0 as u64),
                        );

                        let pi = stripe::PaymentIntent::retrieve(
                            &stripe_client,
                            &payment_platform_id,
                            &["payment_method"],
                        )
                        .await?;

                        let Some(billing_address) = pi
                            .payment_method
                            .and_then(|x| x.into_object())
                            .and_then(|x| x.billing_details.address)
                        else {
                            return Err(ApiError::InvalidInput(
                                "Couldn't retrieve billing address for payment method!"
                                    .to_owned(),
                            ));
                        };

                        let tax_id = product_info_by_product_price_id(
                            charge.price_id,
                            &**pool,
                        )
                        .await?
                        .ok_or_else(|| {
                            ApiError::InvalidInput(
                                "Could not find product tax info for price ID!"
                                    .to_owned(),
                            )
                        })?
                        .tax_identifier
                        .tax_processor_id;

                        let Some((
                            (
                                original_tax_platform_id,
                                original_tax_transaction_version,
                            ),
                            original_tax_platform_accounting_time,
                        )) = charge
                            .tax_platform_id
                            .clone()
                            .zip(charge.tax_transaction_version)
                            .zip(charge.tax_platform_accounting_time)
                        else {
                            return Err(ApiError::InvalidInput(
                                "Charge is missing full tax information. Please wait for the original charge to be synchronized with the tax processor."
                                    .to_owned(),
                            ));
                        };

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

                        let anrok_txn_result = anrok_client.negate_or_create_partial_negation(
                            original_tax_platform_id,
                            original_tax_transaction_version,
                            charge.amount + charge.tax_amount,
                            &anrok::Transaction {
                                id: anrok::transaction_id_stripe_pyr(&refund.id),
                                fields: anrok::TransactionFields {
                                    customer_address: anrok::Address::from_stripe_address(&billing_address),
                                    currency_code: charge.currency_code.clone(),
                                    accounting_time: original_tax_platform_accounting_time,
                                    accounting_time_zone: anrok::AccountingTimeZone::Utc,
                                    line_items: vec![anrok::LineItem::new_including_tax_amount(tax_id, -refund_amount)],
                                    customer_id: Some(format!("stripe:cust:{}", user.stripe_customer_id.unwrap_or_else(|| "unknown".to_owned()))),
                                    customer_name: Some("Customer".to_owned()),
                                }
                            }
                        ).await;

                        (
                            Some(refund.id),
                            refund
                                .balance_transaction
                                .and_then(|x| x.into_object())
                                .map(|x| x.net),
                            Some(anrok_txn_result),
                        )
                    } else {
                        return Err(ApiError::InvalidInput(
                            "Charge does not have attached payment id!"
                                .to_string(),
                        ));
                    }
                }
                PaymentPlatform::None => {
                    return Err(ApiError::InvalidInput(
                        "This charge was not processed via a payment platform."
                            .to_owned(),
                    ));
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
            tax_amount: charge.tax_amount,
            status: ChargeStatus::Succeeded,
            due: Utc::now(),
            last_attempt: None,
            type_: ChargeType::Refund,
            subscription_id: charge.subscription_id,
            subscription_interval: charge.subscription_interval,
            payment_platform: charge.payment_platform,
            tax_platform_id: id.as_ref().map(anrok::transaction_id_stripe_pyr),
            payment_platform_id: id.as_ref().map(|x| x.to_string()),
            parent_charge_id: if refund_amount != 0 {
                Some(charge.id)
            } else {
                None
            },
            net,
            currency_code: charge.currency_code,
            tax_last_updated: Some(Utc::now()),
            tax_drift_loss: Some(0),
            tax_transaction_version: None,
            tax_platform_accounting_time: None,
        }
        .upsert(&mut transaction)
        .await?;

        if body.0.unprovision.unwrap_or(false)
            && let Some(subscription_id) = charge.subscription_id
        {
            let open_charge =
                DBCharge::get_open_subscription(subscription_id, &**pool)
                    .await?;
            if let Some(mut open_charge) = open_charge {
                open_charge.status = ChargeStatus::Cancelled;
                open_charge.due = Utc::now();

                open_charge.upsert(&mut transaction).await?;
            }
        }

        transaction.commit().await?;

        if let Some(Err(error)) = anrok_result {
            if let anrok::AnrokError::Conflict(m) = &error
                && m.contains("transactionExpectedVersionMismatch")
            {
                return Err(ApiError::InvalidInput(
                    "This refund has been processed on Stripe's end, but not on the tax processor's end. The tax transaction has been modified externally since its creation. \
                    This is likely caused by a change in nexus for the customer's jurisdiction, which lead to a new tax amount paid by the seller being calculated on the transaction. \
                    Manual intervention is required to verify the tax transaction on the platform's end and update the refund's tax transaction record."
                        .to_owned(),
                ));
            } else {
                return Err(ApiError::InvalidInput(format!(
                    "This refund has been processed on Stripe's end, but not on the tax processor's end. An unexpected error occurred, preventing the refund transaction from being processed \
                    on the tax platform's end. Error: {error}"
                )));
            }
        }
    }

    Ok(HttpResponse::NoContent().finish())
}

#[post("charge/{id}/tax/reprocess")]
pub async fn reprocess_charge_tax(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    info: web::Path<(crate::models::ids::ChargeId,)>,
    anrok_client: web::Data<anrok::Client>,
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
            "You do not have permission to reprocess a tax transaction!"
                .to_string(),
        ));
    }

    let mut txn = pool.begin().await?;

    let charge_refund = charge_item::DBCharge::get(id.into(), &mut *txn)
        .await?
        .ok_or_else(|| ApiError::NotFound)?;

    let Some(parent_charge_id) = charge_refund.parent_charge_id else {
        return Err(ApiError::InvalidInput(
            "This charge does not have a parent!".to_string(),
        ));
    };

    match charge_refund.tax_platform_id {
        Some(_) => {
            return Err(ApiError::InvalidInput(
                "Refund charge already has a tax transaction ID!".to_string(),
            ));
        }
        None => {
            let charge =
                charge_item::DBCharge::get(parent_charge_id, &mut *txn)
                    .await?
                    .ok_or_else(|| ApiError::NotFound)?;

            let payment_platform_id = charge
                .payment_platform_id
                .ok_or_else(|| {
                    ApiError::Internal(eyre::eyre!(
                        "parent charge is missing a payment platform ID"
                    ))
                })?
                .parse::<stripe::PaymentIntentId>()
                .map_err(|_| {
                    ApiError::Internal(eyre::eyre!(
                        "parent charge has an invalid payment platform ID."
                    ))
                })?;

            let pi = stripe::PaymentIntent::retrieve(
                &stripe_client,
                &payment_platform_id,
                &["payment_method"],
            )
            .await?;

            let Some(billing_address) = pi
                .payment_method
                .and_then(|x| x.into_object())
                .and_then(|x| x.billing_details.address)
            else {
                return Err(ApiError::InvalidInput(
                    "Missing billing address for payment method.".to_owned(),
                ));
            };

            let tax_id =
                product_info_by_product_price_id(charge.price_id, &mut *txn)
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInput(
                            "Could not find product tax info for price ID!"
                                .to_owned(),
                        )
                    })?
                    .tax_identifier
                    .tax_processor_id;

            let Some((
                (original_tax_platform_id, original_tax_transaction_version),
                original_tax_platform_accounting_time,
            )) = charge
                .tax_platform_id
                .clone()
                .zip(charge.tax_transaction_version)
                .zip(charge.tax_platform_accounting_time)
            else {
                return Err(ApiError::InvalidInput(
                    "Charge is missing full tax information. Please wait for the original charge to be synchronized with the tax processor."
                        .to_owned(),
                ));
            };
            let refund_id =
                charge_refund.payment_platform_id.ok_or_else(|| {
                    ApiError::Internal(eyre::eyre!(
                        "Refund charge is missing a payment platform ID!"
                    ))
                })?;

            let refund_id =
                stripe::RefundId::from_str(&refund_id).map_err(|_| {
                    ApiError::Internal(eyre::eyre!("Invalid refund ID!"))
                })?;

            let anrok_txn_result = anrok_client
                .negate_or_create_partial_negation(
                    original_tax_platform_id,
                    original_tax_transaction_version,
                    charge.amount + charge.tax_amount,
                    &anrok::Transaction {
                        id: anrok::transaction_id_stripe_pyr(&refund_id),
                        fields: anrok::TransactionFields {
                            customer_address:
                                anrok::Address::from_stripe_address(
                                    &billing_address,
                                ),
                            currency_code: charge.currency_code.clone(),
                            accounting_time:
                                original_tax_platform_accounting_time,
                            accounting_time_zone:
                                anrok::AccountingTimeZone::Utc,
                            line_items: vec![
                                anrok::LineItem::new_including_tax_amount(
                                    tax_id,
                                    -charge_refund.amount,
                                ),
                            ],
                            customer_id: Some(format!(
                                "stripe:cust:{}",
                                user.stripe_customer_id
                                    .unwrap_or_else(|| "unknown".to_owned())
                            )),
                            customer_name: Some("Customer".to_owned()),
                        },
                    },
                )
                .await;

            if let Err(error) = anrok_txn_result {
                return Err(ApiError::InvalidInput(format!(
                    "There was an error processing the tax transaction: {error}. Please make sure the version has been incremented in case of an external modification."
                )));
            }
        }
    }

    txn.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct SubscriptionEdit {
    pub interval: Option<PriceDuration>,
    pub payment_method: Option<String>,
    pub cancelled: Option<bool>,
    pub region: Option<String>,
    pub product: Option<crate::models::ids::ProductId>,
}

#[derive(Deserialize)]
pub struct SubscriptionEditQuery {
    pub dry: Option<bool>,
}

#[patch("subscription/{id}")]
#[allow(clippy::too_many_arguments)]
pub async fn edit_subscription(
    req: HttpRequest,
    info: web::Path<(crate::models::ids::UserSubscriptionId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    edit_subscription: web::Json<SubscriptionEdit>,
    query: web::Query<SubscriptionEditQuery>,
    stripe_client: web::Data<stripe::Client>,
    anrok_client: web::Data<anrok::Client>,
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

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum PaymentRequirement {
        ChargedPostPromotion,
        RequiresPayment,
    }

    /// For the case of promoting an expiring charge to a full product, determine
    /// if this operation will require immediate payment or if the user can be
    /// charged only after the promotion interval ends.
    async fn promotion_payment_requirement(
        txn: &mut sqlx::PgTransaction<'_>,
        current_product_price: &product_item::DBProductPrice,
        new_product_price: &product_item::DBProductPrice,
    ) -> Result<PaymentRequirement, ApiError> {
        let new_product = product_item::DBProduct::get(
            new_product_price.product_id,
            &mut **txn,
        )
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "Could not link new product price to product.".to_owned(),
            )
        })?;
        let current_product = product_item::DBProduct::get(
            current_product_price.product_id,
            &mut **txn,
        )
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "Could not link current product price to product.".to_owned(),
            )
        })?;

        // Special case: for promoting a 'medal' subscription to 'pyro', compare the RAM. If pyro plan has:
        // - Less RAM: Charge after the promotion duration ends.
        // - More RAM: Require a payment.
        //
        // For other cases (at the time of writing, there are no other cases) require a payment.

        Ok(
            if let (
                ProductMetadata::Pyro {
                    ram: ref pyro_ram, ..
                },
                ProductMetadata::Medal {
                    ram: ref medal_ram, ..
                },
            ) = (new_product.metadata, current_product.metadata)
            {
                if pyro_ram <= medal_ram {
                    PaymentRequirement::ChargedPostPromotion
                } else {
                    PaymentRequirement::RequiresPayment
                }
            } else {
                PaymentRequirement::RequiresPayment
            },
        )
    }

    enum Proration {
        Downgrade,
        TooSmall,
        Required(i32),
    }

    /// For the case of upgrading an existing 'pyro' subscription to another subscription product,
    /// calculates the proration amount that needs to be charged.
    ///
    /// Returns the proration requirement (see [`Proration`]) and the new product price's amount.
    fn proration_amount(
        open_charge: &charge_item::DBCharge,
        subscription: &user_subscription_item::DBUserSubscription,
        current_price: &product_item::DBProductPrice,
        new_product_price: &product_item::DBProductPrice,
    ) -> Result<(Proration, i32), ApiError> {
        let interval = open_charge.due - Utc::now();
        let duration = subscription.interval;

        let current_amount = match &current_price.prices {
            Price::OneTime { price } => *price,
            Price::Recurring { intervals } => {
                *intervals.get(&duration).ok_or_else(|| {
                    ApiError::InvalidInput(
                        "Could not find a valid price for the user's duration"
                            .to_owned(),
                    )
                })?
            }
        };

        let amount = match &new_product_price.prices {
            Price::OneTime { price } => *price,
            Price::Recurring { intervals } => {
                *intervals.get(&duration).ok_or_else(|| {
                    ApiError::InvalidInput(
                        "Could not find a valid price for the user's duration"
                            .to_owned(),
                    )
                })?
            }
        };

        let complete = Decimal::from(interval.num_seconds())
            / Decimal::from(duration.duration().num_seconds());
        let proration = (Decimal::from(amount - current_amount) * complete)
            .floor()
            .to_i32()
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "Could not convert proration to i32".to_owned(),
                )
            })?;

        Ok((
            if current_amount > amount {
                Proration::Downgrade
            } else if proration < 30 {
                Proration::TooSmall
            } else {
                Proration::Required(proration)
            },
            amount,
        ))
    }

    let (id,) = info.into_inner();

    let dry = query.dry.unwrap_or_default();

    let subscription =
        user_subscription_item::DBUserSubscription::get(id.into(), &**pool)
            .await?
            .ok_or_else(|| ApiError::NotFound)?;

    if subscription.user_id != user.id.into() && !user.role.is_admin() {
        return Err(ApiError::NotFound);
    }

    let mut transaction = pool.begin().await?;

    let mut open_charge = charge_item::DBCharge::get_open_subscription(
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

    let maybe_intent_metadata = match edit_subscription.into_inner() {
        // Case of toggling cancellation when the next charge is a failed charge
        SubscriptionEdit {
            cancelled: Some(cancelled),
            ..
        } if open_charge.status == ChargeStatus::Failed => {
            if cancelled {
                DBUsersSubscriptionsAffiliations::deactivate(
                    subscription.id,
                    &mut *transaction,
                )
                .await?;
                open_charge.status = ChargeStatus::Cancelled;
            } else {
                // Forces another resubscription attempt
                open_charge.last_attempt = Some(Utc::now() - Duration::days(2));
            }

            None
        }

        // Case of toggling cancellation when the next charge is cancelled or open
        SubscriptionEdit {
            cancelled: Some(cancelled),
            ..
        } if matches!(
            open_charge.status,
            ChargeStatus::Open | ChargeStatus::Cancelled
        ) =>
        {
            open_charge.status = if cancelled {
                DBUsersSubscriptionsAffiliations::deactivate(
                    subscription.id,
                    &mut *transaction,
                )
                .await?;
                ChargeStatus::Cancelled
            } else {
                ChargeStatus::Open
            };

            None
        }

        // Case of changing the underlying product
        SubscriptionEdit {
            product: Some(product_id),
            region,
            interval,
            payment_method: Some(payment_method),
            ..
        } => {
            // Find the new product's price item based on the current currency.
            let new_product_price =
                product_item::DBProductPrice::get_all_product_prices(
                    product_id.into(),
                    &mut *transaction,
                )
                .await?
                .into_iter()
                .find(|x| x.currency_code == current_price.currency_code)
                .ok_or_else(|| {
                    ApiError::InvalidInput(
                        "Could not find a valid price for your currency code!"
                            .to_owned(),
                    )
                })?;

            // The price is the same! The request likely asked to edit the product to what it already is.
            if new_product_price.id == current_price.id {
                return Err(ApiError::InvalidInput(
                    "You cannot use the existing product when modifying a subscription! Modifications to only the billing interval aren't yet supported."
                        .to_owned(),
                ));
            }

            #[derive(Serialize)]
            struct DryResponse {
                pub requires_payment: bool,
                pub required_payment_is_proration: bool,
            }

            let currency = stripe::Currency::from_str(
                &current_price.currency_code.to_lowercase(),
            )
            .map_err(|_| {
                ApiError::InvalidInput("Invalid currency code".to_string())
            })?;

            // The next charge is an expiring charge, so we are promoting the subscription to a paid product.
            // Instead of doing a proration (since the product is likely free) we either:
            //
            // - Return a payment intent to start the subscription immediately.
            // - Upgrade the subscription to the new product and modify the upcoming expiring charge to be the
            //   first charge of the subscription.
            //
            // ..depending on the special cases defined in `promotion_payment_requirement`.
            if open_charge.status == ChargeStatus::Expiring {
                let new_region = region.ok_or_else(|| ApiError::InvalidInput("You need to specify a region when promoting an expiring charge.".to_owned()))?;
                let new_interval = interval.ok_or_else(|| ApiError::InvalidInput("You need to specify an interval when promoting an expiring charge.".to_owned()))?;

                let req = promotion_payment_requirement(
                    &mut transaction,
                    &current_price,
                    &new_product_price,
                )
                .await?;

                if dry {
                    // Note: we aren't committing the transaction here and it will be aborted.
                    // This is okay and expected, the dry flag is set and we don't want to modify anything.
                    return Ok(HttpResponse::Ok().json(&DryResponse {
                        requires_payment: req
                            == PaymentRequirement::RequiresPayment,
                        required_payment_is_proration: false,
                    }));
                }

                let payment_request_type =
                    PaymentRequestType::from_stripe_id(payment_method)
                        .ok_or_else(|| {
                            ApiError::InvalidInput(
                                "Invalid payment method ID".to_owned(),
                            )
                        })?;

                if req == PaymentRequirement::RequiresPayment {
                    let results = create_or_update_payment_intent(
                        &pool,
                        &redis,
                        &stripe_client,
                        &anrok_client,
                        PaymentBootstrapOptions {
                            user: &user,
                            payment_intent: None,
                            payment_session: PaymentSession::Interactive {
                                payment_request_type,
                            },
                            attached_charge: AttachedCharge::Promotion {
                                product_id: new_product_price.product_id.into(),
                                interval: new_interval,
                                current_subscription: subscription.id.into(),
                                new_region,
                            },
                            currency: CurrencyMode::Set(currency),
                            attach_payment_metadata: None,
                        },
                    )
                    .await?;

                    Some(results)
                } else {
                    /*
                    open_charge.status = ChargeStatus::Open;
                    open_charge.payment_platform = PaymentPlatform::Stripe;
                    open_charge.amount = new_product_price.prices.get_interval(new_interval).ok_or_else(|| ApiError::InvalidInput("Could not find a valid price for the user's duration".to_owned()))?;
                    open_charge.currency_code = new_product_price.currency_code;
                    open_charge.subscription_interval = Some(new_interval);
                    open_charge.price_id = new_product_price.id;
                    open_charge.type_ = ChargeType::Subscription;
                    */

                    None
                }
            } else {
                // The next charge is not an expiring charge: we are upgrading or downgrading the existing subscription
                // to a new product, so prorate.

                let (proration, amount) = proration_amount(
                    &open_charge,
                    &subscription,
                    &current_price,
                    &new_product_price,
                )?;

                if dry {
                    // Note: we aren't committing the transaction here and it will be aborted.
                    // This is okay and expected, the dry flag is set and we don't want to modify anything.
                    return Ok(HttpResponse::Ok().json(&DryResponse {
                        requires_payment: matches!(
                            proration,
                            Proration::Required(_)
                        ),
                        required_payment_is_proration: true,
                    }));
                }

                match proration {
                    // We should be handling the TooSmall branch differently: upgrade the subscription
                    // immediately, and still update the open charge to reflect the desired changes in
                    // product and interval.
                    //
                    // For now we however have no retry-enabled mechanism for immediately upgrade the subscription
                    // via Archon, so just don't upgrade now. This is technically a bug that was present ever
                    // since the `< 30`/`TooSmall` condition was introduced.
                    Proration::Downgrade | Proration::TooSmall => {
                        open_charge.price_id = new_product_price.id;
                        open_charge.subscription_interval =
                            interval.or(open_charge.subscription_interval);
                        open_charge.amount = amount as i64;

                        None
                    }

                    Proration::Required(proration) => {
                        let next_interval = interval
                            .or(open_charge.subscription_interval)
                            .unwrap_or(PriceDuration::Monthly);

                        let results = create_or_update_payment_intent(
                            &pool,
                            &redis,
                            &stripe_client,
                            &anrok_client,
                            PaymentBootstrapOptions {
                                user: &user,
                                payment_intent: None,
                                payment_session: PaymentSession::Interactive {
                                    payment_request_type:
                                        PaymentRequestType::PaymentMethod {
                                            id: payment_method,
                                        },
                                },
                                attached_charge: AttachedCharge::Proration {
                                    amount: proration as i64,
                                    next_product_id: new_product_price
                                        .product_id
                                        .into(),
                                    next_interval,
                                    current_subscription: subscription
                                        .id
                                        .into(),
                                },
                                currency: CurrencyMode::Set(currency),
                                attach_payment_metadata: None,
                            },
                        )
                        .await?;

                        Some(results)
                    }
                }
            }
        }

        SubscriptionEdit {
            product: None,
            region,
            interval,
            ..
        } if region.is_some() || interval.is_some() => {
            return Err(ApiError::InvalidInput(
                "It is not currently possible to only modify the region or interval of a subscription".to_owned(),
            ));
        }

        SubscriptionEdit {
            payment_method: None,
            ..
        } => {
            return Err(ApiError::InvalidInput(
                "A known payment method is required at this point to calculate tax information".to_owned(),
            ));
        }

        _ => {
            return Err(ApiError::InvalidInput(
                "Unexpected combination of fields in subscription PATCH request. Please either only specify `cancelled`, or specify `product` \
                alongside optionally specifying a `region` and `interval`. In some cases, you may be required to provide `region` and `interval`.".to_owned(),
            ));
        }
    };

    if !dry {
        // If `?dry=true`, don't actually commit the changes.
        //
        // At this point, if dry is true, we've already early-returned, except in
        // the `cancelled` branches.

        open_charge.upsert(&mut transaction).await?;
        transaction.commit().await?;
    }

    if let Some(PaymentBootstrapResults {
        new_payment_intent: Some(pi),
        subtotal,
        tax,
        payment_method: _,
        price_id: _,
    }) = maybe_intent_metadata
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "payment_intent_id": pi.id,
            "client_secret": pi.client_secret,
            "tax": tax,
            "total": subtotal + tax,
        })))
    } else {
        Ok(HttpResponse::NoContent().finish())
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
            x.metadata.as_ref().and_then(|metadata| match metadata {
                SubscriptionMetadata::Pyro { id, region } => {
                    Some(ActiveServer {
                        user_id: x.user_id.into(),
                        server_id: id.clone(),
                        price_id: x.price_id.into(),
                        interval: x.interval,
                        region: region.clone(),
                    })
                }
                SubscriptionMetadata::Medal { .. } => None,
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

impl PaymentRequestType {
    pub fn from_stripe_id(id: String) -> Option<Self> {
        let prefix = id.split_at(id.split_once('_')?.0.len() + 1).0;
        if stripe::PaymentMethodId::is_valid_prefix(prefix) {
            Some(Self::PaymentMethod { id })
        } else if prefix == "ctoken_" {
            Some(Self::ConfirmationToken { token: id })
        } else {
            None
        }
    }
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
#[serde(rename_all = "snake_case")]
pub struct PaymentRequestMetadata {
    #[serde(flatten)]
    pub kind: PaymentRequestMetadataKind,
    pub affiliate_code: Option<AffiliateCodeId>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaymentRequestMetadataKind {
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

#[post("payment")]
pub async fn initiate_payment(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    stripe_client: web::Data<stripe::Client>,
    anrok_client: web::Data<anrok::Client>,
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

    let payment_request = payment_request.into_inner();

    let results = create_or_update_payment_intent(
        &pool,
        &redis,
        &stripe_client,
        &anrok_client,
        PaymentBootstrapOptions {
            user: &user,
            payment_intent: payment_request.existing_payment_intent,
            payment_session: PaymentSession::Interactive {
                payment_request_type: payment_request.type_,
            },
            attached_charge: AttachedCharge::from_charge_request_type(
                &**pool,
                payment_request.charge,
            )
            .await?,
            currency: CurrencyMode::Infer,
            attach_payment_metadata: payment_request.metadata,
        },
    )
    .await?;

    match results.new_payment_intent {
        Some(payment_intent) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "payment_intent_id": payment_intent.id,
                "client_secret": payment_intent.client_secret,
                "price_id": to_base62(results.price_id.0 as u64),
                "tax": results.tax,
                "total": results.subtotal + results.tax,
                "payment_method": results.payment_method,
            })))
        }
        None => Ok(HttpResponse::Ok().json(serde_json::json!({
            "price_id": to_base62(results.price_id.0 as u64),
            "tax": results.tax,
            "total": results.subtotal + results.tax,
            "payment_method": results.payment_method,
        }))),
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
            pub new_region: Option<String>,
            pub next_tax_amount: i64,
        }

        #[allow(clippy::too_many_arguments)]
        async fn get_payment_intent_metadata(
            payment_intent_id: PaymentIntentId,
            payment_intent_amount: i64,
            currency: String,
            metadata: HashMap<String, String>,
            pool: &PgPool,
            redis: &RedisPool,
            charge_status: ChargeStatus,
            transaction: &mut Transaction<'_, Postgres>,
        ) -> Result<PaymentIntentMetadata, ApiError> {
            'metadata: {
                let Some(user_id) = metadata
                    .get(MODRINTH_USER_ID)
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
                    .get(MODRINTH_PAYMENT_METADATA)
                    .and_then(|x| serde_json::from_str(x).ok());

                let Some(charge_id) = metadata
                    .get(MODRINTH_CHARGE_ID)
                    .and_then(|x| parse_base62(x).ok())
                    .map(|x| {
                        crate::database::models::ids::DBChargeId(x as i64)
                    })
                else {
                    break 'metadata;
                };

                let tax_amount = metadata
                    .get(MODRINTH_TAX_AMOUNT)
                    .and_then(|x| x.parse::<i64>().ok())
                    .unwrap_or(0);

                let subtotal_amount = payment_intent_amount - tax_amount;

                let Some(charge_type) = metadata
                    .get(MODRINTH_CHARGE_TYPE)
                    .map(|x| ChargeType::from_string(x))
                else {
                    break 'metadata;
                };

                let new_region =
                    metadata.get(MODRINTH_NEW_REGION).map(String::to_owned);

                let (charge, price, product, subscription, new_region) =
                    if let Some(mut charge) =
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

                        let Some(product) = product_item::DBProduct::get(
                            price.product_id,
                            pool,
                        )
                        .await?
                        else {
                            break 'metadata;
                        };

                        charge.status = charge_status;
                        charge.last_attempt = Some(Utc::now());
                        charge.payment_platform_id =
                            Some(payment_intent_id.to_string());
                        charge.tax_amount = tax_amount;
                        charge.tax_platform_id = None;
                        charge.upsert(transaction).await?;

                        if let Some(subscription_id) = charge.subscription_id {
                            let maybe_subscription =
                            user_subscription_item::DBUserSubscription::get(
                                subscription_id,
                                pool,
                            )
                            .await?;

                            let Some(mut subscription) = maybe_subscription
                            else {
                                break 'metadata;
                            };

                            match charge.type_ {
                                ChargeType::OneTime
                                | ChargeType::Subscription => {
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
                                        "Invalid charge type: Refund"
                                            .to_string(),
                                    ));
                                }
                            }

                            subscription.upsert(transaction).await?;

                            (
                                charge,
                                price,
                                product,
                                Some(subscription),
                                new_region,
                            )
                        } else {
                            (charge, price, product, None, new_region)
                        }
                    } else {
                        let Some(price_id) = metadata
                            .get(MODRINTH_PRICE_ID)
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

                        let maybe_product = product_item::DBProduct::get(
                            price.product_id,
                            pool,
                        )
                        .await?;

                        let Some(product) = maybe_product else {
                            break 'metadata;
                        };

                        let subscription = match &price.prices {
                            Price::OneTime { .. } => None,
                            Price::Recurring { intervals } => {
                                let Some(interval) = metadata
                                    .get(MODRINTH_SUBSCRIPTION_INTERVAL)
                                    .map(|x| PriceDuration::from_string(x))
                                else {
                                    break 'metadata;
                                };

                                if intervals.get(&interval).is_some() {
                                    let Some(subscription_id) = metadata
                                    .get(MODRINTH_SUBSCRIPTION_ID)
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
                                        subscription
                                            .upsert(transaction)
                                            .await?;
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
                            amount: subtotal_amount,
                            currency_code: currency,
                            status: charge_status,
                            due: Utc::now(),
                            last_attempt: Some(Utc::now()),
                            type_: charge_type,
                            subscription_id: subscription
                                .as_ref()
                                .map(|x| x.id),
                            subscription_interval: subscription
                                .as_ref()
                                .map(|x| x.interval),
                            payment_platform: PaymentPlatform::Stripe,
                            payment_platform_id: Some(
                                payment_intent_id.to_string(),
                            ),
                            tax_amount,
                            tax_platform_id: None,
                            parent_charge_id: None,
                            net: None,
                            tax_last_updated: Some(Utc::now()),
                            tax_drift_loss: Some(0),
                            tax_transaction_version: None,
                            tax_platform_accounting_time: None,
                        };

                        if charge_status != ChargeStatus::Failed {
                            charge.upsert(transaction).await?;
                        }

                        (charge, price, product, subscription, new_region)
                    };

                return Ok(PaymentIntentMetadata {
                    user_item: user,
                    product_price_item: price,
                    product_item: product,
                    charge_item: charge,
                    user_subscription_item: subscription,
                    payment_metadata,
                    new_region,
                    next_tax_amount: tax_amount,
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
                        // A payment shouldn't be processed for Medal subscriptions.
                        ProductMetadata::Medal { .. } => {
                            warn!(
                                "A payment processed for a free subscription"
                            );
                        }

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
                            if let Some(ref mut subscription) =
                                metadata.user_subscription_item
                            {
                                let client = reqwest::Client::new();

                                if let Some(
                                    subscription_metadata @ (
                                        SubscriptionMetadata::Pyro { id, region: _ }
                                        | SubscriptionMetadata::Medal { id }
                                    ),
                                ) = &subscription.metadata
                                {
                                    let region = match subscription_metadata {
                                        SubscriptionMetadata::Pyro { region, .. } => region.to_owned(),
                                        SubscriptionMetadata::Medal { .. } => {
                                            let region = metadata.new_region.clone();

                                            if region.is_none() {
                                                return Err(ApiError::InvalidInput(
                                                    "We attempted to promote a subscription with type=medal, which requires specifying \
                                                    a new region to move the server to. However, no new region was present in the payment \
                                                    intent metadata.".to_owned()
                                                ));
                                            }

                                            region
                                        }
                                    };

                                    #[derive(Serialize)]
                                    struct ReallocateBody<'a> {
                                        memory_mb: u32,
                                        cpu: u32,
                                        swap_mb: u32,
                                        storage_mb: u32,
                                        region: Option<&'a str>,
                                        force_move: Option<bool>,
                                    }

                                    let body = ReallocateBody {
                                        memory_mb: ram,
                                        cpu,
                                        swap_mb: swap,
                                        storage_mb: storage,
                                        force_move: (region.is_some() && subscription_metadata.is_medal()).then_some(true),
                                        region: region.as_deref(),
                                    };

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
                                        .json(&body)
                                        .send()
                                        .await?
                                        .error_for_status()?;

                                    // As the subscription has been promoted, this is now a Pyro subscription.
                                    // Ensure the metadata is properly updated.
                                    subscription.metadata = Some(SubscriptionMetadata::Pyro { id: id.to_string(), region });

                                } else {
                                    let (server_name, server_region, source) =
                                        if let Some(
                                            PaymentRequestMetadataKind::Pyro {
                                                server_name,
                                                server_region,
                                                source,
                                            },
                                        ) = metadata.payment_metadata.as_ref().map(|m| &m.kind)
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
                                                PriceDuration::FiveDays => 1,
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

                        // If the next open charge is actually an expiring charge,
                        // this means the subscription was promoted from a temporary
                        // free subscription to a paid subscription.
                        //
                        // In this case, we need to modify this expiring charge to be the
                        // next charge of the subscription, turn it into a normal open charge.
                        //
                        // Otherwise, if there *is* an open charge, the subscription was upgraded
                        // and the just-processed payment was the proration charge. In this case,
                        // the existing open charge must be updated to reflect the new product's price.
                        // The subscription interval was updated above.
                        //
                        // If there are no open charges, the just-processed payment was a recurring
                        // or initial subscription charge, and we need to create the next charge.
                        if let Some(mut charge) = open_charge {
                            if charge.status == ChargeStatus::Expiring {
                                charge.status = ChargeStatus::Open;
                                charge.due = Utc::now()
                                    + subscription.interval.duration();
                                charge.payment_platform =
                                    PaymentPlatform::Stripe;
                                charge.last_attempt = None;
                                charge.subscription_interval =
                                    Some(subscription.interval);
                                charge.amount = new_price as i64;
                                charge.price_id =
                                    metadata.product_price_item.id;
                                charge.tax_last_updated = None;
                            } else {
                                // Note: do not update the due date
                                charge.subscription_interval =
                                    Some(subscription.interval);
                                charge.price_id =
                                    metadata.product_price_item.id;
                                charge.amount = new_price as i64;
                            }
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
                                tax_amount: metadata.next_tax_amount,
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
                                tax_platform_id: None,
                                tax_last_updated: Some(Utc::now()),
                                tax_drift_loss: Some(0),
                                tax_transaction_version: None,
                                tax_platform_accounting_time: None,
                            }
                            .upsert(&mut transaction)
                            .await?;

                            if let Some(affiliate_code) = metadata
                                .payment_metadata
                                .as_ref()
                                .and_then(|m| m.affiliate_code)
                            {
                                DBUsersSubscriptionsAffiliations {
                                    subscription_id: subscription.id,
                                    affiliate_code: DBAffiliateCodeId::from(
                                        affiliate_code,
                                    ),
                                    deactivated_at: None,
                                }
                                .insert(&mut *transaction)
                                .await?;
                            }
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

                    if metadata.user_item.email.is_some() {
                        let money = rusty_money::Money::from_minor(
                            metadata.charge_item.amount as i64,
                            rusty_money::iso::find(
                                &metadata.charge_item.currency_code,
                            )
                            .unwrap_or(rusty_money::iso::USD),
                        );

                        NotificationBuilder {
                            body: NotificationBody::PaymentFailed {
                                amount: money.to_string(),
                                service: if metadata
                                    .product_item
                                    .metadata
                                    .is_midas()
                                {
                                    "Modrinth+"
                                } else if metadata
                                    .product_item
                                    .metadata
                                    .is_pyro()
                                {
                                    "Modrinth Hosting"
                                } else {
                                    "a Modrinth product"
                                }
                                .to_owned(),
                            },
                        }
                        .insert(metadata.user_item.id, &mut transaction, &redis)
                        .await?;
                    }

                    transaction.commit().await?;
                }
            }
            EventType::PaymentMethodAttached => {
                if let EventObject::PaymentMethod(payment_method) =
                    event.data.object
                    && let Some(customer_id) =
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
            _ => {}
        }
    } else {
        return Err(ApiError::InvalidInput(
            "Webhook signature validation failed!".to_string(),
        ));
    }

    Ok(HttpResponse::Ok().finish())
}

#[allow(clippy::too_many_arguments)]
async fn apply_credit_many(
    transaction: &mut Transaction<'_, Postgres>,
    redis: &RedisPool,
    current_user_id: crate::database::models::ids::DBUserId,
    subscription_ids: Vec<crate::models::ids::UserSubscriptionId>,
    days: i32,
    send_email: bool,
    message: String,
) -> Result<(), ApiError> {
    let subs_ids: Vec<DBUserSubscriptionId> = subscription_ids
        .iter()
        .map(|id| DBUserSubscriptionId(id.0 as i64))
        .collect();
    let subs = user_subscription_item::DBUserSubscription::get_many(
        &subs_ids,
        &mut **transaction,
    )
    .await?;

    let provisioned_count = subs
        .iter()
        .filter(|s| s.status == SubscriptionStatus::Provisioned)
        .count();

    let mut credit_sub_ids: Vec<DBUserSubscriptionId> =
        Vec::with_capacity(provisioned_count);
    let mut credit_user_ids: Vec<crate::database::models::ids::DBUserId> =
        Vec::with_capacity(provisioned_count);
    let mut credit_creditor_ids: Vec<crate::database::models::ids::DBUserId> =
        Vec::with_capacity(provisioned_count);
    let mut credit_days: Vec<i32> = Vec::with_capacity(provisioned_count);
    let mut credit_prev_dues: Vec<chrono::DateTime<chrono::Utc>> =
        Vec::with_capacity(provisioned_count);
    let mut credit_next_dues: Vec<chrono::DateTime<chrono::Utc>> =
        Vec::with_capacity(provisioned_count);

    for subscription in subs {
        if subscription.status != SubscriptionStatus::Provisioned {
            continue;
        }

        let mut open_charge = charge_item::DBCharge::get_open_subscription(
            subscription.id,
            &mut **transaction,
        )
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(format!(
                "Could not find open charge for subscription {}",
                to_base62(subscription.id.0 as u64)
            ))
        })?;

        let previous_due = open_charge.due;
        open_charge.due = previous_due + Duration::days(days as i64);
        let next_due = open_charge.due;
        open_charge.upsert(&mut *transaction).await?;

        credit_sub_ids.push(subscription.id);
        credit_user_ids.push(subscription.user_id);
        credit_creditor_ids.push(current_user_id);
        credit_days.push(days);
        credit_prev_dues.push(previous_due);
        credit_next_dues.push(next_due);

        if send_email {
            NotificationBuilder {
                body: NotificationBody::SubscriptionCredited {
                    subscription_id: subscription.id.into(),
                    days,
                    previous_due,
                    next_due,
                    header_message: Some(message.clone()),
                },
            }
            .insert(subscription.user_id, &mut *transaction, redis)
            .await?;
        }
    }

    DBUserSubscriptionCredit::insert_many(
        &mut *transaction,
        &credit_sub_ids,
        &credit_user_ids,
        &credit_creditor_ids,
        &credit_days,
        &credit_prev_dues,
        &credit_next_dues,
    )
    .await
    .map_err(|e| ApiError::Internal(eyre::eyre!(e)))?;

    Ok(())
}

#[derive(Deserialize)]
pub struct CreditRequest {
    #[serde(flatten)]
    pub target: CreditTarget,
    pub days: i32,
    pub send_email: bool,
    pub message: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum CreditTarget {
    Subscriptions {
        subscription_ids: Vec<crate::models::ids::UserSubscriptionId>,
    },
    Nodes {
        nodes: Vec<String>,
    },
    Region {
        region: String,
    },
}

#[post("credit")]
pub async fn credit(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    archon_client: web::Data<crate::util::archon::ArchonClient>,
    body: web::Json<CreditRequest>,
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

    if !user.role.is_admin() {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to credit subscriptions!".to_string(),
        ));
    }

    let CreditRequest {
        target,
        days,
        send_email,
        message,
    } = body.into_inner();

    if days <= 0 {
        return Err(ApiError::InvalidInput(
            "Days must be greater than zero".to_string(),
        ));
    }
    let mut transaction = pool.begin().await?;

    match target {
        CreditTarget::Subscriptions { subscription_ids } => {
            if subscription_ids.is_empty() {
                return Err(ApiError::InvalidInput(
                    "You must specify at least one subscription id".to_string(),
                ));
            }
            apply_credit_many(
                &mut transaction,
                &redis,
                crate::database::models::ids::DBUserId(user.id.0 as i64),
                subscription_ids,
                days,
                send_email,
                message,
            )
            .await?;
        }
        CreditTarget::Nodes { nodes } => {
            if nodes.is_empty() {
                return Err(ApiError::InvalidInput(
                    "You must specify at least one node hostname".to_string(),
                ));
            }
            let mut server_ids: Vec<String> = Vec::new();
            for hostname in nodes {
                let ids =
                    archon_client.get_servers_by_hostname(&hostname).await?;
                server_ids.extend(ids.into_iter().map(|id| id.to_string()));
            }
            server_ids.dedup();
            let subs = user_subscription_item::DBUserSubscription::get_many_by_server_ids(
                &server_ids,
                &mut *transaction,
            )
            .await?;
            if subs.is_empty() {
                return Err(ApiError::InvalidInput(
                    "No subscriptions found for provided nodes".to_string(),
                ));
            }
            apply_credit_many(
                &mut transaction,
                &redis,
                crate::database::models::ids::DBUserId(user.id.0 as i64),
                subs.into_iter().map(|s| s.id.into()).collect(),
                days,
                send_email,
                message,
            )
            .await?;
        }
        CreditTarget::Region { region } => {
            let servers =
                archon_client.get_active_servers_by_region(&region).await?;
            let subs = user_subscription_item::DBUserSubscription::get_many_by_server_ids(
                &servers.into_iter().map(|id| id.to_string()).collect::<Vec<String>>(),
                &mut *transaction,
            )
            .await?;
            if subs.is_empty() {
                return Err(ApiError::InvalidInput(
                    "No subscriptions found for provided region".to_string(),
                ));
            }
            apply_credit_many(
                &mut transaction,
                &redis,
                crate::database::models::ids::DBUserId(user.id.0 as i64),
                subs.into_iter().map(|s| s.id.into()).collect(),
                days,
                send_email,
                message,
            )
            .await?;
        }
    }

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

pub mod payments;
