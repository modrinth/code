use actix_web::{get, post, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    models::{payouts::TremendousForexResponse, users::Badges},
    queue::payouts::PayoutsQueue,
    routes::ApiError,
    util::{error::Context, http::HttpClient, tiltify::TiltifyClient},
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(tiltify_webhook).service(pride_26);
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
struct TiltifyWebhook {
    data: TiltifyData,
    meta: TiltifyMeta,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
struct TiltifyData {
    amount: AmountRaised,
    donor_name: String,
    completed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
struct AmountRaised {
    currency: String,
    value: Decimal,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
struct TiltifyMeta {
    attempted_at: DateTime<Utc>,
    event_type: String,
    generated_at: DateTime<Utc>,
    id: Uuid,
    subscription_source_id: Uuid,
    subscription_source_type: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CampaignInfo {
    total_donations_usd: Decimal,
    target_usd: Decimal,
    num_donators: usize,
}

const CAMPAIGN_INFO_CACHE_NAMESPACE: &str = "campaign_info";
const CAMPAIGN_INFO_CACHE_TTL_SECONDS: i64 = 15 * 60;

#[derive(Debug, Deserialize)]
struct TiltifyCampaignResponse {
    data: TiltifyCampaign,
}

#[derive(Debug, Deserialize)]
struct TiltifyCampaign {
    goal: AmountRaised,
    total_amount_raised: AmountRaised,
}

#[derive(Debug, Deserialize)]
struct TiltifyDonationResponse {
    data: Vec<TiltifyDonation>,
    metadata: TiltifyPaginationMetadata,
}

#[derive(Debug, Deserialize)]
struct TiltifyDonation {
    donor_name: String,
}

#[derive(Debug, Deserialize)]
struct TiltifyPaginationMetadata {
    after: Option<String>,
}

struct CampaignDonation {
    id: DBCampaignDonationId,
    tiltify_event_id: Uuid,
    raw_data: serde_json::Value,
    donated_at: DateTime<Utc>,
    amount_usd: Option<Decimal>,
    user_id: Option<DBUserId>,
}

impl CampaignDonation {
    async fn insert(
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
        donated_at: payload.data.completed_at,
        amount_usd: None,
        user_id: None,
    };

    let username = async {
        // then we can attempt user lookups
        let username = payload.data.donor_name.clone();
        let user = DBUser::get(&username, &**pool, &redis)
            .await
            .wrap_err("fetching user from database")?
            .wrap_err_with(|| {
                eyre!("got donation for user '{username}' which does not exist")
            })?;

        donation.user_id = Some(user.id);
        eyre::Ok((username, user.badges))
    }
    .await
    .inspect_err(|err| {
        warn!("Failed to resolve donation to Modrinth user: {err:?}")
    })
    .ok();

    let amount_usd = async {
        // and insert value amount
        let amount_usd =
            amount_raised_usd(&payload.data.amount, &payouts_queue)
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
        username
            .as_ref()
            .map(|(username, _)| username.as_str())
            .unwrap_or("<unknown>"),
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

    if let (Some(user_id), Some((_, badges)), Some(amount_usd)) =
        (donation.user_id, username.as_ref(), donation.amount_usd)
        && amount_usd >= Decimal::ONE
    {
        let badges = *badges | Badges::PRIDE_2026;

        sqlx::query!(
            "
            UPDATE users
            SET badges = $1
            WHERE (id = $2)
            ",
            badges.bits() as i64,
            user_id.0,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("updating user campaign badge")?;
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing transaction")?;

    if let Some(user_id) = donation.user_id {
        DBUser::clear_caches(
            &[(user_id, username.map(|(username, _)| username))],
            &redis,
        )
        .await
        .wrap_internal_err("clearing user caches")?;
    }

    Ok(())
}

#[utoipa::path]
#[get("/pride-26")]
pub async fn pride_26(
    http: web::Data<HttpClient>,
    redis: web::Data<RedisPool>,
    tiltify: web::Data<TiltifyClient>,
) -> Result<web::Json<CampaignInfo>, ApiError> {
    let campaign_id = &ENV.TILTIFY_PRIDE_26_CAMPAIGN_ID;
    let mut redis_connection = redis
        .connect()
        .await
        .wrap_internal_err("connecting to redis")?;

    if let Some(cached) = redis_connection
        .get(CAMPAIGN_INFO_CACHE_NAMESPACE, campaign_id)
        .await
        .wrap_internal_err("getting cached campaign info")?
    {
        let campaign_info = serde_json::from_str::<CampaignInfo>(&cached)
            .wrap_internal_err("parsing cached campaign info")?;
        return Ok(web::Json(campaign_info));
    }

    let access_token = tiltify
        .access_token()
        .await
        .wrap_internal_err("fetching Tiltify access token")?;
    let url = format!(
        "https://v5api.tiltify.com/api/public/team_campaigns/{campaign_id}",
    );
    let response = http
        .get(url)
        .bearer_auth(&access_token)
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

    let campaign_info = CampaignInfo {
        total_donations_usd: response.data.total_amount_raised.value,
        target_usd: response.data.goal.value,
        num_donators: num_donators(&http, &access_token, campaign_id).await?,
    };

    redis_connection
        .set_serialized_to_json(
            CAMPAIGN_INFO_CACHE_NAMESPACE,
            campaign_id,
            &campaign_info,
            Some(CAMPAIGN_INFO_CACHE_TTL_SECONDS),
        )
        .await
        .wrap_internal_err("caching campaign info")?;

    Ok(web::Json(campaign_info))
}

async fn num_donators(
    http: &HttpClient,
    access_token: &str,
    campaign_id: &str,
) -> Result<usize, ApiError> {
    let mut after = None;
    let mut donors = HashSet::new();

    loop {
        let url = format!(
            "https://v5api.tiltify.com/api/public/team_campaigns/{campaign_id}/donations"
        );
        let mut request = http
            .get(url)
            .bearer_auth(access_token)
            .query(&[("limit", "100")]);

        if let Some(after) = &after {
            request = request.query(&[("after", after)]);
        }

        let response = request
            .send()
            .await
            .wrap_internal_err("fetching donations from Tiltify")?
            .error_for_status()
            .wrap_internal_err("fetching donations from Tiltify")?
            .json::<TiltifyDonationResponse>()
            .await
            .wrap_internal_err("parsing Tiltify donations response")?;

        donors.extend(
            response
                .data
                .into_iter()
                .map(|donation| donation.donor_name)
                .filter(|donor_name| donor_name != "Anonymous"),
        );

        match response.metadata.after {
            Some(next_after) => after = Some(next_after),
            None => break,
        }
    }

    Ok(donors.len())
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
