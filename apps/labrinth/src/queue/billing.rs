use crate::database::models::charge_item::{self, CustomerCharge, DBCharge};
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::user_item::DBUser;
use crate::database::models::user_subscription_item::{
    DBUserSubscription, fetch_update_lock_pending_taxation_notification,
};
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

    let tax_charges_index_handle =
        tokio::spawn(roll_index_tax_amount_on_charges(
            pool.clone(),
            anrok_client,
            stripe_client,
            150,
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

    if let Err(error) = tax_charges_index_handle.await {
        if error.is_panic() {
            std::panic::resume_unwind(error.into_panic());
        }
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
        user_aware_of_tax_changes: true,
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

            let skip_tax_collection = if let Some(subscription_id) = charge.subscription_id {
                let subs = DBUserSubscription::get(subscription_id, &pool).await?.ok_or_else(|| DatabaseError::Database(sqlx::Error::RowNotFound))?;

                !subs.user_aware_of_tax_changes
            } else {
                false
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
                    skip_tax_collection,
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

        index_tax_notifications(pool, redis).await?;

        Ok::<(), ApiError>(())
    }
    .await;

    if let Err(e) = res {
        warn!("Error indexing billing queue: {:?}", e);
    }

    info!("Done indexing billing queue");
}

/// Updates charges which are missing an amount of tax. This is done within a timer to avoid reaching
/// Anrok API limits.
///
/// The global rate limit for Anrok API operations is 10 RPS, so we run ~5 requests every second up
/// to the specified limit of processed charges.
async fn roll_index_tax_amount_on_charges(
    pg: PgPool,
    anrok_client: anrok::Client,
    stripe_client: stripe::Client,
    limit: i64,
) -> Result<(), ApiError> {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut processed_charges = 0;

    loop {
        interval.tick().await;

        let mut txn = pg.begin().await?;

        let charges =
            charge_item::get_missing_tax_with_limit(&mut *txn, 5).await?;

        let anrok_client = anrok_client.clone();
        let stripe_client = stripe_client.clone();

        let mut futures = charges
            .into_iter()
            .map(move |customer_charge| {
                let CustomerCharge {
                    mut charge,
                    stripe_customer_id,
                    product_tax_id,
                } = customer_charge;

                let stripe_client = stripe_client.clone();
                let anrok_client = anrok_client.clone();

                async move {
                    let Ok(customer_id): Result<stripe::CustomerId, _> =
                        stripe_customer_id.parse()
                    else {
                        return Err(ApiError::InvalidInput(
                            "Charge's Stripe customer ID was invalid"
                                .to_owned(),
                        ));
                    };

                    let customer = stripe::Customer::retrieve(
                        &stripe_client,
                        &customer_id,
                        &[],
                    )
                    .await?;

                    let Some(stripe_address) = customer.address else {
                        return Err(ApiError::InvalidInput(
                            "Stripe customer had no address".to_owned(),
                        ));
                    };

                    let customer_address =
                        anrok::Address::from_stripe_address(&stripe_address);

                    let tax_amount = anrok_client
                        .create_ephemeral_txn(&anrok::TransactionFields {
                            customer_address,
                            currency_code: charge.currency_code.clone(),
                            accounting_time: charge.due,
                            accounting_time_zone:
                                anrok::AccountingTimeZone::Utc,
                            line_items: vec![anrok::LineItem::new(
                                product_tax_id,
                                charge.amount,
                            )],
                        })
                        .await?
                        .tax_amount_to_collect;

                    charge.tax_amount = tax_amount;
                    charge.tax_last_updated = Some(Utc::now());

                    Result::<DBCharge, ApiError>::Ok(charge)
                }
            })
            .collect::<FuturesUnordered<_>>();

        let mut upsert_results = vec![];

        while let Some(result) = futures.next().await {
            match result {
                Ok(charge) => {
                    upsert_results.push(charge.upsert(&mut txn).await);
                }
                Err(error) => {
                    error!(%error, "Error indexing tax amount on charge");
                }
            }
        }

        let _ = upsert_results
            .into_iter()
            .collect::<Result<Vec<_>, DatabaseError>>()?;

        txn.commit().await?;

        processed_charges += 5;

        if processed_charges >= limit {
            break Ok(());
        }
    }
}

async fn index_tax_notifications(
    pool: PgPool,
    redis: RedisPool,
) -> Result<(), ApiError> {
    info!("Indexing tax notifications");

    let mut txn = pool.begin().await?;

    let subscriptions =
        fetch_update_lock_pending_taxation_notification(&mut txn, 300).await?;

    let users = DBUser::get_many_ids(
        &subscriptions.iter().map(|x| x.user_id).collect::<Vec<_>>(),
        &pool,
        &redis,
    )
    .await?;

    for subs in subscriptions {
        let Some(user) = users.iter().find(|x| x.id == subs.user_id) else {
            continue;
        };

        // Some users don't have an email and so can't send them a notification. Just
        // skip these people for now as they count very few users on the site with
        // subscriptions.
        //
        // We skip them to be sure here but they're already skipped in `fetch_update_lock_pending_taxation_notification`.
        if user.email.is_none() {
            continue;
        }

        NotificationBuilder {
            body: NotificationBody::TaxNotification {
                amount: subs.amount,
                tax_amount: subs.tax_amount,
                due: subs.due,
                service: subs.product_metadata.as_product_denomination(),
            },
        }
        .insert(user.id, &mut txn, &redis)
        .await?;
    }

    txn.commit().await?;

    Ok(())
}
