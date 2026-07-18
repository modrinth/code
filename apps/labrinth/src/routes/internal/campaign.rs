use actix_web::{HttpRequest, get, post, web};
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use eyre::eyre;
use hmac::{Hmac, Mac};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashSet;
use tracing::{debug, info, warn};
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

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
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

#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CampaignInfo {
    total_donations_usd: Decimal,
    target_usd: Decimal,
    num_donators: usize,
    cached_at: DateTime<Utc>,
}

const CAMPAIGN_INFO_CACHE_NAMESPACE: &str = "campaign_info:v1";
const CAMPAIGN_INFO_CACHE_STALE_SECONDS: i64 = 15 * 60;
const CAMPAIGN_INFO_CACHE_TTL_SECONDS: i64 = 24 * 60 * 60;

impl CampaignInfo {
    fn is_stale(&self) -> bool {
        Utc::now().signed_duration_since(self.cached_at)
            >= Duration::seconds(CAMPAIGN_INFO_CACHE_STALE_SECONDS)
    }
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

/// Receive a Tiltify webhook.  
#[utoipa::path(
	context_path = "/campaign",
	tag = "campaigns",
	request_body(content = String, content_type = "text/plain"),
	responses((status = NO_CONTENT))
)]
#[post("/webhook")]
pub async fn tiltify_webhook(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    payouts_queue: web::Data<PayoutsQueue>,
    body: String,
) -> Result<(), ApiError> {
    verify_tiltify_webhook_signature(&req, &body)?;

    let raw_payload = serde_json::from_str::<serde_json::Value>(&body)
        .wrap_internal_err_with(|| eyre!("invalid Tiltify webhook JSON"))?;

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

fn verify_tiltify_webhook_signature(
    req: &HttpRequest,
    body: &str,
) -> Result<(), ApiError> {
    let signature = req
        .headers()
        .get("X-Tiltify-Signature")
        .and_then(|x| x.to_str().ok())
        .wrap_request_err("missing Tiltify webhook signature")?;
    let signature = base64::engine::general_purpose::STANDARD
        .decode(signature)
        .wrap_request_err("invalid Tiltify webhook signature")?;

    let timestamp = req
        .headers()
        .get("X-Tiltify-Timestamp")
        .and_then(|x| x.to_str().ok())
        .wrap_request_err("missing Tiltify webhook timestamp")?;
    let parsed_timestamp = DateTime::parse_from_rfc3339(timestamp)
        .wrap_request_err("invalid Tiltify webhook timestamp")?;
    let parsed_timestamp = parsed_timestamp.with_timezone(&Utc);
    let age = Utc::now().signed_duration_since(parsed_timestamp);
    if age < -Duration::minutes(1) || age > Duration::minutes(1) {
        return Err(ApiError::Request(eyre!(
            "expired Tiltify webhook timestamp",
        )));
    }

    if ENV.TILTIFY_WEBHOOK_SIGNING_KEY.is_empty() {
        return Err(ApiError::Internal(eyre!(
            "TILTIFY_WEBHOOK_SIGNING_KEY must be set"
        )));
    }

    let mut mac: Hmac<Sha256> =
        Hmac::new_from_slice(ENV.TILTIFY_WEBHOOK_SIGNING_KEY.as_bytes())
            .wrap_internal_err("initializing Tiltify webhook HMAC")?;
    mac.update(timestamp.as_bytes());
    mac.update(b".");
    mac.update(body.as_bytes());
    mac.verify_slice(&signature)
        .wrap_request_err("invalid Tiltify webhook signature")?;

    Ok(())
}

/// Get Pride campaign data.  
#[utoipa::path(
	context_path = "/campaign",
	tag = "campaigns",
	responses((status = OK, body = CampaignInfo))
)]
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

    let cached = redis_connection
        .get_deserialized::<CampaignInfo>(
            CAMPAIGN_INFO_CACHE_NAMESPACE,
            campaign_id,
        )
        .await
        .wrap_internal_err("getting cached campaign info")?;

    if let Some(cached) = &cached
        && !cached.is_stale()
    {
        return Ok(web::Json(cached.clone()));
    }

    let result = async {
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
            num_donators: num_donators(&http, &access_token, campaign_id)
                .await?,
            cached_at: Utc::now(),
        };

        redis_connection
            .set_serialized(
                CAMPAIGN_INFO_CACHE_NAMESPACE,
                campaign_id,
                &campaign_info,
                Some(CAMPAIGN_INFO_CACHE_TTL_SECONDS),
            )
            .await
            .wrap_internal_err("caching campaign info")?;

        Ok(campaign_info)
    }
    .await;

    match result {
        Ok(campaign_info) => Ok(web::Json(campaign_info)),
        Err(error) => {
            if let Some(cached) = cached {
                debug!(
                    "Failed to refresh campaign info from Tiltify: {error:?}"
                );
                Ok(web::Json(cached))
            } else {
                Err(error)
            }
        }
    }
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
