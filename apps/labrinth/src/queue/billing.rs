use crate::database::models::charge_item::DBCharge;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::product_item::DBProduct;
use crate::database::models::products_tax_identifier_item::DBProductsTaxIdentifier;
use crate::database::models::user_item::DBUser;
use crate::database::models::user_subscription_item::DBUserSubscription;
use crate::database::models::users_redeemals::UserRedeemal;
use crate::database::models::{DatabaseError, ids::*};
use crate::database::models::{
    product_item, user_subscription_item, users_redeemals,
};
use crate::database::redis::RedisPool;
use crate::models::billing::{
    ChargeStatus, ChargeType, PaymentPlatform, Price, PriceDuration,
    ProductMetadata, SubscriptionMetadata, SubscriptionStatus,
};
use crate::models::notifications::NotificationBody;
use crate::models::users::Badges;
use crate::models::users::User;
use crate::routes::ApiError;
use crate::routes::internal::billing::payments::*;
use crate::util::anrok;
use crate::util::archon::ArchonClient;
use crate::util::archon::{CreateServerRequest, Specs};
use ariadne::ids::base62_impl::to_base62;
use chrono::Utc;
use futures::FutureExt;
use futures::stream::{FuturesUnordered, StreamExt};
use sqlx::PgPool;
use std::collections::HashSet;
use std::str::FromStr;
use stripe::{self, Currency};
use tracing::{error, info, warn};

pub async fn index_subscriptions(
    pool: PgPool,
    redis: RedisPool,
    stripe_client: stripe::Client,
    anrok_client: anrok::Client,
) {
    info!("Indexing subscriptions");

    async fn anrok_api_operations(
        pool: PgPool,
        redis: RedisPool,
        stripe_client: stripe::Client,
        anrok_client: anrok::Client,
    ) {
        let then = std::time::Instant::now();
        let result = update_tax_amounts(
            pool.clone(),
            redis.clone(),
            anrok_client.clone(),
            stripe_client.clone(),
            100,
        )
        .await;

        if let Err(e) = result {
            warn!("Error updating tax amount on charges: {:?}", e);
        }

        let result = update_tax_transactions(
            pool,
            redis,
            anrok_client,
            stripe_client,
            100,
        )
        .await;

        if let Err(e) = result {
            warn!("Error updating tax transactions: {:?}", e);
        }

        info!(
            "Updating tax amounts and Anrok transactions took {:?}",
            then.elapsed()
        );
    }

    /// Updates charges which need to have their tax amount updated. This is done within a timer to avoid reaching
    /// Anrok API limits.
    ///
    /// The global rate limit for Anrok API operations is 10 RPS, so we run ~6 requests every second up
    /// to the specified limit of processed charges.
    async fn update_tax_amounts(
        pg: PgPool,
        redis: RedisPool,
        anrok_client: anrok::Client,
        stripe_client: stripe::Client,
        limit: i64,
    ) -> Result<(), ApiError> {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(1));
        interval
            .set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut processed_charges = 0;

        loop {
            interval.tick().await;

            let mut txn = pg.begin().await?;

            let charges = DBCharge::get_updateable_lock(&mut *txn, 6).await?;

            if charges.is_empty() {
                info!("No more charges to process");
                break Ok(());
            }

            let anrok_client_ref = anrok_client.clone();
            let stripe_client_ref = stripe_client.clone();
            let pg_ref = pg.clone();
            let redis_ref = redis.clone();

            struct ProcessedCharge {
                new_tax_amount: i64,
                product_name: String,
            }

            let mut futures = charges
                .into_iter()
                .map(|charge| {
                    let stripe_client = stripe_client_ref.clone();
                    let anrok_client = anrok_client_ref.clone();
                    let pg = pg_ref.clone();
                    let redis = redis_ref.clone();

                    let charge_clone = charge.clone();

                    let op_fut = async move {
                        let tax_id = DBProductsTaxIdentifier::get_price(
                            charge.price_id,
                            &pg,
                        )
                        .await?
                        .ok_or_else(|| {
                            DatabaseError::Database(sqlx::Error::RowNotFound)
                        })?;

                        let product =
                            DBProduct::get_price(charge.price_id, &pg)
                                .await?
                                .ok_or_else(|| {
                                DatabaseError::Database(
                                    sqlx::Error::RowNotFound,
                                )
                            })?;

                        let stripe_address = 'a: {
                            let stripe_customer_id =
                                DBUser::get_id(charge.user_id, &pg, &redis)
                                    .await?
                                    .ok_or_else(|| {
                                        ApiError::from(DatabaseError::Database(
                                            sqlx::Error::RowNotFound,
                                        ))
                                    })
                                    .and_then(|user| {
                                        user.stripe_customer_id.ok_or_else(
                                            || {
                                                ApiError::InvalidInput(
                                                "User has no Stripe customer ID"
                                                    .to_owned(),
                                            )
                                            },
                                        )
                                    })?
                                    .parse()
                                    .map_err(|_| {
                                        ApiError::InvalidInput(
                                            "User Stripe customer ID was invalid".to_owned(),
                                        )
                                    })?;

                            let customer = stripe::Customer::retrieve(
                                &stripe_client,
                                &stripe_customer_id,
                                &["invoice_settings.default_payment_method"],
                            )
                            .await?;

                            let payment_method = customer
                                .invoice_settings
                                .and_then(|x| {
                                    x.default_payment_method.and_then(|x| x.into_object())
                                })
                                .ok_or_else(|| {
                                    ApiError::InvalidInput(
                                        "Customer has no default payment method!".to_string(),
                                    )
                                })?;

                            let stripe_address = payment_method.billing_details.address;

                            // Attempt the default payment method's address first, then the customer's address.
                            match stripe_address {
                                Some(address) => break 'a address,
                                None => {
                                    warn!("PaymentMethod had no address");
                                }
                            };

                            customer.address.ok_or_else(|| {
                                ApiError::InvalidInput(
                                    "Couldn't get an address for the Stripe customer"
                                        .to_owned(),
                                )
                            })?
                        };

                        let customer_address =
                            anrok::Address::from_stripe_address(
                                &stripe_address,
                            );

                        let tax_amount = anrok_client
                            .create_ephemeral_txn(&anrok::TransactionFields {
                                customer_address,
                                currency_code: charge.currency_code.clone(),
                                accounting_time: charge.due,
                                accounting_time_zone:
                                    anrok::AccountingTimeZone::Utc,
                                line_items: vec![anrok::LineItem::new(
                                    tax_id.tax_processor_id,
                                    charge.amount,
                                )],
                            })
                            .await?
                            .tax_amount_to_collect;

                        Result::<ProcessedCharge, ApiError>::Ok(
                            ProcessedCharge {
                                new_tax_amount: tax_amount,
                                product_name: product
                                    .name
                                    .unwrap_or_else(|| "Modrinth".to_owned()),
                            },
                        )
                    };

                    op_fut.then(move |res| async move { (charge_clone, res) })
                })
                .collect::<FuturesUnordered<_>>();

            while let Some(result) = futures.next().await {
                processed_charges += 1;

                let mut charge = match result {
                    (
                        mut charge,
                        Ok(ProcessedCharge {
                            new_tax_amount,
                            product_name,
                        }),
                    ) => {
                        if new_tax_amount != charge.tax_amount {
                            // The price of the subscription has changed, we need to insert a notification
                            // for this.

                            let subscription_id =
                                charge.subscription_id.ok_or_else(|| {
                                    ApiError::InvalidInput(
                                        "Charge has no subscription ID"
                                            .to_owned(),
                                    )
                                })?;

                            NotificationBuilder {
                                body: NotificationBody::TaxNotification {
                                    subscription_id: subscription_id.into(),
                                    new_amount: charge.amount,
                                    new_tax_amount,
                                    old_amount: charge.amount,
                                    old_tax_amount: charge.tax_amount,
                                    billing_interval: charge
                                        .subscription_interval
                                        .unwrap_or(PriceDuration::Monthly),
                                    due: charge.due,
                                    service: product_name,
                                    currency: charge.currency_code.clone(),
                                },
                            }
                            .insert(charge.user_id, &mut txn, &redis)
                            .await?;

                            charge.tax_amount = new_tax_amount;
                        }

                        charge
                    }
                    (charge, Err(error)) => {
                        error!(%error, "Error indexing tax amount on charge");
                        charge
                    }
                };

                charge.tax_last_updated = Some(Utc::now());
                charge.upsert(&mut txn).await?;
            }

            txn.commit().await?;

            if processed_charges >= limit {
                break Ok(());
            }
        }
    }

    /// Registers Anrok transactions for charges which are missing a tax identifier.
    ///
    /// Same as update_tax_amounts, this is done within a timer to avoid reaching Anrok API limits.
    ///
    /// The global rate limit for Anrok API operations is 10 RPS, so we run ~6 requests every second up
    /// to the specified limit of processed charges.
    async fn update_tax_transactions(
        pg: PgPool,
        redis: RedisPool,
        anrok_client: anrok::Client,
        stripe_client: stripe::Client,
        limit: i64,
    ) -> Result<(), ApiError> {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(1));
        interval
            .set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut processed_charges = 0;

        loop {
            interval.tick().await;

            let mut txn = pg.begin().await?;

            let charges =
                DBCharge::get_missing_tax_identifier_lock(&mut *txn, 6).await?;

            if charges.is_empty() {
                info!("No more charges to process");
                break Ok(());
            }

            for mut c in charges {
                let payment_intent_id = c
                    .payment_platform_id
                    .as_deref()
                    .and_then(|x| x.parse().ok())
                    .ok_or_else(|| {
                        ApiError::InvalidInput(
                            "Charge has no or an invalid payment platform ID"
                                .to_owned(),
                        )
                    })?;

                let customer_address = 'a: {
                    let stripe_id: stripe::PaymentIntentId = c
                        .payment_platform_id
                        .as_ref()
                        .and_then(|x| x.parse().ok())
                        .ok_or_else(|| {
                            ApiError::InvalidInput(
                                "Charge has no payment platform ID".to_owned(),
                            )
                        })?;

                    // Attempt retrieving the address via the payment intent's payment method

                    let pi = stripe::PaymentIntent::retrieve(
                        &stripe_client,
                        &stripe_id,
                        &["payment_method"],
                    )
                    .await?;

                    let pi_stripe_address = pi
                        .payment_method
                        .and_then(|x| x.into_object())
                        .and_then(|x| x.billing_details.address);

                    match pi_stripe_address {
                        Some(address) => break 'a address,
                        None => {
                            warn!("PaymentMethod had no address");
                        }
                    };

                    let stripe_customer_id =
                        DBUser::get_id(c.user_id, &pg, &redis)
                            .await?
                            .ok_or_else(|| {
                                ApiError::from(DatabaseError::Database(
                                    sqlx::Error::RowNotFound,
                                ))
                            })
                            .and_then(|user| {
                                user.stripe_customer_id.ok_or_else(|| {
                                    ApiError::InvalidInput(
                                        "User has no Stripe customer ID"
                                            .to_owned(),
                                    )
                                })
                            })?;

                    let customer_id =
                        stripe_customer_id.parse().map_err(|e| {
                            ApiError::InvalidInput(format!(
                                "Charge's Stripe customer ID was invalid ({e})"
                            ))
                        })?;

                    let customer = stripe::Customer::retrieve(
                        &stripe_client,
                        &customer_id,
                        &[],
                    )
                    .await?;

                    customer.address.ok_or_else(|| {
                        ApiError::InvalidInput(
                            "Stripe customer had no address".to_owned(),
                        )
                    })?
                };

                let tax_id =
                    DBProductsTaxIdentifier::get_price(c.price_id, &pg)
                        .await?
                        .ok_or_else(|| {
                            DatabaseError::Database(sqlx::Error::RowNotFound)
                        })?;

                let tax_platform_id =
                    anrok::transaction_id_stripe_pi(&payment_intent_id);

                // Note: if the tax amount that was charged to the customer is *different* than
                // what it *should* be NOW, we will take on a loss here.

                let should_have_collected = anrok_client
                    .create_or_update_txn(&anrok::Transaction {
                        id: tax_platform_id.clone(),
                        fields: anrok::TransactionFields {
                            customer_address:
                                anrok::Address::from_stripe_address(
                                    &customer_address,
                                ),
                            currency_code: c.currency_code.clone(),
                            accounting_time: c.due,
                            accounting_time_zone:
                                anrok::AccountingTimeZone::Utc,
                            line_items: vec![
                                anrok::LineItem::new_including_tax_amount(
                                    tax_id.tax_processor_id,
                                    c.tax_amount + c.amount,
                                ),
                            ],
                        },
                    })
                    .await?
                    .tax_amount_to_collect;

                let drift = should_have_collected - c.tax_amount;

                c.tax_drift_loss = Some(drift);
                c.tax_platform_id = Some(tax_platform_id);
                c.upsert(&mut txn).await?;

                processed_charges += 1;
            }

            txn.commit().await?;

            if processed_charges >= limit {
                break Ok(());
            }
        }
    }

    let tax_charges_index_handle = tokio::spawn(anrok_api_operations(
        pool.clone(),
        redis.clone(),
        stripe_client.clone(),
        anrok_client.clone(),
    ));

    let res = async {
        let mut transaction = pool.begin().await?;
        let mut clear_cache_users = Vec::new();

        // If an active subscription has:
        // - A canceled charge due now
        // - An expiring charge due now
        // - A failed charge more than two days ago
        // It should be unprovisioned.
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
        let users = DBUser::get_many_ids(
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
                        user.id as DBUserId,
                    )
                    .execute(&mut *transaction)
                    .await?;

                    true
                }

                ProductMetadata::Pyro { .. }
                | ProductMetadata::Medal { .. } => 'server: {
                    let server_id = match &subscription.metadata {
                        Some(SubscriptionMetadata::Pyro { id, region: _ }) => {
                            id
                        }
                        Some(SubscriptionMetadata::Medal { id }) => id,
                        _ => break 'server true,
                    };

                    let res = reqwest::Client::new()
                        .post(format!(
                            "{}/modrinth/v0/servers/{}/suspend",
                            dotenvy::var("ARCHON_URL")?,
                            server_id
                        ))
                        .header("X-Master-Key", dotenvy::var("PYRO_API_KEY")?)
                        .json(&serde_json::json!({
                            "reason": if charge.status == ChargeStatus::Cancelled || charge.status == ChargeStatus::Expiring {
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

        // If an offer redeemal has been processing for over 5 minutes, it should be set pending.
        UserRedeemal::update_stuck_5_minutes(&pool).await?;

        // If an offer redeemal is pending, try processing it.
        // Try processing it.
        let pending_redeemals = UserRedeemal::get_pending(&pool, 100).await?;
        for redeemal in pending_redeemals {
            if let Err(error) =
                try_process_user_redeemal(&pool, &redis, redeemal).await
            {
                warn!(%error, "Failed to process a redeemal.")
            }
        }

        Ok::<(), ApiError>(())
    };

    if let Err(e) = res.await {
        warn!("Error indexing subscriptions: {:?}", e);
    }

    if let Err(error) = tax_charges_index_handle.await
        && error.is_panic()
    {
        std::panic::resume_unwind(error.into_panic());
    }

    info!("Done indexing subscriptions");
}

/// Attempts to process a user redeemal.
///
/// Returns `Ok` if the entry has been succesfully processed, or will not be processed.
pub async fn try_process_user_redeemal(
    pool: &PgPool,
    redis: &RedisPool,
    mut user_redeemal: UserRedeemal,
) -> Result<(), ApiError> {
    // Immediately update redeemal row
    user_redeemal.last_attempt = Some(Utc::now());
    user_redeemal.n_attempts += 1;
    user_redeemal.status = users_redeemals::Status::Processing;
    let updated = user_redeemal.update_status_if_pending(pool).await?;

    if !updated {
        return Ok(());
    }

    let user_id = user_redeemal.user_id;

    // Find the Medal product's price & metadata

    let mut medal_products =
        product_item::QueryProductWithPrices::list_by_product_type(
            pool, "medal",
        )
        .await?;

    let Some(product_item::QueryProductWithPrices {
        id: _product_id,
        metadata,
        mut prices,
        unitary: _,
        name: _,
    }) = medal_products.pop()
    else {
        return Err(ApiError::Conflict(
            "Missing Medal subscription product".to_owned(),
        ));
    };

    let ProductMetadata::Medal {
        cpu,
        ram,
        swap,
        storage,
        region,
    } = metadata
    else {
        return Err(ApiError::Conflict(
            "Missing or incorrect metadata for Medal subscription".to_owned(),
        ));
    };

    let Some(medal_price) = prices.pop() else {
        return Err(ApiError::Conflict(
            "Missing price for Medal subscription".to_owned(),
        ));
    };

    let (price_duration, price_amount) = match medal_price.prices {
        Price::OneTime { price: _ } => {
            return Err(ApiError::Conflict(
                "Unexpected metadata for Medal subscription price".to_owned(),
            ));
        }

        Price::Recurring { intervals } => {
            let Some((price_duration, price_amount)) =
                intervals.into_iter().next()
            else {
                return Err(ApiError::Conflict(
                    "Missing price interval for Medal subscription".to_owned(),
                ));
            };

            (price_duration, price_amount)
        }
    };

    let price_id = medal_price.id;

    // Get the user's username

    let user = DBUser::get_id(user_id, pool, redis)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Send the provision request to Archon. On failure, the redeemal will be "stuck" processing,
    // and moved back to pending by `index_subscriptions`.

    let archon_client = ArchonClient::from_env()?;
    let server_id = archon_client
        .create_server(&CreateServerRequest {
            user_id: to_base62(user_id.0 as u64),
            name: format!("{}'s Medal server", user.username),
            specs: Specs {
                memory_mb: ram,
                cpu,
                swap_mb: swap,
                storage_mb: storage,
            },
            source: crate::util::archon::Empty::default(),
            region,
            tags: vec!["medal".to_owned()],
        })
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
            id: server_id.to_string(),
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
        tax_amount: 0,
        tax_platform_id: None,
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
        tax_last_updated: Some(Utc::now()),
        tax_drift_loss: Some(0),
    }
    .upsert(&mut txn)
    .await?;

    // Update `users_redeemal`, mark subscription as redeemed.
    user_redeemal.status = users_redeemals::Status::Processed;
    user_redeemal.update(&mut *txn).await?;

    txn.commit().await?;

    Ok(())
}

pub async fn index_billing(
    stripe_client: stripe::Client,
    anrok_client: anrok::Client,
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

            let Ok(currency) =
                Currency::from_str(&product_price.currency_code.to_lowercase())
            else {
                warn!(
                    "Could not find currency for {}",
                    product_price.currency_code
                );
                continue;
            };

            let user = User::from_full(user.clone());

            let result = create_or_update_payment_intent(
                &pool,
                &redis,
                &stripe_client,
                &anrok_client,
                PaymentBootstrapOptions {
                    user: &user,
                    payment_intent: None,
                    payment_session: PaymentSession::AutomatedRenewal,
                    attached_charge: AttachedCharge::UseExisting {
                        charge: charge.clone(),
                    },
                    currency: CurrencyMode::Set(
                        currency,
                    ),
                    attach_payment_metadata: None,
                },
            )
            .await;

            charge.status = ChargeStatus::Processing;
            charge.last_attempt = Some(Utc::now());

            let mut failure = false;

            match result {
                Ok(PaymentBootstrapResults {
                    new_payment_intent,
                    payment_method: _,
                    price_id: _,
                    subtotal,
                    tax,
                }) => {
                    if new_payment_intent.is_some() {
                        // The PI will automatically be confirmed
                        charge.amount = subtotal;
                        charge.tax_amount = tax;
                        charge.payment_platform = PaymentPlatform::Stripe;
                    } else {
                        error!("Payment bootstrap succeeded but no payment intent was created");
                        failure = true;
                    }
                }

                Err(error) => {
                    error!(%error, "Failed to bootstrap payment for renewal");
                    failure = true;
                }
            };

            if failure {
                charge.status = ChargeStatus::Failed;
            }

            let mut transaction = pool.begin().await?;
            charge.upsert(&mut transaction).await?;
            transaction.commit().await?;
        }

        Ok::<(), ApiError>(())
    }
    .await;

    if let Err(e) = res {
        warn!("Error indexing billing queue: {:?}", e);
    }

    info!("Done indexing billing queue");
}
