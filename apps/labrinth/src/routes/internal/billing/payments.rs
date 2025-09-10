use crate::database::models::charge_item::DBCharge;
use crate::database::models::{
    generate_charge_id, generate_user_subscription_id, product_item,
    products_tax_identifier_item, user_subscription_item,
};
use crate::database::redis::RedisPool;
use crate::models::ids::*;
use crate::models::v3::billing::SubscriptionStatus;
use crate::models::v3::users::User;
use crate::routes::ApiError;
use crate::util::anrok;

use ariadne::ids::base62_impl::to_base62;
use ariadne::ids::*;
use serde::Deserialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use stripe::{
    self, CreatePaymentIntent, Currency, PaymentIntentOffSession,
    PaymentIntentSetupFutureUsage, PaymentMethod, PaymentMethodId,
};

use super::{
    ChargeRequestType, ChargeType, PaymentRequestMetadata, PaymentRequestType,
    Price, PriceDuration, get_or_create_customer, infer_currency_code,
};

const DEFAULT_USER_COUNTRY: &str = "US";

pub const MODRINTH_SUBSCRIPTION_ID: &str = "modrinth_subscription_id";
pub const MODRINTH_PRICE_ID: &str = "modrinth_price_id";
pub const MODRINTH_INTERVAL: &str = "modrinth_interval";
pub const MODRINTH_CHARGE_TYPE: &str = "modrinth_charge_type";
pub const MODRINTH_NEW_REGION: &str = "modrinth_new_region";
pub const MODRINTH_USER_ID: &str = "modrinth_user_id";
pub const MODRINTH_CHARGE_ID: &str = "modrinth_charge_id";
pub const MODRINTH_TAX_AMOUNT: &str = "modrinth_tax_amount";
pub const MODRINTH_PAYMENT_METADATA: &str = "modrinth_payment_metadata";

pub enum AttachedCharge {
    /// Base the payment intent amount and tax on the product's price at this interval,
    /// but don't actually create a charge item until the payment intent is confirmed.
    ///
    /// The amount will be based on the product's price at this interval,
    /// and tax calculated based on the payment method.
    BaseUpon {
        product_id: ProductId,
        interval: Option<PriceDuration>,
    },
    /// Use an existing charge to base the payment intent upon.
    ///
    /// This can be used in the case of resubscription flows. The amount from this
    /// charge will be used, but the tax will be recalculated and the charge updated.
    ///
    /// The charge's status will NOT be updated - it is the caller's responsability to
    /// update the charge's status on failure or success.
    UseExisting { charge: DBCharge },
}

impl AttachedCharge {
    pub fn as_charge(&self) -> Option<&DBCharge> {
        if let AttachedCharge::UseExisting { charge } = self {
            Some(charge)
        } else {
            None
        }
    }

    pub async fn from_charge_request_type(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        charge_request_type: ChargeRequestType,
    ) -> Result<Self, ApiError> {
        Ok(match charge_request_type {
            ChargeRequestType::Existing { id } => AttachedCharge::UseExisting {
                charge: DBCharge::get(id.into(), exec).await?.ok_or_else(
                    || {
                        ApiError::InvalidInput(
                            "Could not find charge".to_string(),
                        )
                    },
                )?,
            },
            ChargeRequestType::New {
                product_id,
                interval,
            } => AttachedCharge::BaseUpon {
                product_id,
                interval,
            },
        })
    }
}

pub enum PaymentSession {
    Interactive {
        payment_request_type: PaymentRequestType,
    },
    AutomatedRenewal,
}

impl PaymentSession {
    pub fn set_payment_intent_off_session_options(
        &self,
        intent: &mut CreatePaymentIntent,
    ) {
        if matches!(self, PaymentSession::AutomatedRenewal) {
            intent.off_session = Some(PaymentIntentOffSession::Exists(true));
            intent.confirm = Some(true);
        } else {
            intent.off_session = None;
        }
    }
}

pub enum CurrencyMode {
    UseSpecified(Currency),
    InferFromBillingDetails,
}

pub struct PaymentBootstrapOptions<'a> {
    pub user: &'a User,
    /// Update this payment intent instead of creating a new intent.
    pub existing_payment_intent: Option<stripe::PaymentIntentId>,
    /// The status of the current payment session. This is used to derive the payment
    /// method as well as set the appropriate parameters on the payment intent.
    ///
    /// For interactive payment flows, a `PaymentRequestType` can be attached, we can be
    /// either an existing PaymentMethodId for existing payment methods, or a ConfirmationToken
    /// (ctoken) for new payment methods.
    ///
    /// For automated subscription renewal flows, use the `AutomatedRenewal` variant to
    /// select the default payment method from the Stripe customer.
    ///
    /// Taxes will always be collected.
    ///
    /// Note the charge will NOT be updated. It is the caller's responsability to update the charge
    /// on success or failure.
    pub payment_session: PaymentSession,
    /// The charge the payment intent on should be based upon.
    pub attached_charge: AttachedCharge,
    /// The currency used for the payment amount.
    pub currency_mode: CurrencyMode,
    /// Some products have additional provisioning metadata that should be attached to the payment
    /// intent.
    pub attach_payment_metadata: Option<PaymentRequestMetadata>,
}

pub struct PaymentBootstrapResults {
    pub new_payment_intent: Option<stripe::PaymentIntent>,
    pub payment_method: PaymentMethod,
    pub price_id: ProductPriceId,
    pub subtotal: i64,
    pub tax: i64,
}

/// Updates a PaymentIntent or creates a new one, recalculating tax information and
/// setting metadata fields based on the specified payment request and session options.
///
/// # Important notes
///
/// - This function does not perform any database writes. It is the caller's responsability to, for
///   example, update the charge's status on success or failure, or update the charge's tax amount,
///   tax eligibility or payment and tax platform IDs.
/// - You may not update or create a payment intent for an off-session payment flow without
///   attaching a charge.
pub async fn create_or_update_payment_intent(
    pg: &PgPool,
    redis: &RedisPool,
    stripe_client: &stripe::Client,
    anrok_client: &anrok::Client,
    PaymentBootstrapOptions {
        user,
        existing_payment_intent,
        payment_session,
        attached_charge,
        currency_mode,
        attach_payment_metadata,
    }: PaymentBootstrapOptions<'_>,
) -> Result<PaymentBootstrapResults, ApiError> {
    let customer_id = get_or_create_customer(
        user.id,
        user.stripe_customer_id.as_deref(),
        user.email.as_deref(),
        stripe_client,
        pg,
        redis,
    )
    .await?;

    let payment_method = match &payment_session {
        PaymentSession::Interactive {
            payment_request_type: PaymentRequestType::PaymentMethod { id },
        } => {
            let payment_method_id =
                PaymentMethodId::from_str(id).map_err(|_| {
                    ApiError::InvalidInput(
                        "Invalid payment method id".to_string(),
                    )
                })?;

            

            PaymentMethod::retrieve(
                stripe_client,
                &payment_method_id,
                &[],
            )
            .await?
        }
        PaymentSession::Interactive {
            payment_request_type:
                PaymentRequestType::ConfirmationToken { token },
        } => {
            #[derive(Deserialize)]
            struct ConfirmationToken {
                payment_method_preview: Option<PaymentMethod>,
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

            

            confirmation.payment_method_preview.ok_or_else(|| {
                    ApiError::InvalidInput(
                        "Confirmation token is missing payment method!"
                            .to_string(),
                    )
                })?
        }
        PaymentSession::AutomatedRenewal => {
            if attached_charge.as_charge().is_none() {
                return Err(ApiError::InvalidInput(
                    "Missing attached charge for automated renewal".to_string(),
                ));
            }

            let customer =
                stripe::Customer::retrieve(stripe_client, &customer_id, &[])
                    .await?;

            let maybe_payment_method_id = customer
                .invoice_settings
                .and_then(|x| x.default_payment_method.map(|x| x.id()));

            match maybe_payment_method_id {
                Some(payment_method_id) => {
                    

                    stripe::PaymentMethod::retrieve(
                        stripe_client,
                        &payment_method_id,
                        &[],
                    )
                    .await?
                }
                None => {
                    return Err(ApiError::InvalidInput(
                        "Customer has no default payment method!".to_string(),
                    ));
                }
            }
        }
    };

    let user_country = payment_method
        .billing_details
        .address
        .as_ref()
        .and_then(|x| x.country.as_deref())
        .unwrap_or(DEFAULT_USER_COUNTRY);

    let inferred_stripe_currency = match currency_mode {
        CurrencyMode::UseSpecified(currency) => currency,
        CurrencyMode::InferFromBillingDetails => {
            infer_currency_code(user_country)
                .to_lowercase()
                .parse::<Currency>()
                .map_err(|_| {
                    ApiError::InvalidInput("Invalid currency code".to_string())
                })?
        }
    };

    let charge_data = match attached_charge {
        AttachedCharge::UseExisting { ref charge } => ChargeData {
            amount: charge.amount,
            currency_code: charge.currency_code.clone(),
            interval: charge.subscription_interval,
            price_id: charge.price_id.into(),
            charge_type: charge.type_,
        },
        AttachedCharge::BaseUpon {
            product_id,
            interval,
        } => {
            derive_charge_data_from_product_selector(
                pg,
                user.id,
                product_id,
                interval,
                inferred_stripe_currency,
            )
            .await?
        }
    };

    // Create an ephemeral transaction to precalculate taxation amount statelessly

    let product_info =
        products_tax_identifier_item::product_info_by_product_price_id(
            charge_data.price_id.into(),
            pg,
        )
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "Missing product tax identifier for charge to continue"
                    .to_owned(),
            )
        })?;

    let address =
        payment_method
            .billing_details
            .address
            .clone()
            .ok_or_else(|| {
                ApiError::InvalidInput(
                    "Missing billing details from payment method to continue"
                        .to_owned(),
                )
            })?;

    let ephemeral_invoice = anrok_client
        .create_ephemeral_txn(&anrok::TransactionFields {
            customer_address: anrok::Address {
                line1: address.line1,
                city: address.city,
                region: address.state,
                postal_code: address.postal_code,
                country: address.country,
            },
            currency_code: charge_data.currency_code.clone(),
            accounting_date: chrono::Utc::now(),
            line_items: vec![anrok::LineItem::new(
                product_info.tax_identifier.tax_processor_id,
                charge_data.amount,
            )],
        })
        .await?;

    let tax_amount = ephemeral_invoice.tax_amount_to_collect;

    let mut metadata = HashMap::new();

    metadata.insert(MODRINTH_USER_ID.to_owned(), to_base62(user.id.0));
    metadata.insert(
        MODRINTH_CHARGE_TYPE.to_owned(),
        charge_data.charge_type.as_str().to_owned(),
    );

    if let Some(payment_metadata) = attach_payment_metadata {
        metadata.insert(
            MODRINTH_PAYMENT_METADATA.to_owned(),
            serde_json::to_string(&payment_metadata)?,
        );
    }

    if let AttachedCharge::UseExisting { charge } = attached_charge {
        metadata.insert(
            MODRINTH_CHARGE_ID.to_owned(),
            to_base62(charge.id.0 as u64),
        );

        // These are only used to post-create the charge in the stripe webhook, so
        // unset them.
        metadata.insert(MODRINTH_PRICE_ID.to_owned(), String::new());
        metadata.insert(MODRINTH_INTERVAL.to_owned(), String::new());
        metadata.insert(MODRINTH_SUBSCRIPTION_ID.to_owned(), String::new());
    } else {
        let mut transaction = pg.begin().await?;
        let charge_id = generate_charge_id(&mut transaction).await?;
        let subscription_id =
            generate_user_subscription_id(&mut transaction).await?;

        metadata.insert(
            MODRINTH_CHARGE_ID.to_owned(),
            to_base62(charge_id.0 as u64),
        );
        metadata.insert(
            MODRINTH_SUBSCRIPTION_ID.to_owned(),
            to_base62(subscription_id.0 as u64),
        );

        metadata.insert(
            MODRINTH_PRICE_ID.to_owned(),
            to_base62(charge_data.price_id.0 as u64),
        );

        if let Some(interval) = charge_data.interval {
            metadata.insert(
                MODRINTH_INTERVAL.to_owned(),
                interval.as_str().to_owned(),
            );
        }
    }

    if let Some(payment_intent_id) = existing_payment_intent {
        let mut update_payment_intent = stripe::UpdatePaymentIntent {
            amount: Some(charge_data.amount + tax_amount),
            currency: Some(inferred_stripe_currency),
            customer: Some(customer_id),
            metadata: Some(metadata),
            ..Default::default()
        };

        if let PaymentSession::Interactive {
            payment_request_type: PaymentRequestType::PaymentMethod { id },
        } = &payment_session
        {
            update_payment_intent.payment_method =
                Some(PaymentMethodId::from_str(id).map_err(|_| {
                    ApiError::InvalidInput(
                        "Invalid payment method id".to_string(),
                    )
                })?);
        }

        stripe::PaymentIntent::update(
            stripe_client,
            &payment_intent_id,
            update_payment_intent,
        )
        .await?;

        Ok(PaymentBootstrapResults {
            new_payment_intent: None,
            payment_method,
            price_id: charge_data.price_id,
            subtotal: charge_data.amount,
            tax: tax_amount,
        })
    } else {
        let mut intent = CreatePaymentIntent::new(
            charge_data.amount + tax_amount,
            inferred_stripe_currency,
        );

        intent.customer = Some(customer_id);
        intent.metadata = Some(metadata);
        intent.receipt_email = user.email.as_deref();
        intent.setup_future_usage =
            Some(PaymentIntentSetupFutureUsage::OffSession);

        if let PaymentSession::Interactive {
            payment_request_type: PaymentRequestType::PaymentMethod { id },
        } = &payment_session
        {
            intent.payment_method =
                Some(PaymentMethodId::from_str(id).map_err(|_| {
                    ApiError::InvalidInput(
                        "Invalid payment method id".to_string(),
                    )
                })?);
        }

        let payment_intent =
            stripe::PaymentIntent::create(stripe_client, intent).await?;

        Ok(PaymentBootstrapResults {
            new_payment_intent: Some(payment_intent),
            payment_method,
            price_id: charge_data.price_id,
            subtotal: charge_data.amount,
            tax: tax_amount,
        })
    }
}

struct ChargeData {
    pub amount: i64,
    pub currency_code: String,
    pub interval: Option<PriceDuration>,
    pub price_id: ProductPriceId,
    pub charge_type: ChargeType,
}

async fn derive_charge_data_from_product_selector(
    pool: &PgPool,
    user_id: UserId,
    product_id: ProductId,
    interval: Option<PriceDuration>,
    stripe_currency: Currency,
) -> Result<ChargeData, ApiError> {
    let recommended_currency_code = stripe_currency.to_string().to_uppercase();

    let product = product_item::DBProduct::get(product_id.into(), pool)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "Specified product could not be found!".to_string(),
            )
        })?;

    let mut product_prices =
        product_item::DBProductPrice::get_all_public_product_prices(
            product.id, pool,
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
            "Could not find a valid price for the user's country".to_string(),
        ));
    };

    let price = match price_item.prices {
        Price::OneTime { price } => price,
        Price::Recurring { ref intervals } => {
            let interval = interval.ok_or_else(|| {
                ApiError::InvalidInput(
                    "Could not find a valid price for the user's country"
                        .to_string(),
                )
            })?;

            *intervals.get(&interval).ok_or_else(|| {
                ApiError::InvalidInput(
                    "Could not find a valid price for the user's country"
                        .to_string(),
                )
            })?
        }
    };

    if let Price::Recurring { .. } = price_item.prices
        && product.unitary
    {
        let user_subscriptions =
            user_subscription_item::DBUserSubscription::get_all_user(
                user_id.into(),
                pool,
            )
            .await?;

        let user_products = product_item::DBProductPrice::get_many(
            &user_subscriptions
                .iter()
                .filter(|x| x.status == SubscriptionStatus::Provisioned)
                .map(|x| x.price_id)
                .collect::<Vec<_>>(),
            pool,
        )
        .await?;

        if user_products
            .into_iter()
            .any(|x| x.product_id == product.id)
        {
            return Err(ApiError::InvalidInput(
                "You are already subscribed to this product!".to_string(),
            ));
        }
    }

    Ok(ChargeData {
        amount: price as i64,
        currency_code: price_item.currency_code.clone(),
        interval,
        price_id: price_item.id.into(),
        charge_type: if let Price::Recurring { .. } = price_item.prices {
            ChargeType::Subscription
        } else {
            ChargeType::OneTime
        },
    })
}
