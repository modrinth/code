use chrono::{Datelike, Duration, TimeZone, Utc};
use eyre::{Context, Result, eyre};
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::warn;

use crate::database::models::{DBAffiliateCodeId, DBUserId};

pub async fn process_affiliate_payouts(postgres: &PgPool) -> Result<()> {
    // process:
    // - get any subscriptions which are in `users_subscriptions_affiliations`
    // - for those subscriptions, get any charges which are not in `users_subscriptions_affiliations_payouts`
    // - for each of those charges,
    //   - get the subscription's `affiliate_code`
    //   - get the affiliate user of that code
    //   - add a payout for that affiliate user, proportional to the `net - tax` of the charge
    //   - add a record of this into `users_subscriptions_affiliations_payouts`

    let mut txn = postgres
        .begin()
        .await
        .wrap_err("failed to begin transaction")?;

    let charges = sqlx::query!(
        r#"
        SELECT
            c.id as charge_id,
            c.subscription_id AS "subscription_id!",
            c.net as charge_net,
            c.tax_amount as charge_tax_amount,
            c.last_attempt as charge_last_attempt,
            c.currency_code,
            usa.affiliate_code,
            ac.affiliate as affiliate_user_id,
            ac.revenue_split
        -- get any charges...
        FROM charges c
        -- ...which have a subscription...
        INNER JOIN users_subscriptions_affiliations usa
            ON c.subscription_id = usa.subscription_id
            AND c.subscription_id IS NOT NULL
            AND (usa.deactivated_at IS NULL OR c.last_attempt < usa.deactivated_at)
        -- ...which have an affiliate code...
        INNER JOIN affiliate_codes ac
            ON usa.affiliate_code = ac.id
        -- ...and where no payout to an affiliate has been made for this charge yet
        LEFT JOIN users_subscriptions_affiliations_payouts usap
            ON c.id = usap.charge_id
        WHERE
            c.status = 'succeeded'
            AND c.net > 0
            AND usap.id IS NULL
        "#
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_err("failed to fetch charges awaiting affiliate payout")?;

    let default_affiliate_revenue_split =
        dotenvy::var("DEFAULT_AFFILIATE_REVENUE_SPLIT")
            .wrap_err("no env var `DEFAULT_AFFILIATE_REVENUE_SPLIT`")?
            .parse::<Decimal>()
            .wrap_err("`DEFAULT_AFFILIATE_REVENUE_SPLIT` is not a decimal")?;

    let (
        mut insert_usap_charges,
        mut insert_usap_subscriptions,
        mut insert_usap_affiliate_codes,
        mut insert_usap_payout_values,
    ) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());

    for row in charges {
        let Some(net) = row.charge_net else {
            warn!(
                "Charge {} has no net amount; cannot calculate affiliate payout",
                row.charge_id
            );
            continue;
        };
        let net = Decimal::new(net, 2);
        let tax_amount = Decimal::new(row.charge_tax_amount, 2);

        let Some(last_attempt) = row.charge_last_attempt else {
            warn!(
                "Charge {} has no last attempt; cannot calculate affiliate payout",
                row.charge_id
            );
            continue;
        };

        // affiliate payouts are Net 60 from the end of the month
        // this is net 60 relative to the time of the charge's last attempt, not from now
        let available = {
            let year = last_attempt.year();
            let month = last_attempt.month();

            // get the first day of the next month
            let last_day_of_month = if month == 12 {
                Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap()
            } else {
                Utc.with_ymd_and_hms(year, month + 1, 1, 0, 0, 0).unwrap()
            };

            last_day_of_month + Duration::days(59)
        };

        let revenue_split = row
            .revenue_split
            .and_then(Decimal::from_f64_retain)
            .unwrap_or(default_affiliate_revenue_split);
        if !(Decimal::from(0)..=Decimal::from(1)).contains(&revenue_split) {
            warn!(
                "Charge {} has revenue split {revenue_split} which is out of range",
                row.charge_id,
            );
            continue;
        }

        let affiliate_cut = (net - tax_amount) * revenue_split;
        let affiliate_user_id = DBUserId(row.affiliate_user_id);
        let affiliate_code_id = DBAffiliateCodeId(row.affiliate_code);

        // don't batch up affiliate code payouts into one big payout per (user, affiliate code)
        // because we want to associate 1 payout to an affiliate, with 1 concrete charge

        let payout_value_id = sqlx::query!(
            "
            INSERT INTO payouts_values
                (user_id, amount, created,
                date_available, affiliate_code_source)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            ",
            affiliate_user_id as _,
            affiliate_cut,
            last_attempt,
            available,
            affiliate_code_id as _,
        )
        .fetch_one(&mut *txn)
        .await
        .wrap_err_with(|| eyre!("failed to insert payout value for ({affiliate_user_id:?}, {affiliate_code_id:?})"))?
        .id;

        insert_usap_charges.push(row.charge_id);
        insert_usap_subscriptions.push(row.subscription_id);
        insert_usap_affiliate_codes.push(affiliate_code_id.0);
        insert_usap_payout_values.push(payout_value_id);
    }

    sqlx::query!(
        "
        INSERT INTO users_subscriptions_affiliations_payouts
            (charge_id, subscription_id,
            affiliate_code, payout_value_id)
        SELECT * FROM UNNEST($1::bigint[], $2::bigint[], $3::bigint[], $4::bigint[])
        ",
        &insert_usap_charges[..],
        &insert_usap_subscriptions[..],
        &insert_usap_affiliate_codes[..],
        &insert_usap_payout_values[..],
    )
    .execute(&mut *txn)
    .await
    .wrap_err("failed to associate charges with affiliate payouts")?;

    txn.commit()
        .await
        .wrap_err("failed to commit transaction")?;

    Ok(())
}
