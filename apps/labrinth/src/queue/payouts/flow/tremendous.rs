use chrono::Utc;
use eyre::eyre;
use modrinth_util::decimal::Decimal2dp;
use reqwest::Method;
use rust_decimal::{Decimal, RoundingStrategy, dec};
use serde::Deserialize;
use serde_json::json;

use crate::{
    database::models::payout_item::DBPayout,
    models::payouts::{
        PayoutMethod, PayoutMethodFee, PayoutMethodType, PayoutStatus,
        TremendousCurrency, TremendousDetails, TremendousForexResponse,
    },
    queue::payouts::{
        PayoutsQueue,
        flow::{
            ExecuteContext, PayoutFlow, PayoutFlowInner, get_verified_email,
        },
    },
    routes::ApiError,
    util::error::Context,
};

#[derive(Debug)]
pub(super) struct TremendousFlow {
    value_denomination: Decimal,
    value_currency_code: String,
    net_usd: Decimal2dp,
    total_fee_usd: Decimal2dp,
    delivery_email: String,
    method_id: String,
}

pub(super) async fn create(
    queue: &PayoutsQueue,
    amount: Decimal,
    details: TremendousDetails,
    method: &PayoutMethod,
) -> Result<PayoutFlow, ApiError> {
    let forex: TremendousForexResponse = queue
        .make_tremendous_request(Method::GET, "forex", None::<()>)
        .await
        .wrap_internal_err("failed to fetch Tremendous forex data")?;
    let usd_to_currency_for = |currency_code: &str| {
        forex
            .forex
            .get(currency_code)
            .copied()
            .wrap_internal_err_with(|| {
                eyre!("no Tremendous forex rate for '{currency_code}'")
            })
    };

    let category = method.category.as_ref().wrap_internal_err_with(|| {
        eyre!("method '{}' should have a category", method.id)
    })?;

    let delivery_email = details.delivery_email;
    let method_id = method.id.clone();

    match category.as_str() {
        "paypal" | "venmo" => {
            let currency = details.currency.unwrap_or(TremendousCurrency::Usd);
            let currency_code = currency.to_string();
            let usd_to_currency = usd_to_currency_for(&currency_code)?;

            let fee = PayoutMethodFee {
                // If a user withdraws $10:
                //
                //   amount charged by Tremendous = X * 1.04 = $10.00
                //
                // We have to solve for X here:
                //
                //   X = $10.00 / 1.04
                //
                // So the percentage fee is `1 - (1 / 1.04)`
                // Roughly 0.03846, not 0.04
                percentage: dec!(1) - (dec!(1) / dec!(1.04)),
                min: dec!(0.25),
                max: None,
            };

            let gross_usd =
                Decimal2dp::new(amount).wrap_request_err("invalid amount")?;

            let total_fee_usd = Decimal2dp::rounded(
                fee.compute_fee(amount),
                RoundingStrategy::AwayFromZero,
            );
            let net_usd = gross_usd - total_fee_usd;

            Ok(PayoutFlow {
                net_usd,
                total_fee_usd,
                min_amount_usd: Decimal2dp::ZERO,
                max_amount_usd: Decimal2dp::new(dec!(5000.0)).unwrap(),
                forex_usd_to_currency: Some(usd_to_currency),
                inner: PayoutFlowInner::Tremendous(TremendousFlow {
                    // In the Tremendous dashboard, we have configured it so that,
                    // if we make a $10 request for a premium method, *we* get
                    // charged an extra 4% - the user gets the full $10, and we get
                    // $10.40 subtracted from our Tremendous balance.
                    //
                    // To offset this, we (the platform) take the fees off before
                    // we send the request to Tremendous. Afterwards, the method
                    // (Tremendous) will take 0% off the top of our $10.
                    value_denomination: net_usd.get(),
                    value_currency_code: TremendousCurrency::Usd.to_string(),
                    net_usd,
                    total_fee_usd,
                    delivery_email,
                    method_id,
                }),
            })
        }
        _ => {
            let currency_code =
                if let Some(currency_code) = &method.currency_code {
                    currency_code.clone()
                } else {
                    TremendousCurrency::Usd.to_string()
                };
            let usd_to_currency = usd_to_currency_for(&currency_code)?;
            let currency_to_usd = dec!(1) / usd_to_currency;

            // no fees
            let net_usd = Decimal2dp::rounded(
                amount * currency_to_usd,
                RoundingStrategy::AwayFromZero,
            );

            Ok(PayoutFlow {
                net_usd,
                total_fee_usd: Decimal2dp::ZERO,
                min_amount_usd: Decimal2dp::ZERO,
                max_amount_usd: Decimal2dp::new(dec!(10_000.0)).unwrap(),
                forex_usd_to_currency: Some(usd_to_currency),
                inner: PayoutFlowInner::Tremendous(TremendousFlow {
                    // we have to use the exact `amount` here,
                    // since interval cards (e.g. PLN 70.00)
                    // require you to input that exact amount
                    value_denomination: amount,
                    value_currency_code: currency_code,
                    net_usd,
                    total_fee_usd: Decimal2dp::ZERO,
                    delivery_email,
                    method_id,
                }),
            })
        }
    }
}

pub(super) async fn execute(
    ExecuteContext {
        queue,
        user,
        payout_id,
        mut transaction,
        gotenberg: _,
    }: ExecuteContext<'_>,
    TremendousFlow {
        value_denomination,
        value_currency_code,
        net_usd,
        total_fee_usd,
        delivery_email,
        method_id,
    }: TremendousFlow,
) -> Result<(), ApiError> {
    #[derive(Debug, Deserialize)]
    struct Reward {
        pub id: String,
    }

    #[derive(Debug, Deserialize)]
    struct Order {
        pub rewards: Vec<Reward>,
    }

    #[derive(Debug, Deserialize)]
    struct TremendousResponse {
        pub order: Order,
    }

    let user_email = get_verified_email(user)?;

    let order_req = json!({
        "payment": {
            "funding_source_id": "BALANCE",
        },
        "rewards": [{
            "value": {
                "denomination": value_denomination,
                "currency_code": value_currency_code,
            },
            "delivery": {
                "method": "EMAIL"
            },
            "recipient": {
                "name": user.username,
                "email": delivery_email
            },
            "products": [
                method_id,
            ],
            "campaign_id": dotenvy::var("TREMENDOUS_CAMPAIGN_ID")?,
        }]
    });

    let order_res: TremendousResponse = queue
        .make_tremendous_request(Method::POST, "orders", Some(order_req))
        .await
        .wrap_internal_err("failed to make Tremendous order request")?;

    let platform_id = order_res
        .order
        .rewards
        .first()
        .map(|reward| reward.id.clone());

    DBPayout {
        id: payout_id,
        user_id: user.id,
        created: Utc::now(),
        status: PayoutStatus::InTransit,
        amount: net_usd.get(),
        fee: Some(total_fee_usd.get()),
        method: Some(PayoutMethodType::Tremendous),
        method_id: Some(method_id),
        method_address: Some(user_email.to_string()),
        platform_id,
    }
    .insert(&mut transaction)
    .await
    .wrap_internal_err("failed to insert payout")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}
