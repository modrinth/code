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
use crate::util::stripe::StripeContext as _;
use xredis::RedisPool;

use crate::database::PgPool;
use ariadne::ids::base62_impl::to_base62;
use ariadne::ids::*;
use std::collections::HashMap;
use std::str::FromStr;
use stripe_core::customer::{
    CreateCustomer, ListPaymentMethodsCustomer, RetrieveCustomer,
    RetrieveCustomerReturned, UpdateCustomer, UpdateCustomerInvoiceSettings,
};
use stripe_core::payment_intent::{
    CreatePaymentIntent, CreatePaymentIntentOffSession, RetrievePaymentIntent,
    UpdatePaymentIntent,
};
use stripe_payment::ConfirmationTokenId;
use stripe_payment::confirmation_token::RetrieveConfirmationToken;
use stripe_payment::payment_method::RetrievePaymentMethod;
use stripe_shared::{
    Customer, CustomerId, PaymentIntent, PaymentIntentCaptureMethod,
    PaymentIntentId, PaymentIntentSetupFutureUsage, PaymentMethod,
    PaymentMethodId,
};
use stripe_types::Currency;

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

/// Prefixes of the IDs Stripe accepts as payment method IDs.
pub const PAYMENT_METHOD_ID_PREFIXES: &[&str] =
    &["pm_", "card_", "src_", "ba_"]; // typos:ignore

/// Parses a Stripe object ID that came from a client, checking that it has one of
/// `prefixes` and only contains characters Stripe uses in IDs.
///
/// `async-stripe` accepts any string as an ID and puts IDs into request paths
/// without escaping them, so IDs from untrusted input must go through this.
pub fn parse_stripe_id<T: for<'a> From<&'a str>>(
    id: &str,
    prefixes: &[&str],
) -> Option<T> {
    let has_prefix = prefixes
        .iter()
        .any(|prefix| id.len() > prefix.len() && id.starts_with(prefix));
    let valid_chars =
        id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');

    (has_prefix && valid_chars).then(|| T::from(id))
}

/// Parses a payment method ID that came from a client.
pub fn parse_payment_method_id(id: &str) -> Option<PaymentMethodId> {
    parse_stripe_id(id, PAYMENT_METHOD_ID_PREFIXES)
}

/// Parses an ISO 4217 currency code, returning `None` if Stripe doesn't support it.
pub fn parse_currency(code: &str) -> Option<Currency> {
    let Ok(currency) = code.to_lowercase().parse::<Currency>();
    (!matches!(currency, Currency::Unknown(_))).then_some(currency)
}

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
        intent: CreatePaymentIntent,
    ) -> CreatePaymentIntent {
        if matches!(self, PaymentSession::AutomatedRenewal) {
            // The customer isn't on-session to perform manual verification, so
            // confirm the payment intent immediately.
            intent
                .off_session(CreatePaymentIntentOffSession::Bool(true))
                .confirm(true)
        } else {
            intent.setup_future_usage(PaymentIntentSetupFutureUsage::OffSession)
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
    pub payment_intent: Option<PaymentIntentId>,
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
    pub new_payment_intent: Option<PaymentIntent>,
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

    if let Some(payment_intent_id) = &existing_payment_intent {
        ensure_payment_intent_owned_by(
            stripe_client,
            payment_intent_id,
            &customer_id,
        )
        .await?;
    }

    let mut intent_uses_confirmation_token = false;

    let payment_method = match &payment_session {
        PaymentSession::Interactive {
            payment_request_type: PaymentRequestType::PaymentMethod { id },
        } => {
            let payment_method_id = parse_payment_method_id(id)
                .wrap_request_err("invalid payment method id".to_string())?;

            RetrievePaymentMethod::new(payment_method_id)
                .send(stripe_client)
                .await
                .wrap_stripe_err("communicating with payment provider")?
        }
        PaymentSession::Interactive {
            payment_request_type:
                PaymentRequestType::ConfirmationToken { token },
        } => {
            intent_uses_confirmation_token = true;

            let token: ConfirmationTokenId =
                parse_stripe_id(token, &["ctoken_"]).wrap_request_err(
                    "invalid confirmation token".to_string(),
                )?;

            let confirmation = RetrieveConfirmationToken::new(token)
                .send(stripe_client)
                .await
                .wrap_stripe_err("communicating with payment provider")?;

            let preview = confirmation
                .payment_method_preview
                .wrap_request_err_with(|| {
                    "confirmation token is missing payment method".to_string()
                })?;

            payment_method_from_preview(&preview)?
        }
        PaymentSession::AutomatedRenewal => {
            if attached_charge.as_charge().is_none() {
                return Err(ApiError::Request(eyre::eyre!(
                    "Missing attached charge for automated renewal",
                )));
            }

            get_default_payment_method(stripe_client, &customer_id)
                .await?
                .wrap_request_err_with(|| {
                    "customer has no default payment method".to_string()
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
        CurrencyMode::Infer => {
            parse_currency(&infer_currency_code(user_country))
                .wrap_request_err("invalid currency code".to_string())?
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
        let mut update_payment_intent =
            UpdatePaymentIntent::new(payment_intent_id)
                .amount(charge_data.amount + tax_amount)
                .currency(
                    charge_data.stripe_currency_code().wrap_api_err(
                        "converting charge currency for Stripe",
                    )?,
                )
                .customer(customer_id.as_str())
                .metadata(metadata);

        // If the payment request type was done through a confirmation token,
        // the payment method ID is an invalid placeholder so we don't want
        // to use it.
        //
        // The PaymentIntent will be confirmed using the confirmation token
        // by the client.
        if !intent_uses_confirmation_token {
            update_payment_intent = update_payment_intent
                .payment_method(payment_method.id.as_str());
        }

        update_payment_intent
            .send(stripe_client)
            .await
            .wrap_stripe_err(
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
        // Stripe captures asynchronously by default since API version 2024-04-10,
        // which leaves the charge without a balance transaction when the
        // `payment_intent.succeeded` webhook is handled.
        let mut intent = CreatePaymentIntent::new(
            charge_data.amount + tax_amount,
            charge_data
                .stripe_currency_code()
                .wrap_api_err("converting charge currency for Stripe")?,
        )
        .capture_method(PaymentIntentCaptureMethod::Automatic)
        .customer(customer_id.as_str())
        .metadata(metadata);

        if let Some(email) = user.email.as_deref() {
            intent = intent.receipt_email(email);
        }
        if !intent_uses_confirmation_token {
            intent = intent.payment_method(payment_method.id.as_str());
        }

        let payment_intent = payment_session
            .set_payment_intent_session_options(intent)
            .send(stripe_client)
            .await
            .wrap_stripe_err("communicating with payment provider")?;

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
        stripe_customer_id.and_then(|x| CustomerId::from_str(x).ok())
    {
        Ok(customer_id)
    } else {
        let mut metadata = HashMap::new();
        metadata.insert(MODRINTH_USER_ID.to_owned(), to_base62(user_id.0));

        let mut create_customer = CreateCustomer::new().metadata(metadata);
        if let Some(email) = user_email {
            create_customer = create_customer.email(email);
        }

        let customer = create_customer
            .send(client)
            .await
            .wrap_stripe_err("communicating with payment provider")?;

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

/// Builds a [`PaymentMethod`] from a confirmation token's payment method preview.
///
/// The preview has the same shape as a payment method, minus the fields that only
/// exist once the payment method is created. Those are filled with placeholders, so
/// the returned payment method's ID must not be sent to Stripe.
fn payment_method_from_preview(
    preview: &stripe_payment::ConfirmationTokensResourcePaymentMethodPreview,
) -> Result<PaymentMethod, ApiError> {
    let mut value = serde_json::to_value(preview)
        .wrap_internal_err("serializing payment method preview")?;
    let object = value
        .as_object_mut()
        .wrap_internal_err("payment method preview is not an object")?;
    object.insert("id".to_owned(), "pm_1PirTdJygY5LJFfKmPIaM1N1".into());
    object.insert("created".to_owned(), 1723183475.into());
    object.insert("livemode".to_owned(), false.into());

    serde_json::from_value(value)
        .wrap_internal_err("converting payment method preview")
}

/// Checks that a payment intent exists and belongs to the Stripe customer.
///
/// A missing payment intent and one belonging to another customer return the same
/// error, so callers can't use this to probe for payment intent IDs.
async fn ensure_payment_intent_owned_by(
    client: &stripe::Client,
    payment_intent_id: &PaymentIntentId,
    customer_id: &CustomerId,
) -> Result<(), ApiError> {
    let not_found =
        || ApiError::NotFound(eyre::eyre!("payment intent not found"));

    let payment_intent =
        match RetrievePaymentIntent::new(payment_intent_id.clone())
            .send(client)
            .await
        {
            Ok(payment_intent) => payment_intent,
            Err(stripe::StripeError::Stripe(_, 404)) => return Err(not_found()),
            Err(err) => {
                return Err(err)
                    .wrap_stripe_err("communicating with payment provider");
            }
        };

    if payment_intent
        .customer
        .as_ref()
        .is_some_and(|customer| customer.id() == customer_id)
    {
        Ok(())
    } else {
        Err(not_found())
    }
}

/// Fetches a Stripe customer, failing if the customer has been deleted.
pub async fn retrieve_customer(
    client: &stripe::Client,
    customer_id: &CustomerId,
    expand: &[&str],
) -> Result<Customer, ApiError> {
    let mut request = RetrieveCustomer::new(customer_id.clone());
    if !expand.is_empty() {
        request = request
            .expand(expand.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>());
    }

    let customer = request
        .send(client)
        .await
        .wrap_stripe_err("communicating with payment provider")?;

    match customer {
        RetrieveCustomerReturned::Customer(customer) => Ok(customer),
        RetrieveCustomerReturned::DeletedCustomer(_) => Err(ApiError::Request(
            eyre::eyre!("Stripe customer has been deleted"),
        )),
    }
}

/// Lists up to `limit` payment methods attached to the customer.
pub async fn list_payment_methods(
    client: &stripe::Client,
    customer_id: &CustomerId,
    limit: i64,
) -> Result<Vec<PaymentMethod>, ApiError> {
    Ok(ListPaymentMethodsCustomer::new(customer_id.clone())
        .limit(limit)
        .send(client)
        .await
        .wrap_stripe_err("communicating with payment provider")?
        .data)
}

/// Sets the customer's default payment method, which is charged for automated renewals.
pub async fn set_default_payment_method(
    client: &stripe::Client,
    customer_id: &CustomerId,
    payment_method_id: &PaymentMethodId,
) -> Result<(), ApiError> {
    let mut invoice_settings = UpdateCustomerInvoiceSettings::new();
    invoice_settings.default_payment_method =
        Some(payment_method_id.to_string());

    UpdateCustomer::new(customer_id.clone())
        .invoice_settings(invoice_settings)
        .send(client)
        .await
        .wrap_stripe_err("communicating with payment provider")?;

    Ok(())
}

/// Fetches the customer's default payment method.
///
/// If the customer has no default payment method but has exactly one payment method
/// attached, that payment method is set as the default and returned.
pub async fn get_default_payment_method(
    client: &stripe::Client,
    customer_id: &CustomerId,
) -> Result<Option<PaymentMethod>, ApiError> {
    let customer = retrieve_customer(
        client,
        customer_id,
        &["invoice_settings.default_payment_method"],
    )
    .await?;

    if let Some(payment_method) = customer
        .invoice_settings
        .and_then(|x| x.default_payment_method)
        .and_then(|x| x.into_object())
    {
        return Ok(Some(payment_method));
    }

    let payment_methods = list_payment_methods(client, customer_id, 2).await?;

    let Ok([payment_method]) = <[PaymentMethod; 1]>::try_from(payment_methods)
    else {
        return Ok(None);
    };

    set_default_payment_method(client, customer_id, &payment_method.id).await?;

    Ok(Some(payment_method))
}

pub struct ChargeTaxQuote {
    pub tax_amount: i64,
    /// The default payment method which will be charged for this charge.
    pub payment_method: PaymentMethod,
}

/// Calculates the current tax amount for an existing charge, based on the address of the
/// customer's default payment method, falling back to the customer's address.
///
/// This does not update the charge.
pub async fn quote_charge_tax(
    pg: &PgPool,
    redis: &RedisPool,
    stripe_client: &stripe::Client,
    anrok_client: &anrok::Client,
    charge: &DBCharge,
) -> Result<ChargeTaxQuote, ApiError> {
    let tax_id =
        products_tax_identifier_item::DBProductsTaxIdentifier::get_price(
            charge.price_id,
            pg,
        )
        .await
        .wrap_api_err("fetching price")?
        .wrap_internal_err("finding product tax identifier for price")?;

    let user = crate::database::models::user_item::DBUser::get_id(
        charge.user_id,
        pg,
        redis,
    )
    .await
    .wrap_internal_err("fetching Stripe customer from database")?
    .wrap_internal_err("finding Stripe customer in database")?;
    let stripe_customer_id: CustomerId = user
        .stripe_customer_id
        .wrap_request_err("finding Stripe customer ID on user")?
        .parse()
        .wrap_request_err("parsing user Stripe customer ID")?;

    let payment_method =
        get_default_payment_method(stripe_client, &stripe_customer_id)
            .await?
            .wrap_request_err_with(|| {
                "customer has no default payment method".to_string()
            })?;

    let stripe_address = match &payment_method.billing_details.address {
        Some(address) => address.clone(),
        None => {
            tracing::warn!("PaymentMethod had no address");

            retrieve_customer(stripe_client, &stripe_customer_id, &[])
                .await?
                .address
                .wrap_request_err_with(|| {
                    "couldn't get an address for the Stripe customer".to_owned()
                })?
        }
    };

    let tax_amount = anrok_client
        .create_ephemeral_txn(&anrok::TransactionFields {
            customer_address: anrok::Address::from_stripe_address(
                &stripe_address,
            ),
            currency_code: charge.currency_code.clone(),
            accounting_time: charge.due.max(chrono::Utc::now()),
            accounting_time_zone: anrok::AccountingTimeZone::Utc,
            line_items: vec![anrok::LineItem::new(
                tax_id.tax_processor_id,
                charge.amount,
            )],
            customer_id: None,
            customer_name: None,
        })
        .await
        .wrap_internal_err("calculating tax amount for charge")?
        .tax_amount_to_collect;

    Ok(ChargeTaxQuote {
        tax_amount,
        payment_method,
    })
}

/// Makes the customer's newest payment method the default if the customer has no
/// default payment method left after `removed` was detached.
pub async fn promote_default_payment_method(
    client: &stripe::Client,
    customer_id: &CustomerId,
    removed: &PaymentMethodId,
) -> Result<(), ApiError> {
    let customer = retrieve_customer(client, customer_id, &[]).await?;

    if customer
        .invoice_settings
        .and_then(|x| x.default_payment_method)
        .is_some_and(|x| x.id() != removed)
    {
        return Ok(());
    }

    let payment_methods = list_payment_methods(client, customer_id, 1).await?;

    if let Some(payment_method) = payment_methods.first() {
        set_default_payment_method(client, customer_id, &payment_method.id)
            .await?;
    }

    Ok(())
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
    pub fn stripe_currency_code(&self) -> Result<Currency, ApiError> {
        parse_currency(&self.currency_code)
            .ok_or_else(|| ApiError::Request(eyre::eyre!(
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
