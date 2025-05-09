use crate::database::redis::RedisPool;
use crate::routes::ApiError;
use crate::util::env::parse_var;
use actix_web::{
    Error, ResponseError,
    body::{EitherBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web,
};
use chrono::Utc;
use std::str::FromStr;
use std::sync::Arc;

const RATE_LIMIT_NAMESPACE: &str = "rate_limit";
const RATE_LIMIT_EXPIRY: i64 = 300; // 5 minutes
const MINUTE_IN_NANOS: i64 = 60_000_000_000;

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
    params: Arc<GCRAParameters>,
}

impl AsyncRateLimiter {
    pub fn new(redis_pool: RedisPool, params: GCRAParameters) -> Self {
        Self {
            redis_pool,
            params: Arc::new(params),
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> RateLimitDecision {
        let mut conn = match self.redis_pool.connect().await {
            Ok(conn) => conn,
            Err(_) => {
                // If Redis is unavailable, allow the request but with reduced limit
                return RateLimitDecision {
                    allowed: true,
                    limit: self.params.burst_size,
                    remaining: 1,
                    reset_after_ms: 60_000, // 1 minute
                    retry_after_ms: None,
                };
            }
        };

        // Get current time in nanoseconds since UNIX epoch
        let now = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Get the current TAT from Redis (if it exists)
        let tat_str = conn.get(RATE_LIMIT_NAMESPACE, key).await.ok().flatten();

        // Parse the TAT or use current time if not found
        let current_tat = match tat_str {
            Some(tat_str) => tat_str.parse::<i64>().unwrap_or(now),
            None => now,
        };

        // Calculate the new TAT using GCRA
        let increment = self.params.emission_interval;
        let max_tat_delta = increment * self.params.burst_size as i64;

        // Calculate allowance: how much time has passed since the TAT
        let allowance = now - current_tat;

        if allowance < -max_tat_delta {
            // Too many requests, rate limit exceeded
            // Calculate when the client can retry
            let retry_after_ms = (-allowance - max_tat_delta) / 1_000_000;

            return RateLimitDecision {
                allowed: false,
                limit: self.params.burst_size,
                remaining: 0,
                reset_after_ms: -allowance / 1_000_000,
                retry_after_ms: Some(retry_after_ms.max(0)),
            };
        }

        let new_tat = std::cmp::max(current_tat + increment, now);

        let _ = conn
            .set(
                RATE_LIMIT_NAMESPACE,
                key,
                &new_tat.to_string(),
                Some(RATE_LIMIT_EXPIRY),
            )
            .await;

        let remaining_capacity =
            ((max_tat_delta - (new_tat - now)) / increment).max(0) as u32;

        RateLimitDecision {
            allowed: true,
            limit: self.params.burst_size,
            remaining: remaining_capacity,
            reset_after_ms: (new_tat - now) / 1_000_000,
            retry_after_ms: None,
        }
    }
}

pub async fn rate_limit_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, Error> {
    let rate_limiter = req
        .app_data::<web::Data<AsyncRateLimiter>>()
        .expect("Rate limiter not configured properly")
        .clone();

    if let Some(key) = req.headers().get("x-ratelimit-key") {
        if key.to_str().ok()
            == dotenvy::var("RATE_LIMIT_IGNORE_KEY").ok().as_deref()
        {
            return Ok(next.call(req).await?.map_into_left_body());
        }
    }

    let conn_info = req.connection_info().clone();
    let ip = if parse_var("CLOUDFLARE_INTEGRATION").unwrap_or(false) {
        if let Some(header) = req.headers().get("CF-Connecting-IP") {
            header.to_str().ok()
        } else {
            conn_info.peer_addr()
        }
    } else {
        conn_info.peer_addr()
    };

    if let Some(ip) = ip {
        let decision = rate_limiter.check_rate_limit(ip).await;

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
            let mut response = ApiError::RateLimitError(
                decision.retry_after_ms.unwrap_or(0) as u128,
                decision.limit,
            )
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
        let response = ApiError::CustomAuthentication(
            "Unable to obtain user IP address!".to_string(),
        )
        .error_response();

        Ok(req.into_response(response.map_into_right_body()))
    }
}
