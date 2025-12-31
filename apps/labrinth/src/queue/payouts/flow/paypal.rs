use chrono::Utc;
use modrinth_util::decimal::Decimal2dp;
use reqwest::Method;
use rust_decimal::{Decimal, RoundingStrategy, dec};
use serde::Deserialize;
use serde_json::json;
use tracing::error;

use crate::{
    database::models::payout_item::DBPayout,
    models::payouts::{PayoutMethodFee, PayoutMethodType, PayoutStatus},
    queue::payouts::{
        PayoutsQueue,
        flow::{ExecuteContext, PayoutFlow, PayoutFlowInner},
    },
    routes::ApiError,
    util::error::Context,
};

pub const FEE: PayoutMethodFee = PayoutMethodFee {
    percentage: dec!(0.02),
    min: dec!(0.25),
    max: Some(dec!(1.0)),
};

pub const MIN_USD: Decimal2dp = Decimal2dp::new_unchecked(dec!(0.25));
pub const MAX_USD: Decimal2dp = Decimal2dp::new_unchecked(dec!(100_000.0));

#[derive(Debug)]
pub(super) struct PayPalFlow {
    is_venmo: bool,
    net_usd: Decimal2dp,
    fee_usd: Decimal2dp,
}

pub(super) async fn create(
    _queue: &PayoutsQueue,
    amount: Decimal,
    is_venmo: bool,
) -> Result<PayoutFlow, ApiError> {
    let gross_usd =
        Decimal2dp::new(amount).wrap_request_err("invalid amount")?;
    let fee_usd = Decimal2dp::rounded(
        FEE.compute_fee(amount),
        RoundingStrategy::AwayFromZero,
    );
    let net_usd = gross_usd - fee_usd;

    Ok(PayoutFlow {
        net_usd,
        total_fee_usd: fee_usd,
        min_amount_usd: MIN_USD,
        max_amount_usd: MAX_USD,
        forex_usd_to_currency: None,
        inner: PayoutFlowInner::PayPal(PayPalFlow {
            is_venmo,
            net_usd,
            fee_usd,
        }),
    })
}

pub(super) async fn execute(
    ExecuteContext {
        queue,
        user,
        payout_id,
        mut transaction,
        gotenberg: _,
    }: ExecuteContext<'_>,
    PayPalFlow {
        is_venmo,
        net_usd,
        fee_usd,
    }: PayPalFlow,
) -> Result<(), ApiError> {
    #[derive(Deserialize)]
    struct PayPalLink {
        href: String,
    }

    #[derive(Deserialize)]
    struct PayoutsResponse {
        pub links: Vec<PayPalLink>,
    }

    #[derive(Deserialize)]
    struct PayoutItem {
        pub payout_item_id: String,
    }

    #[derive(Deserialize)]
    struct PayoutData {
        pub items: Vec<PayoutItem>,
    }

    // keep the `method_id` code here since the big if block below is legacy code
    // when we had paypal intl methods as well
    let method_id = if is_venmo { "venmo" } else { "paypal_us" };

    let (wallet, wallet_type, address, display_address) = if is_venmo {
        if let Some(venmo) = &user.venmo_handle {
            ("Venmo", "user_handle", venmo.clone(), venmo)
        } else {
            return Err(ApiError::InvalidInput(
                "Venmo address has not been set for account!".to_string(),
            ));
        }
    } else if let Some(paypal_id) = &user.paypal_id {
        if let Some(paypal_country) = &user.paypal_country {
            if paypal_country == "US" && method_id != "paypal_us" {
                return Err(ApiError::InvalidInput(
                    "Please use the US PayPal transfer option!".to_string(),
                ));
            } else if paypal_country != "US" && method_id == "paypal_us" {
                return Err(ApiError::InvalidInput(
                    "Please use the International PayPal transfer option!"
                        .to_string(),
                ));
            }

            (
                "PayPal",
                "paypal_id",
                paypal_id.clone(),
                user.paypal_email.as_ref().unwrap_or(paypal_id),
            )
        } else {
            return Err(ApiError::InvalidInput(
                "Please re-link your PayPal account!".to_string(),
            ));
        }
    } else {
        return Err(ApiError::InvalidInput(
            "You have not linked a PayPal account!".to_string(),
        ));
    };

    let payout_req = json!({
        "sender_batch_header": {
            "sender_batch_id": format!("{}-payouts", Utc::now().to_rfc3339()),
            "email_subject": "You have received a payment from Modrinth!",
            "email_message": "Thank you for creating projects on Modrinth. Please claim this payment within 30 days.",
        },
        "items": [{
            "amount": {
                "currency": "USD",
                "value": net_usd.to_string()
            },
            "receiver": address,
            "note": "Payment from Modrinth creator monetization program",
            "recipient_type": wallet_type,
            "recipient_wallet": wallet,
            "sender_item_id": crate::models::ids::PayoutId::from(payout_id),
        }]
    });

    let res: PayoutsResponse = queue
        .make_paypal_request(
            Method::POST,
            "payments/payouts",
            Some(payout_req),
            None,
            None,
        )
        .await
        .wrap_internal_err("failed to make payout request")?;

    // by this point, we've made a monetary payout request to this user;
    // no matter what we do, we *must* track this payout in the DB,
    // even if the next steps fail, so that the user's balance is subtracted.

    let mut payout = DBPayout {
        id: payout_id,
        user_id: user.id,
        created: Utc::now(),
        status: PayoutStatus::InTransit,
        amount: net_usd.get(),
        fee: Some(fee_usd.get()),
        method: Some(if is_venmo {
            PayoutMethodType::Venmo
        } else {
            PayoutMethodType::PayPal
        }),
        method_id: Some(method_id.to_string()),
        method_address: Some(display_address.clone()),
        platform_id: None, // attempt to populate this later
    };

    // poor man's async try/catch block
    let result = (async {
        let link = res
            .links
            .first()
            .wrap_request_err("no PayPal links available")?;

        let res = queue
            .make_paypal_request::<(), PayoutData>(
                Method::GET,
                &link.href,
                None,
                None,
                Some(true),
            )
            .await
            .wrap_internal_err("failed to make PayPal link request")?;
        let data = res.items.first().wrap_internal_err(
            "no payout items returned from PayPal link request",
        )?;

        payout.platform_id = Some(data.payout_item_id.clone());
        Ok::<_, ApiError>(())
    })
    .await;

    if let Err(err) = result {
        error!(
            "Failed to get PayPal payout platform ID, will track this payout with no platform ID: {err:#}"
        );
    }

    payout
        .insert(&mut transaction)
        .await
        .wrap_internal_err("failed to insert payout")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}
