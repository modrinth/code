use crate::database::models::charge_item::DBCharge;
use crate::database::models::{
    generate_charge_id, generate_user_subscription_id, product_item,
    products_tax_identifier_item, user_subscription_item,
};
use crate::models::ids::*;
use crate::models::v3::billing::SubscriptionStatus;
use crate::models::v3::users::User;
use crate::routes::ApiError;
use crate::util::anrok;
use crate::util::error::{ApiContext as _, Context as _};
use xredis::RedisPool;

use crate::database::PgPool;
use ariadne::ids::base62_impl::to_base62;
use ariadne::ids::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use stripe::{
    self, CreateCustomer, CreatePaymentIntent, Currency, CustomerId,
    PaymentIntentOffSession, PaymentIntentSetupFutureUsage, PaymentMethod,
    PaymentMethodId,
};

use super::{
    ChargeRequestType, ChargeType, PaymentRequestMetadata, PaymentRequestType,
    Price, PriceDuration,
};

const DEFAULT_USER_COUNTRY: &str = "US";

pub const MODRINTH_SUBSCRIPTION_ID: &str = "modrinth_subscription_id";
pub const MODRINTH_PRICE_ID: &str = "modrinth_price_id";
pub const MODRINTH_SUBSCRIPTION_INTERVAL: &str =
    "modrinth_subscription_interval";
pub const MODRINTH_CHARGE_TYPE: &str = "modrinth_charge_type";
pub const MODRINTH_NEW_REGION: &str = "modrinth_new_region";
pub const MODRINTH_USER_ID: &str = "modrinth_user_id";
pub const MODRINTH_CHARGE_ID: &str = "modrinth_charge_id";
pub const MODRINTH_TAX_AMOUNT: &str = "modrinth_tax_amount";
pub const MODRINTH_PAYMENT_METADATA: &str = "modrinth_payment_metadata";

pub enum AttachedCharge {
    /// Create a proration charge.
    ///
    /// This should be accompanied by an interactive payment session.
    Proration {
        next_product_id: ProductId,
        next_interval: PriceDuration,
        current_subscription: UserSubscriptionId,
        amount: i64,
    },
    /// Create a promotion charge.
    ///
    /// This should be accompanied by an interactive payment session.
    Promotion {
        product_id: ProductId,
        interval: PriceDuration,
        current_subscription: UserSubscriptionId,
        new_region: String,
    },
    /// Base the payment intent amount and tax on the product's price at this interval,
    /// but don't actually create a charge item until the payment intent is confirmed.
    ///
    /// The amount will be based on the product's price at this interval,
    /// and tax calculated based on the payment method.
    ///
    /// This should be accompanied by an interactive payment session.
    BaseUpon {
        product_id: ProductId,
        interval: Option<PriceDuration>,
    },
    /// Use an existing charge to base the payment intent upon.
    ///
    /// This can be used in the case of resubscription flows. The amount from this
    /// charge will be used, but the tax will be recalculated and the charge updated.
    ///
    /// The charge's status will NOT be updated - it is the caller's responsibility to
    /// update the charge's status on failure or success.
    ///
    /// This may be accompanied by an automated payment session.
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
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
        charge_request_type: ChargeRequestType,
    ) -> Result<Self, ApiError> {
        Ok(match charge_request_type {
            ChargeRequestType::Existing { id } => AttachedCharge::UseExisting {
                charge: DBCharge::get(id.into(), exec)
                    .await
                    .wrap_internal_err("fetching charge from database")?
                    .wrap_request_err_with(|| {
                        "could not find charge".to_string()
                    })?,
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
    pub fn set_payment_intent_session_options(
        &self,
        intent: &mut CreatePaymentIntent,
    ) {
        if matches!(self, PaymentSession::AutomatedRenewal) {
            intent.off_session = Some(PaymentIntentOffSession::Exists(true)); // Mark as the customer isn't able to perform manual verification/isn't on-session
            intent.confirm = Some(true); // Immediately confirm the PI
        } else {
            intent.off_session = None;
            intent.setup_future_usage =
                Some(PaymentIntentSetupFutureUsage::OffSession);
        }
    }
}

pub enum CurrencyMode {
    Set(Currency),
    Infer,
}

pub struct PaymentBootstrapOptions<'a> {
    pub user: &'a User,
    /// Update this payment intent instead of creating a new intent.
    pub payment_intent: Option<stripe::PaymentIntentId>,
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
    /// Note the charge will NOT be updated. It is the caller's responsibility to update the charge
    /// on success or failure.
    pub payment_session: PaymentSession,
    /// The charge the payment intent on should be based upon.
    pub attached_charge: AttachedCharge,
    /// The currency used for the payment amount.
    pub currency: CurrencyMode,
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
/// - This function does not perform any database writes. It is the caller's responsibility to, for
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
        payment_intent: existing_payment_intent,
        payment_session,
        attached_charge,
        currency: currency_mode,
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
    .await
    .wrap_api_err("fetching or creating Stripe customer")?;

    let mut intent_uses_confirmation_token = false;

    let payment_method = match &payment_session {
        PaymentSession::Interactive {
            payment_request_type: PaymentRequestType::PaymentMethod { id },
        } => {
            let payment_method_id = PaymentMethodId::from_str(id)
                .map_err(|err| eyre::eyre!(err))
                .wrap_request_err("invalid payment method id".to_string())?;

            PaymentMethod::retrieve(stripe_client, &payment_method_id, &[])
                .await
                .wrap_failed_dependency_err(
                    "communicating with payment provider",
                )?
        }
        PaymentSession::Interactive {
            payment_request_type:
                PaymentRequestType::ConfirmationToken { token },
        } => {
            intent_uses_confirmation_token = true;

            #[derive(Deserialize)]
            struct ConfirmationToken {
                payment_method_preview: Option<PaymentMethod>,
            }

            let mut confirmation: serde_json::Value = stripe_client
                .get(&format!("confirmation_tokens/{token}"))
                .await
                .wrap_failed_dependency_err(
                    "communicating with payment provider",
                )?;

            // We patch the JSONs to support the PaymentMethod struct
            let p: json_patch::Patch = serde_json::from_value(serde_json::json!([
                { "op": "add", "path": "/payment_method_preview/id", "value": "pm_1PirTdJygY5LJFfKmPIaM1N1" },
                { "op": "add", "path": "/payment_method_preview/created", "value": 1723183475 },
                { "op": "add", "path": "/payment_method_preview/livemode", "value": false }
            ])).unwrap();
            json_patch::patch(&mut confirmation, &p).unwrap();

            let confirmation: ConfirmationToken =
                serde_json::from_value(confirmation)
                    .wrap_request_err("deserializing JSON data")?;

            confirmation
                .payment_method_preview
                .wrap_request_err_with(|| {
                    "confirmation token is missing payment method!".to_string()
                })?
        }
        PaymentSession::AutomatedRenewal => {
            if attached_charge.as_charge().is_none() {
                return Err(ApiError::Request(eyre::eyre!(
                    "Missing attached charge for automated renewal",
                )));
            }

            let customer = stripe::Customer::retrieve(
                stripe_client,
                &customer_id,
                &["invoice_settings.default_payment_method"],
            )
            .await
            .wrap_failed_dependency_err(
                "communicating with payment provider",
            )?;

            customer
                .invoice_settings
                .and_then(|x| {
                    x.default_payment_method.and_then(|x| x.into_object())
                })
                .wrap_request_err_with(|| {
                    "customer has no default payment method!".to_string()
                })?
        }
    };

    let user_country = payment_method
        .billing_details
        .address
        .as_ref()
        .and_then(|x| x.country.as_deref())
        .unwrap_or(DEFAULT_USER_COUNTRY);

    let inferred_stripe_currency = match currency_mode {
        CurrencyMode::Set(currency) => currency,
        CurrencyMode::Infer => infer_currency_code(user_country)
            .to_lowercase()
            .parse::<Currency>()
            .map_err(|err| eyre::eyre!(err))
            .wrap_request_err("invalid currency code".to_string())?,
    };

    let charge_data = match attached_charge {
        AttachedCharge::UseExisting { ref charge } => ChargeData {
            amount: charge.amount,
            currency_code: charge.currency_code.clone(),
            interval: charge.subscription_interval,
            price_id: charge.price_id.into(),
            charge_type: charge.type_,
        },
        AttachedCharge::Proration {
            amount,
            next_product_id,
            next_interval,
            current_subscription: _,
        } => {
            // Use the same data as we would use when basing the charge data on
            // a product/interval pair, except override the amount and charge type
            // to the proration values.
            //
            // Then, the tax will be based on the next product, and the metadata
            // will be inserted as is desired for proration charges, except
            // the actual payment intent amount will be the proration amount.

            let mut charge_data = derive_charge_data_from_product_selector(
                pg,
                user.id,
                next_product_id,
                Some(next_interval),
                inferred_stripe_currency,
            )
            .await
            .wrap_api_err("deriving proration charge data")?;

            charge_data.amount = amount;
            charge_data.charge_type = ChargeType::Proration;
            charge_data
        }
        AttachedCharge::Promotion {
            product_id,
            interval,
            current_subscription: _,
            new_region: _,
        } => derive_charge_data_from_product_selector(
            pg,
            user.id,
            product_id,
            Some(interval),
            inferred_stripe_currency,
        )
        .await
        .wrap_api_err("deriving promotion charge data")?,
        AttachedCharge::BaseUpon {
            product_id,
            interval,
        } => derive_charge_data_from_product_selector(
            pg,
            user.id,
            product_id,
            interval,
            inferred_stripe_currency,
        )
        .await
        .wrap_api_err("deriving base charge data")?,
    };

    // Create an ephemeral transaction to calculate the tax amount if needed

    let tax_amount = 'tax: {
        // If a charge is attached, we must use the tax amount noted on the charge
        // as the tax amount.
        //
        // Note: if we supported interactive payments of existing charges, we may
        // want to update the charge's tax amount immediately here.
        if let Some(c) = attached_charge.as_charge() {
            break 'tax c.tax_amount;
        }

        let product_info =
            products_tax_identifier_item::product_info_by_product_price_id(
                charge_data.price_id.into(),
                pg,
            )
            .await
            .wrap_internal_err("fetching product tax identifier")?
            .wrap_request_err_with(|| {
                "missing product tax identifier for charge to continue"
                    .to_owned()
            })?;

        let address = payment_method
            .billing_details
            .address
            .clone()
            .wrap_request_err("finding billing details on payment method")?;

        let ephemeral_invoice = anrok_client
            .create_ephemeral_txn(&anrok::TransactionFields {
                customer_address: anrok::Address::from_stripe_address(&address),
                currency_code: charge_data.currency_code.clone(),
                accounting_time: chrono::Utc::now(),
                accounting_time_zone: anrok::AccountingTimeZone::Utc,
                line_items: vec![anrok::LineItem::new(
                    product_info.tax_identifier.tax_processor_id,
                    charge_data.amount,
                )],
                customer_id: None,
                customer_name: None,
            })
            .await
            .wrap_internal_err("inserting database records for `create_or_update_payment_intent`")?;

        ephemeral_invoice.tax_amount_to_collect
    };

    let mut metadata = HashMap::new();

    metadata.insert(MODRINTH_USER_ID.to_owned(), to_base62(user.id.0));
    metadata.insert(
        MODRINTH_CHARGE_TYPE.to_owned(),
        charge_data.charge_type.as_str().to_owned(),
    );
    metadata.insert(MODRINTH_TAX_AMOUNT.to_owned(), tax_amount.to_string());

    if let Some(payment_metadata) = attach_payment_metadata {
        metadata.insert(
            MODRINTH_PAYMENT_METADATA.to_owned(),
            serde_json::to_string(&payment_metadata)
                .wrap_request_err("inserting payment metadata into database")?,
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
        metadata
            .insert(MODRINTH_SUBSCRIPTION_INTERVAL.to_owned(), String::new());
        metadata.insert(MODRINTH_SUBSCRIPTION_ID.to_owned(), String::new());
    } else if let AttachedCharge::Proration {
        amount: _,
        next_product_id: _,
        next_interval,
        current_subscription,
    } = attached_charge
    {
        let mut transaction = pg
            .begin()
            .await
            .wrap_internal_err("starting database transaction")?;
        let charge_id = generate_charge_id(&mut transaction)
            .await
            .wrap_internal_err("generating charge ID")?;

        metadata.insert(
            MODRINTH_CHARGE_ID.to_owned(),
            to_base62(charge_id.0 as u64),
        );

        metadata.insert(
            MODRINTH_PRICE_ID.to_owned(),
            charge_data.price_id.to_string(),
        );
        metadata.insert(
            MODRINTH_SUBSCRIPTION_INTERVAL.to_owned(),
            next_interval.as_str().to_owned(),
        );
        metadata.insert(
            MODRINTH_SUBSCRIPTION_ID.to_owned(),
            current_subscription.to_string(),
        );
    } else if let AttachedCharge::Promotion {
        product_id: _,
        interval,
        current_subscription,
        new_region,
    } = attached_charge
    {
        let mut transaction = pg
            .begin()
            .await
            .wrap_internal_err("starting database transaction")?;
        let charge_id = generate_charge_id(&mut transaction)
            .await
            .wrap_internal_err("generating charge ID")?;

        metadata.insert(
            MODRINTH_CHARGE_ID.to_owned(),
            to_base62(charge_id.0 as u64),
        );

        metadata.insert(
            MODRINTH_PRICE_ID.to_owned(),
            charge_data.price_id.to_string(),
        );
        metadata.insert(
            MODRINTH_SUBSCRIPTION_INTERVAL.to_owned(),
            interval.as_str().to_owned(),
        );
        metadata.insert(
            MODRINTH_SUBSCRIPTION_ID.to_owned(),
            current_subscription.to_string(),
        );
        metadata.insert(MODRINTH_NEW_REGION.to_owned(), new_region);
    } else {
        let mut transaction = pg
            .begin()
            .await
            .wrap_internal_err("starting database transaction")?;
        let charge_id = generate_charge_id(&mut transaction)
            .await
            .wrap_internal_err("generating charge ID")?;
        let subscription_id = generate_user_subscription_id(&mut transaction)
            .await
            .wrap_internal_err("generating charge ID")?;

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
            charge_data.price_id.to_string(),
        );

        if let Some(interval) = charge_data.interval {
            metadata.insert(
                MODRINTH_SUBSCRIPTION_INTERVAL.to_owned(),
                interval.as_str().to_owned(),
            );
        }
    }

    if let Some(payment_intent_id) = existing_payment_intent {
        let mut update_payment_intent = stripe::UpdatePaymentIntent {
            amount: Some(charge_data.amount + tax_amount),
            currency: Some(
                charge_data
                    .stripe_currency_code()
                    .wrap_api_err("converting charge currency for Stripe")?,
            ),
            customer: Some(customer_id),
            metadata: Some(metadata),
            ..Default::default()
        };

        // If the payment request type was done through a confirmation token,
        // the payment method ID is an invalid placeholder so we don't want
        // to use it.
        //
        // The PaymentIntent will be confirmed using the confirmation token
        // by the client.
        if !intent_uses_confirmation_token {
            update_payment_intent.payment_method =
                Some(payment_method.id.clone());
        }

        stripe::PaymentIntent::update(
            stripe_client,
            &payment_intent_id,
            update_payment_intent,
        )
        .await
        .wrap_failed_dependency_err(
            "updating database records for `create_or_update_payment_intent`",
        )?;

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
            charge_data
                .stripe_currency_code()
                .wrap_api_err("converting charge currency for Stripe")?,
        );

        intent.customer = Some(customer_id);
        intent.metadata = Some(metadata);
        intent.receipt_email = user.email.as_deref();
        if !intent_uses_confirmation_token {
            intent.payment_method = Some(payment_method.id.clone());
        }

        payment_session.set_payment_intent_session_options(&mut intent);

        let payment_intent =
            stripe::PaymentIntent::create(stripe_client, intent)
                .await
                .wrap_failed_dependency_err(
                    "communicating with payment provider",
                )?;

        Ok(PaymentBootstrapResults {
            new_payment_intent: Some(payment_intent),
            payment_method,
            price_id: charge_data.price_id,
            subtotal: charge_data.amount,
            tax: tax_amount,
        })
    }
}

pub async fn get_or_create_customer(
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
        metadata.insert(MODRINTH_USER_ID.to_owned(), to_base62(user_id.0));

        let customer = stripe::Customer::create(
            client,
            CreateCustomer {
                email: user_email,
                metadata: Some(metadata),
                ..Default::default()
            },
        )
        .await
        .wrap_failed_dependency_err("communicating with payment provider")?;

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
        .await
        .wrap_internal_err("querying database for `get_or_create_customer`")?;

        crate::database::models::user_item::DBUser::clear_caches(
            &[(user_id.into(), None)],
            redis,
        )
        .await
        .wrap_internal_err("clearing cached data from Redis")?;

        Ok(customer.id)
    }
}

pub fn infer_currency_code(country: &str) -> String {
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

struct ChargeData {
    pub amount: i64,
    pub currency_code: String,
    pub interval: Option<PriceDuration>,
    pub price_id: ProductPriceId,
    pub charge_type: ChargeType,
}

impl ChargeData {
    pub fn stripe_currency_code(&self) -> Result<stripe::Currency, ApiError> {
        self.currency_code
            .to_lowercase()
            .parse::<stripe::Currency>()
            .map_err(|_| ApiError::Request(eyre::eyre!(
                format!("Invalid currency code '{}': could not convert to Stripe currency", &self.currency_code)
            )))
    }
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
        .await
        .wrap_internal_err("fetching product from database")?
        .wrap_request_err_with(|| {
            "specified product could not be found!".to_string()
        })?;

    let mut product_prices =
        product_item::DBProductPrice::get_all_public_product_prices(
            product.id, pool,
        )
        .await
        .wrap_internal_err("fetching product prices from database")?;

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
        return Err(ApiError::Request(eyre::eyre!(
            "Could not find a valid price for the user's country",
        )));
    };

    let price = match price_item.prices {
        Price::OneTime { price } => price,
        Price::Recurring { ref intervals } => {
            let interval = interval.wrap_request_err_with(|| {
                "could not find a valid price for the user's country"
                    .to_string()
            })?;

            *intervals.get(&interval).wrap_request_err_with(|| {
                "could not find a valid price for the user's country"
                    .to_string()
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
            .await
            .wrap_internal_err("fetching user subscriptions from database")?;

        let user_products = product_item::DBProductPrice::get_many(
            &user_subscriptions
                .iter()
                .filter(|x| x.status == SubscriptionStatus::Provisioned)
                .map(|x| x.price_id)
                .collect::<Vec<_>>(),
            pool,
        )
        .await
        .wrap_internal_err("fetching product prices from database")?;

        if user_products
            .into_iter()
            .any(|x| x.product_id == product.id)
        {
            return Err(ApiError::Request(eyre::eyre!(
                "You are already subscribed to this product!",
            )));
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
