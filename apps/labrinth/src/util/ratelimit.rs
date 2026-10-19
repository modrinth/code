use crate::auth::validate::extract_authorization_header;
use crate::database::models::DBUserId;
use crate::database::models::pat_item::DBPersonalAccessToken;
use crate::database::models::session_item::DBSession;
use crate::database::models::user_limits::get_user_requests_per_minute;
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::env::ENV;
use crate::routes::ApiError;
use actix_web::{
    Error, ResponseError,
    body::{EitherBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web,
};
use chrono::Utc;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};
use tracing::warn;
use xredis::RedisPool;

const RATE_LIMIT_NAMESPACE: &str = "rate_limit:v4";
const RATE_LIMIT_EXPIRY: i64 = 300; // 5 minutes
const MINUTE_IN_NANOS: i64 = 60_000_000_000;
const REDIS_WARNING_INTERVAL_SECS: i64 = 10;

static LAST_REDIS_WARNING_SECS: AtomicI64 = AtomicI64::new(0);

/// Allows at most one Redis warning per interval across all requests, so a
/// Redis outage at high request rates does not flood the logs.
fn should_log_redis_warning() -> bool {
    let now = Utc::now().timestamp();
    let last = LAST_REDIS_WARNING_SECS.load(Ordering::Relaxed);
    now - last >= REDIS_WARNING_INTERVAL_SECS
        && LAST_REDIS_WARNING_SECS
            .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
}

#[derive(Clone, Copy)]
pub struct GCRAParameters {
    emission_interval: i64,
    burst_size: u32,
}

impl GCRAParameters {
    pub(crate) fn new(requests_per_minute: u32, burst_size: u32) -> Self {
        // Calculate emission interval in nanoseconds
        let emission_interval = MINUTE_IN_NANOS / requests_per_minute as i64;

        Self {
            emission_interval,
            burst_size,
        }
    }
}

pub struct RateLimitDecision {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset_after_ms: i64,
    pub retry_after_ms: Option<i64>,
}

#[derive(Clone)]
pub struct AsyncRateLimiter {
    redis_pool: RedisPool,
    default_params: GCRAParameters,
}

impl AsyncRateLimiter {
    pub fn new(redis_pool: RedisPool, default_params: GCRAParameters) -> Self {
        Self {
            redis_pool,
            default_params,
        }
    }

    pub async fn check_rate_limit(
        &self,
        key: &str,
        params: GCRAParameters,
    ) -> RateLimitDecision {
        let mut conn = match self.redis_pool.connect().await {
            Ok(conn) => conn,
            Err(err) => {
                if should_log_redis_warning() {
                    warn!(
                        "Rate limiter could not connect to Redis, allowing request: {err:#}"
                    );
                }
                // If Redis is unavailable, allow the request but with reduced limit
                return RateLimitDecision {
                    allowed: true,
                    limit: params.burst_size,
                    remaining: 1,
                    reset_after_ms: 60_000, // 1 minute
                    retry_after_ms: None,
                };
            }
        };

        // Get current time in nanoseconds since UNIX epoch
        let now = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let key = conn.key().with_slot(RATE_LIMIT_NAMESPACE, key, key);

        // Get the current TAT from Redis (if it exists)
        let tat_str = conn
            .get(&key)
            .await
            .inspect_err(|err| {
                if should_log_redis_warning() {
                    warn!("Rate limiter failed to read from Redis: {err:#}");
                }
            })
            .ok()
            .flatten();

        // Parse the TAT or use current time if not found
        let current_tat = match tat_str {
            Some(tat_str) => tat_str.parse::<i64>().unwrap_or(now),
            None => now,
        };

        // Calculate the new TAT using GCRA
        let increment = params.emission_interval;
        let max_tat_delta = increment * params.burst_size as i64;

        // Calculate allowance: how much time has passed since the TAT
        let allowance = now - current_tat;

        if allowance < -max_tat_delta {
            // Too many requests, rate limit exceeded
            // Calculate when the client can retry
            let retry_after_ms = (-allowance - max_tat_delta) / 1_000_000;

            return RateLimitDecision {
                allowed: false,
                limit: params.burst_size,
                remaining: 0,
                reset_after_ms: -allowance / 1_000_000,
                retry_after_ms: Some(retry_after_ms.max(0)),
            };
        }

        let new_tat = std::cmp::max(current_tat + increment, now);

        if let Err(err) = conn
            .set(&key, &new_tat.to_string(), Some(RATE_LIMIT_EXPIRY))
            .await
            && should_log_redis_warning()
        {
            warn!("Rate limiter failed to write to Redis: {err:#}");
        }

        let remaining_capacity =
            ((max_tat_delta - (new_tat - now)) / increment).max(0) as u32;

        RateLimitDecision {
            allowed: true,
            limit: params.burst_size,
            remaining: remaining_capacity,
            reset_after_ms: (new_tat - now) / 1_000_000,
            retry_after_ms: None,
        }
    }
}

/// Resolves the request's session or PAT to a user with a rate limit override.
///
/// Tokens are only looked up in Redis, so unknown or cold tokens fall through
/// to the IP limiter without touching Postgres. The handler warms the cache for
/// valid tokens. OAuth tokens are skipped because their lookup is not cached.
async fn get_user_rate_limit(
    req: &ServiceRequest,
) -> eyre::Result<Option<(DBUserId, u32)>> {
    let Ok(token) = extract_authorization_header(req.request()) else {
        return Ok(None);
    };

    let pool: &PgPool = req
        .app_data::<web::Data<ReadOnlyPgPool>>()
        .expect("Read-only Postgres pool not configured properly");
    let redis = req
        .app_data::<web::Data<RedisPool>>()
        .expect("Redis pool not configured properly");

    let user_id = match token.split_once('_') {
        Some(("mrp", _)) => DBPersonalAccessToken::get_cached(token, redis)
            .await?
            .filter(|pat| pat.expires > Utc::now())
            .map(|pat| pat.user_id),
        Some(("mra", _)) => DBSession::get_cached(token, redis)
            .await?
            .filter(|session| session.expires > Utc::now())
            .map(|session| session.user_id),
        _ => None,
    };
    let Some(user_id) = user_id else {
        return Ok(None);
    };

    let requests_per_minute =
        get_user_requests_per_minute(user_id, pool, redis).await?;
    Ok(requests_per_minute.map(|rpm| (user_id, rpm)))
}

pub async fn rate_limit_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, Error> {
    let rate_limiter = req
        .app_data::<web::Data<AsyncRateLimiter>>()
        .expect("Rate limiter not configured properly")
        .clone();

    if let Some(key) = req.headers().get("x-ratelimit-key")
        && key.to_str().ok() == Some(&ENV.RATE_LIMIT_IGNORE_KEY)
    {
        return Ok(next.call(req).await?.map_into_left_body());
    }

    let conn_info = req.connection_info().clone();
    let ip = if ENV.CLOUDFLARE_INTEGRATION {
        if let Some(header) = req.headers().get("CF-Connecting-IP") {
            header.to_str().ok()
        } else {
            conn_info.peer_addr()
        }
    } else {
        conn_info.peer_addr()
    };

    let user_rate_limit =
        get_user_rate_limit(&req).await.unwrap_or_else(|err| {
            if should_log_redis_warning() {
                warn!("Failed to resolve user rate limit, using IP: {err:#}");
            }
            None
        });

    let rate_limit_target = match user_rate_limit {
        Some((user_id, requests_per_minute)) => Some((
            format!("user:{}", user_id.0),
            GCRAParameters::new(requests_per_minute, requests_per_minute),
        )),
        None => ip.map(|ip| (ip.to_string(), rate_limiter.default_params)),
    };

    if let Some((key, params)) = rate_limit_target {
        let started_at = Instant::now();
        let decision = rate_limiter.check_rate_limit(&key, params).await;
        let elapsed = started_at.elapsed();
        if elapsed > Duration::from_secs(1) && should_log_redis_warning() {
            warn!(
                elapsed_ms = elapsed.as_millis() as u64,
                path = req.path(),
                "Slow rate limit check against Redis"
            );
        }

        if decision.allowed {
            let mut service_response = next.call(req).await?;

            // Add rate limit headers
            let headers = service_response.headers_mut();
            headers.insert(
                actix_web::http::header::HeaderName::from_str(
                    "x-ratelimit-limit",
                )
                .unwrap(),
                decision.limit.into(),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_str(
                    "x-ratelimit-remaining",
                )
                .unwrap(),
                decision.remaining.into(),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_str(
                    "x-ratelimit-reset",
                )
                .unwrap(),
                (decision.reset_after_ms / 1000).into(),
            );

            Ok(service_response.map_into_left_body())
        } else {
            let retry_after_ms = decision.retry_after_ms.unwrap_or(0);
            let mut response = ApiError::RateLimit(eyre::eyre!(
                "rate limit exceeded; retry after {retry_after_ms} milliseconds"
            ))
            .error_response();

            // Add rate limit headers
            let headers = response.headers_mut();
            headers.insert(
                actix_web::http::header::HeaderName::from_str(
                    "x-ratelimit-limit",
                )
                .unwrap(),
                decision.limit.into(),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_str(
                    "x-ratelimit-remaining",
                )
                .unwrap(),
                0.into(),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_str(
                    "x-ratelimit-reset",
                )
                .unwrap(),
                (decision.reset_after_ms / 1000).into(),
            );

            // TODO: Centralize CORS in the CORS util.
            headers.insert(
                actix_web::http::header::HeaderName::from_str(
                    "Access-Control-Allow-Origin",
                )
                .unwrap(),
                "*".parse().unwrap(),
            );

            Ok(req.into_response(response.map_into_right_body()))
        }
    } else {
        let response =
            ApiError::Auth(eyre::eyre!("Unable to obtain user IP address!",))
                .error_response();

        Ok(req.into_response(response.map_into_right_body()))
    }
}
