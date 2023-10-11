use crate::routes::ApiError;
use crate::util::env::parse_var;
use crate::{database::redis::RedisPool, models::projects::MonetizationStatus};
use base64::Engine;
use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;

pub struct PayoutsQueue {
    credential: PaypalCredential,
    credential_expires: DateTime<Utc>,
}

#[derive(Deserialize, Default)]
struct PaypalCredential {
    access_token: String,
    token_type: String,
    expires_in: i64,
}

#[derive(Serialize)]
pub struct PayoutItem {
    pub amount: PayoutAmount,
    pub receiver: String,
    pub note: String,
    pub recipient_type: String,
    pub recipient_wallet: String,
    pub sender_item_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct PayoutAmount {
    pub currency: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub value: Decimal,
}

// Batches payouts and handles token refresh
impl PayoutsQueue {
    pub fn new() -> Self {
        PayoutsQueue {
            credential: Default::default(),
            credential_expires: Utc::now() - Duration::days(30),
        }
    }

    pub async fn refresh_token(&mut self) -> Result<(), ApiError> {
        let client = reqwest::Client::new();

        let combined_key = format!(
            "{}:{}",
            dotenvy::var("PAYPAL_CLIENT_ID")?,
            dotenvy::var("PAYPAL_CLIENT_SECRET")?
        );
        let formatted_key = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(combined_key)
        );

        let mut form = HashMap::new();
        form.insert("grant_type", "client_credentials");

        let credential: PaypalCredential = client
            .post(&format!("{}oauth2/token", dotenvy::var("PAYPAL_API_URL")?))
            .header("Accept", "application/json")
            .header("Accept-Language", "en_US")
            .header("Authorization", formatted_key)
            .form(&form)
            .send()
            .await
            .map_err(|_| ApiError::Payments("Error while authenticating with PayPal".to_string()))?
            .json()
            .await
            .map_err(|_| {
                ApiError::Payments(
                    "Error while authenticating with PayPal (deser error)".to_string(),
                )
            })?;

        self.credential_expires = Utc::now() + Duration::seconds(credential.expires_in);
        self.credential = credential;

        Ok(())
    }

    pub async fn send_payout(&mut self, mut payout: PayoutItem) -> Result<Decimal, ApiError> {
        if self.credential_expires < Utc::now() {
            self.refresh_token().await.map_err(|_| {
                ApiError::Payments("Error while authenticating with PayPal".to_string())
            })?;
        }

        let wallet = payout.recipient_wallet.clone();

        let fee = if wallet == *"Venmo" {
            Decimal::ONE / Decimal::from(4)
        } else {
            std::cmp::min(
                std::cmp::max(
                    Decimal::ONE / Decimal::from(4),
                    (Decimal::from(2) / Decimal::ONE_HUNDRED) * payout.amount.value,
                ),
                Decimal::from(20),
            )
        };

        payout.amount.value -= fee;
        payout.amount.value = payout.amount.value.round_dp(2);

        if payout.amount.value <= Decimal::ZERO {
            return Err(ApiError::InvalidInput(
                "You do not have enough funds to make this payout!".to_string(),
            ));
        }

        let client = reqwest::Client::new();

        let res = client.post(&format!("{}payments/payouts", dotenvy::var("PAYPAL_API_URL")?))
            .header("Authorization", format!("{} {}", self.credential.token_type, self.credential.access_token))
            .json(&json! ({
                    "sender_batch_header": {
                        "sender_batch_id": format!("{}-payouts", Utc::now().to_rfc3339()),
                        "email_subject": "You have received a payment from Modrinth!",
                        "email_message": "Thank you for creating projects on Modrinth. Please claim this payment within 30 days.",
                    },
                    "items": vec![payout]
                }))
            .send().await.map_err(|_| ApiError::Payments("Error while sending payout to PayPal".to_string()))?;

        if !res.status().is_success() {
            #[derive(Deserialize)]
            struct PayPalError {
                pub body: PayPalErrorBody,
            }

            #[derive(Deserialize)]
            struct PayPalErrorBody {
                pub message: String,
            }

            let body: PayPalError = res.json().await.map_err(|_| {
                ApiError::Payments("Error while registering payment in PayPal!".to_string())
            })?;

            return Err(ApiError::Payments(format!(
                "Error while registering payment in PayPal: {}",
                body.body.message
            )));
        } else if wallet != *"Venmo" {
            #[derive(Deserialize)]
            struct PayPalLink {
                href: String,
            }

            #[derive(Deserialize)]
            struct PayoutsResponse {
                pub links: Vec<PayPalLink>,
            }

            #[derive(Deserialize)]
            struct PayoutDataItem {
                payout_item_fee: PayoutAmount,
            }

            #[derive(Deserialize)]
            struct PayoutData {
                pub items: Vec<PayoutDataItem>,
            }

            // Calculate actual fee + refund if we took too big of a fee.
            if let Ok(res) = res.json::<PayoutsResponse>().await {
                if let Some(link) = res.links.first() {
                    if let Ok(res) = client
                        .get(&link.href)
                        .header(
                            "Authorization",
                            format!(
                                "{} {}",
                                self.credential.token_type, self.credential.access_token
                            ),
                        )
                        .send()
                        .await
                    {
                        if let Ok(res) = res.json::<PayoutData>().await {
                            if let Some(data) = res.items.first() {
                                if (fee - data.payout_item_fee.value) > Decimal::ZERO {
                                    return Ok(fee - data.payout_item_fee.value);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Decimal::ZERO)
    }
}

pub async fn process_payout(
    pool: &PgPool,
    redis: &RedisPool,
    client: &clickhouse::Client,
) -> Result<(), ApiError> {
    let start: DateTime<Utc> = DateTime::from_utc(
        (Utc::now() - Duration::days(1))
            .date_naive()
            .and_hms_nano_opt(0, 0, 0, 0)
            .unwrap_or_default(),
        Utc,
    );

    let results = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM payouts_values WHERE created = $1)",
        start,
    )
    .fetch_one(pool)
    .await?;

    if results.exists.unwrap_or(false) {
        return Ok(());
    }

    let end = start + Duration::days(1);
    #[derive(Deserialize, clickhouse::Row)]
    struct ProjectMultiplier {
        pub page_views: u64,
        pub project_id: u64,
    }

    let (views_values, views_sum, downloads_values, downloads_sum) = futures::future::try_join4(
        client
            .query(
                r#"
                SELECT COUNT(id) page_views, project_id
                FROM views
                WHERE (recorded BETWEEN ? AND ?) AND (project_id != 0)
                GROUP BY project_id
                ORDER BY page_views DESC
                "#,
            )
            .bind(start.timestamp())
            .bind(end.timestamp())
            .fetch_all::<ProjectMultiplier>(),
        client
            .query("SELECT COUNT(id) FROM views WHERE (recorded BETWEEN ? AND ?) AND (project_id != 0)")
            .bind(start.timestamp())
            .bind(end.timestamp())
            .fetch_one::<u64>(),
        client
            .query(
                r#"
                SELECT COUNT(id) page_views, project_id
                FROM downloads
                WHERE (recorded BETWEEN ? AND ?) AND (user_id != 0)
                GROUP BY project_id
                ORDER BY page_views DESC
                "#,
            )
            .bind(start.timestamp())
            .bind(end.timestamp())
            .fetch_all::<ProjectMultiplier>(),
        client
            .query("SELECT COUNT(id) FROM downloads WHERE (recorded BETWEEN ? AND ?) AND (user_id != 0)")
            .bind(start.timestamp())
            .bind(end.timestamp())
            .fetch_one::<u64>(),
    )
        .await?;

    let mut transaction = pool.begin().await?;

    struct PayoutMultipliers {
        sum: u64,
        values: HashMap<u64, u64>,
    }

    let mut views_values = views_values
        .into_iter()
        .map(|x| (x.project_id, x.page_views))
        .collect::<HashMap<u64, u64>>();
    let downloads_values = downloads_values
        .into_iter()
        .map(|x| (x.project_id, x.page_views))
        .collect::<HashMap<u64, u64>>();

    for (key, value) in downloads_values.iter() {
        let counter = views_values.entry(*key).or_insert(0);
        *counter += *value;
    }

    let multipliers: PayoutMultipliers = PayoutMultipliers {
        sum: downloads_sum + views_sum,
        values: views_values,
    };

    struct Project {
        // user_id, payouts_split
        team_members: Vec<(i64, Decimal)>,
    }

    let mut projects_map: HashMap<i64, Project> = HashMap::new();

    use futures::TryStreamExt;

    sqlx::query!(
        "
        SELECT m.id id, tm.user_id user_id, tm.payouts_split payouts_split
        FROM mods m
        INNER JOIN team_members tm on m.team_id = tm.team_id AND tm.accepted = TRUE
        WHERE m.id = ANY($1) AND m.monetization_status = $2
        ",
        &multipliers
            .values
            .keys()
            .map(|x| *x as i64)
            .collect::<Vec<i64>>(),
        MonetizationStatus::Monetized.as_str(),
    )
    .fetch_many(&mut *transaction)
    .try_for_each(|e| {
        if let Some(row) = e.right() {
            if let Some(project) = projects_map.get_mut(&row.id) {
                project.team_members.push((row.user_id, row.payouts_split));
            } else {
                projects_map.insert(
                    row.id,
                    Project {
                        team_members: vec![(row.user_id, row.payouts_split)],
                    },
                );
            }
        }

        futures::future::ready(Ok(()))
    })
    .await?;

    let amount = Decimal::from(parse_var::<u64>("PAYOUTS_BUDGET").unwrap_or(0));

    let days = Decimal::from(28);
    let weekdays = Decimal::from(20);
    let weekend_bonus = Decimal::from(5) / Decimal::from(4);

    let weekday_amount = amount / (weekdays + (weekend_bonus) * (days - weekdays));
    let weekend_amount = weekday_amount * weekend_bonus;

    let payout = match start.weekday() {
        Weekday::Sat | Weekday::Sun => weekend_amount,
        _ => weekday_amount,
    };

    let mut clear_cache_users = Vec::new();
    let (mut insert_user_ids, mut insert_project_ids, mut insert_payouts, mut insert_starts) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (id, project) in projects_map {
        if let Some(value) = &multipliers.values.get(&(id as u64)) {
            let project_multiplier: Decimal =
                Decimal::from(**value) / Decimal::from(multipliers.sum);

            let sum_splits: Decimal = project.team_members.iter().map(|x| x.1).sum();

            if sum_splits > Decimal::ZERO {
                for (user_id, split) in project.team_members {
                    let payout: Decimal = payout * project_multiplier * (split / sum_splits);

                    if payout > Decimal::ZERO {
                        insert_user_ids.push(user_id);
                        insert_project_ids.push(id);
                        insert_payouts.push(payout);
                        insert_starts.push(start);

                        sqlx::query!(
                            "
                            UPDATE users
                            SET balance = balance + $1
                            WHERE id = $2
                            ",
                            payout,
                            user_id
                        )
                        .execute(&mut *transaction)
                        .await?;

                        clear_cache_users.push(user_id);
                    }
                }
            }
        }
    }

    sqlx::query!(
        "
        INSERT INTO payouts_values (user_id, mod_id, amount, created)
        SELECT * FROM UNNEST ($1::bigint[], $2::bigint[], $3::numeric[], $4::timestamptz[])
        ",
        &insert_user_ids[..],
        &insert_project_ids[..],
        &insert_payouts[..],
        &insert_starts[..]
    )
    .execute(&mut *transaction)
    .await?;

    if !clear_cache_users.is_empty() {
        crate::database::models::User::clear_caches(
            &clear_cache_users
                .into_iter()
                .map(|x| (crate::database::models::UserId(x), None))
                .collect::<Vec<_>>(),
            redis,
        )
        .await?;
    }

    transaction.commit().await?;

    Ok(())
}
