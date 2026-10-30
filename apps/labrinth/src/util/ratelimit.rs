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
use redis::Script;
use std::str::FromStr;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};
use tracing::warn;
use xredis::RedisPool;

const RATE_LIMIT_NAMESPACE: &str = "rate_limit:v5";
const RATE_LIMIT_EXPIRY: i64 = 300; // 5 minutes
const MINUTE_IN_MICROS: i64 = 60_000_000;
const REDIS_WARNING_INTERVAL_SECS: i64 = 10;

/// Runs one GCRA step atomically so concurrent requests cannot read the same
/// TAT. Times are in microseconds from the Redis clock: Lua numbers are
/// doubles, so nanoseconds since epoch would lose precision, and using one
/// clock keeps replicas with skewed clocks consistent.
///
/// Returns `{allowed, remaining, reset_after_us, retry_after_us}`.
const GCRA_LUA: &str = r#"
local emission_interval = tonumber(ARGV[1])
local burst_size = tonumber(ARGV[2])
local expiry = tonumber(ARGV[3])

local time = redis.call('TIME')
local now = tonumber(time[1]) * 1000000 + tonumber(time[2])

local tat = tonumber(redis.call('GET', KEYS[1])) or now
local max_tat_delta = emission_interval * burst_size

if tat - now > max_tat_delta then
    return {0, 0, tat - now, tat - now - max_tat_delta}
end

local new_tat = math.max(tat + emission_interval, now)
redis.call('SET', KEYS[1], string.format('%.0f', new_tat), 'EX', expiry)

local remaining = math.floor((max_tat_delta - (new_tat - now)) / emission_interval)
return {1, math.max(remaining, 0), new_tat - now, 0}
"#;

static GCRA_SCRIPT: LazyLock<Script> = LazyLock::new(|| Script::new(GCRA_LUA));

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
        let emission_interval = MINUTE_IN_MICROS / requests_per_minute as i64;

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
                return Self::fail_open(params);
            }
        };

        let key = conn.key().with_slot(RATE_LIMIT_NAMESPACE, key, key);

        let (allowed, remaining, reset_after_us, retry_after_us): (
            bool,
            u32,
            i64,
            i64,
        ) = match GCRA_SCRIPT
            .key(&key)
            .arg(params.emission_interval)
            .arg(params.burst_size)
            .arg(RATE_LIMIT_EXPIRY)
            .invoke_async(&mut conn)
            .await
        {
            Ok(result) => result,
            Err(err) => {
                if should_log_redis_warning() {
                    warn!(
                        "Rate limiter script failed, allowing request: {err:#}"
                    );
                }
                return Self::fail_open(params);
            }
        };

        RateLimitDecision {
            allowed,
            limit: params.burst_size,
            remaining,
            reset_after_ms: reset_after_us / 1_000,
            retry_after_ms: (!allowed).then_some(retry_after_us / 1_000),
        }
    }

    /// Allows the request when Redis is unavailable, with a reduced limit.
    fn fail_open(params: GCRAParameters) -> RateLimitDecision {
        RateLimitDecision {
            allowed: true,
            limit: params.burst_size,
            remaining: 1,
            reset_after_ms: 60_000,
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
