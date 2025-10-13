use std::collections::HashMap;

use ariadne::ids::UserId;
use chrono::{Datelike, Duration, TimeZone, Utc};
use eyre::Result;
use rust_decimal::{Decimal, dec};
use sqlx::PgPool;
use tracing::{trace, warn};

use crate::{
    database::models::{DBAffiliateCodeId, DBUserId},
    models::ids::AffiliateCodeId,
};

const AFFILIATE_CUT_PERCENTAGE: Decimal = dec!(0.1);

pub async fn process_affiliate_code_revenue(pool: &PgPool) -> Result<()> {
    let end_date = Utc::now() - Duration::days(30);
    let start_date = end_date - Duration::days(30);

    let affiliate_charges = sqlx::query!(
        r#"
        SELECT
            c.id as charge_id,
            c.affiliate_code,
            c.net,
            ac.affiliate as affiliate_user_id,
            ac.revenue_split
        FROM charges c
        INNER JOIN affiliate_codes ac ON c.affiliate_code = ac.id
        WHERE
            c.status = 'succeeded'
            AND c.net > 0
            AND c.due BETWEEN $1 AND $2
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    if affiliate_charges.is_empty() {
        return Ok(());
    }

    let mut transaction = pool.begin().await?;

    // Group by affiliate user and affiliate code to create unique rows per affiliate code
    let mut affiliate_payouts: HashMap<(DBUserId, DBAffiliateCodeId), Decimal> =
        HashMap::new();

    for charge in affiliate_charges {
        let Some(net_amount) = charge.net else {
            continue;
        };
        let net_amount = Decimal::new(net_amount, 2);

        let affiliate_user_id = DBUserId(charge.affiliate_user_id);
        let affiliate_code_id =
            DBAffiliateCodeId(charge.affiliate_code.unwrap());

        // Use the custom revenue split if specified, otherwise use the default 10%
        let revenue_split =
            charge
                .revenue_split
                .map_or(AFFILIATE_CUT_PERCENTAGE, |split| {
                    Decimal::from_f64_retain(split)
                        .unwrap_or(AFFILIATE_CUT_PERCENTAGE)
                });

        if revenue_split < dec!(0) || revenue_split > dec!(1) {
            warn!(
                "Charge {} resulted in invalid revenue split {revenue_split}",
                charge.charge_id
            );
            continue;
        }

        let affiliate_cut = net_amount * revenue_split;

        if affiliate_cut > dec!(0) {
            *affiliate_payouts
                .entry((affiliate_user_id, affiliate_code_id))
                .or_insert(dec!(0)) += affiliate_cut;
        }
    }

    let mut insert_user_ids = Vec::new();
    let mut insert_payouts = Vec::new();
    let mut insert_starts = Vec::new();
    let mut insert_availables = Vec::new();
    let mut insert_affiliate_code_ids = Vec::new();

    let created_timestamp = end_date;

    // Affiliate payouts are Net 30 from the end of the processing month
    let available_timestamp = {
        let processing_month = end_date.date_naive();
        let year = processing_month.year();
        let month = processing_month.month();

        let first_of_next_month = if month == 12 {
            Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap()
        } else {
            Utc.with_ymd_and_hms(year, month + 1, 1, 0, 0, 0).unwrap()
        };

        first_of_next_month + Duration::days(29)
    };

    for ((user_id, affiliate_code_id), total_payout) in affiliate_payouts {
        if total_payout > dec!(0) {
            insert_user_ids.push(user_id.0);
            insert_payouts.push(total_payout);
            insert_starts.push(created_timestamp);
            insert_availables.push(available_timestamp);
            insert_affiliate_code_ids.push(affiliate_code_id.0);

            trace!(
                "User {} gets {total_payout} from affiliate code {}",
                UserId::from(user_id),
                AffiliateCodeId::from(affiliate_code_id),
            );
        }
    }

    if !insert_user_ids.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO payouts_values (user_id, amount, created, date_available, affiliate_code_id)
            SELECT * FROM UNNEST ($1::bigint[], $2::numeric[], $3::timestamptz[], $4::timestamptz[], $5::bigint[])
            "#,
            &insert_user_ids[..],
            &insert_payouts[..],
            &insert_starts[..],
            &insert_availables[..],
            &insert_affiliate_code_ids[..],
        )
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(())
}
