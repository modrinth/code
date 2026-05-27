use actix_web::{post, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    database::{
        PgPool, PgTransaction,
        models::{
            DBCampaignDonationId, DBUser, DBUserId,
            generate_campaign_donation_id,
        },
        redis::RedisPool,
    },
    models::payouts::TremendousForexResponse,
    queue::payouts::PayoutsQueue,
    routes::ApiError,
    util::error::Context,
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(tiltify_webhook);
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyWebhook {
    pub data: TiltifyData,
    pub meta: TiltifyMeta,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyData {
    pub amount_raised: AmountRaised,
    pub user: TiltifyUser,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AmountRaised {
    pub currency: String,
    pub value: Decimal,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyUser {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyMeta {
    pub attempted_at: DateTime<Utc>,
    pub event_type: String,
    pub generated_at: DateTime<Utc>,
    pub id: Uuid,
    pub subscription_source_id: Uuid,
    pub subscription_source_type: String,
}

pub struct CampaignDonation {
    pub id: DBCampaignDonationId,
    pub raw_data: serde_json::Value,
    pub donated_at: DateTime<Utc>,
    pub amount_usd: Option<Decimal>,
    pub user_id: Option<DBUserId>,
}

impl CampaignDonation {
    pub async fn insert(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), ApiError> {
        let user_id = self.user_id.map(|id| id.0);
        sqlx::query!(
            "
            insert into campaign_donations (id, raw_data, donated_at, amount_usd, user_id)
            values ($1, $2, $3, $4, $5)
            ",
            self.id.0,
            self.raw_data,
            self.donated_at,
            self.amount_usd,
            user_id,
        )
        .execute(transaction)
        .await
        .wrap_internal_err("inserting campaign donation")?;

        Ok(())
    }
}

#[utoipa::path]
#[post("/webhook")]
pub async fn tiltify_webhook(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    payouts_queue: web::Data<PayoutsQueue>,
    web::Json(raw_payload): web::Json<serde_json::Value>,
) -> Result<(), ApiError> {
    // deserialize the JSON in the request handler, not in the params,
    // since if the JSON fails to deserialize then it's *our* fault,
    // not the caller's.
    let payload = TiltifyWebhook::deserialize(&raw_payload)
        .wrap_internal_err_with(|| {
            eyre!(
                "invalid Tiltify webhook payload schema\n{}",
                serde_json::to_string_pretty(&raw_payload)
                    .expect("serializing should not fail")
            )
        })?;

    // no matter what, we need to insert this donation record into the db
    // so we'll make one upfront
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("beginning transaction")?;
    let id = generate_campaign_donation_id(&mut transaction).await?;

    let mut donation = CampaignDonation {
        id,
        raw_data: raw_payload,
        donated_at: payload.meta.generated_at,
        amount_usd: None,
        user_id: None,
    };

    let username = async {
        // then we can attempt user lookups
        let username = payload.data.user.username;
        let user = DBUser::get(&username, &**pool, &redis)
            .await
            .wrap_err("fetching user from database")?
            .wrap_err_with(|| {
                eyre!("got donation for user '{username}' which does not exist")
            })?;

        donation.user_id = Some(user.id);
        eyre::Ok(username)
    }
    .await
    .inspect_err(|err| {
        warn!("Failed to resolve donation to Modrinth user: {err:?}")
    })
    .ok();

    let amount_usd = async {
        // and insert value amount
        let amount_usd =
            amount_raised_usd(&payload.data.amount_raised, &payouts_queue)
                .await
                .wrap_err("failed to get donation amount")?;

        donation.amount_usd = Some(amount_usd);
        eyre::Ok(amount_usd)
    }
    .await
    .inspect_err(|err| warn!("Failed to resolve donation amount: {err:?}"))
    .ok();

    info!(
        "Resolved donation from {} for US${}",
        username.as_deref().unwrap_or("<unknown>"),
        amount_usd
            .map(|a| a.to_string())
            .unwrap_or_else(|| "<unknown>".to_string())
    );

    donation
        .insert(&mut transaction)
        .await
        .wrap_internal_err("inserting donation")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing transaction")?;

    if let Some(user_id) = donation.user_id {
        DBUser::clear_caches(&[(user_id, username)], &redis)
            .await
            .wrap_internal_err("clearing user caches")?;
    }

    Ok(())
}

async fn amount_raised_usd(
    amount: &AmountRaised,
    payouts_queue: &PayoutsQueue,
) -> Result<Decimal, ApiError> {
    let currency = amount.currency.to_uppercase();

    if currency == "USD" {
        return Ok(amount.value);
    }

    let forex: TremendousForexResponse = payouts_queue
        .make_tremendous_request(Method::GET, "forex", None::<()>)
        .await
        .wrap_internal_err("failed to fetch Tremendous forex data")?;

    let usd_to_currency = forex
        .forex
        .get(&currency)
        .copied()
        .wrap_internal_err_with(|| {
            eyre!("no Tremendous forex rate for '{currency}'")
        })?;

    Ok(amount.value / usd_to_currency)
}
