use actix_web::{get, post, web};
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
    env::ENV,
    models::payouts::TremendousForexResponse,
    queue::payouts::PayoutsQueue,
    routes::ApiError,
    util::{error::Context, http::HttpClient, tiltify::TiltifyClient},
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(tiltify_webhook).service(pride_26);
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

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CampaignInfo {
    pub total_donations_usd: Decimal,
    pub target_usd: Decimal,
}

#[derive(Debug, Deserialize)]
struct TiltifyCampaignResponse {
    data: TiltifyCampaign,
}

#[derive(Debug, Deserialize)]
struct TiltifyCampaign {
    goal: AmountRaised,
    total_amount_raised: AmountRaised,
}

pub struct CampaignDonation {
    pub id: DBCampaignDonationId,
    pub tiltify_event_id: Uuid,
    pub raw_data: serde_json::Value,
    pub donated_at: DateTime<Utc>,
    pub amount_usd: Option<Decimal>,
    pub user_id: Option<DBUserId>,
}

impl CampaignDonation {
    pub async fn insert(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<bool, ApiError> {
        let user_id = self.user_id.map(|id| id.0);
        let inserted = sqlx::query!(
            "
            insert into campaign_donations (id, tiltify_event_id, raw_data, donated_at, amount_usd, user_id)
            values ($1, $2::text::uuid, $3, $4, $5, $6)
            on conflict (tiltify_event_id) do nothing
            returning id
            ",
            self.id.0,
            self.tiltify_event_id.to_string(),
            self.raw_data,
            self.donated_at,
            self.amount_usd,
            user_id,
        )
        .fetch_optional(transaction)
        .await
        .wrap_internal_err("inserting campaign donation")?;

        Ok(inserted.is_some())
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
        tiltify_event_id: payload.meta.id,
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

    let inserted = donation
        .insert(&mut transaction)
        .await
        .wrap_internal_err("inserting donation")?;

    if !inserted {
        transaction
            .commit()
            .await
            .wrap_internal_err("committing duplicate donation transaction")?;
        info!("Ignoring duplicate Tiltify webhook {}", payload.meta.id);
        return Ok(());
    }

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

#[utoipa::path]
#[get("/pride-26")]
pub async fn pride_26(
    http: web::Data<HttpClient>,
    tiltify: web::Data<TiltifyClient>,
) -> Result<web::Json<CampaignInfo>, ApiError> {
    let access_token = tiltify
        .access_token()
        .await
        .wrap_internal_err("fetching Tiltify access token")?;
    let url = format!(
        "https://v5api.tiltify.com/api/public/team_campaigns/{}",
        ENV.TILTIFY_PRIDE_26_CAMPAIGN_ID
    );
    info!("at = {access_token}");
    let response = http
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .wrap_internal_err("fetching campaign from Tiltify")?
        .error_for_status()
        .wrap_internal_err("fetching campaign from Tiltify")?
        .json::<TiltifyCampaignResponse>()
        .await
        .wrap_internal_err("parsing Tiltify response")?;

    let raised_currency = &response.data.total_amount_raised.currency;
    if raised_currency != "USD" {
        return Err(ApiError::Internal(eyre!(
            "total amount raised is in {raised_currency}, must be USD"
        )));
    }

    let goal_currency = &response.data.goal.currency;
    if goal_currency != "USD" {
        return Err(ApiError::Internal(eyre!(
            "goal amount is in {goal_currency}, must be USD"
        )));
    }

    Ok(web::Json(CampaignInfo {
        total_donations_usd: response.data.total_amount_raised.value,
        target_usd: response.data.goal.value,
    }))
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
