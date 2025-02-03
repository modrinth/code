use governor::clock::{Clock, DefaultClock};
use governor::{middleware, state, RateLimiter};
use std::str::FromStr;
use std::sync::Arc;

use crate::routes::ApiError;
use crate::util::env::parse_var;
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, ResponseError,
};
use futures_util::future::LocalBoxFuture;
use futures_util::future::{ready, Ready};

pub type KeyedRateLimiter<
    K = String,
    MW = middleware::StateInformationMiddleware,
> = Arc<
    RateLimiter<K, state::keyed::DefaultKeyedStateStore<K>, DefaultClock, MW>,
>;

pub struct RateLimit(pub KeyedRateLimiter);

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RateLimitService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitService {
            service,
            rate_limiter: Arc::clone(&self.0),
        }))
    }
}

#[doc(hidden)]
pub struct RateLimitService<S> {
    service: S,
    rate_limiter: KeyedRateLimiter,
}

impl<S, B> Service<ServiceRequest> for RateLimitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(key) = req.headers().get("x-ratelimit-key") {
            if key.to_str().ok()
                == dotenvy::var("RATE_LIMIT_IGNORE_KEY").ok().as_deref()
            {
                let res = self.service.call(req);

                return Box::pin(async move {
                    let service_response = res.await?;
                    Ok(service_response.map_into_left_body())
                });
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
            let ip = ip.to_string();

            match self.rate_limiter.check_key(&ip) {
                Ok(snapshot) => {
                    let fut = self.service.call(req);

                    Box::pin(async move {
                        match fut.await {
                            Ok(mut service_response) => {
                                // Now you have a mutable reference to the ServiceResponse, so you can modify its headers.
                                let headers = service_response.headers_mut();
                                headers.insert(
                                    actix_web::http::header::HeaderName::from_str(
                                        "x-ratelimit-limit",
                                    )
                                    .unwrap(),
                                    snapshot.quota().burst_size().get().into(),
                                );
                                headers.insert(
                                    actix_web::http::header::HeaderName::from_str(
                                        "x-ratelimit-remaining",
                                    )
                                    .unwrap(),
                                    snapshot.remaining_burst_capacity().into(),
                                );

                                headers.insert(
                                    actix_web::http::header::HeaderName::from_str(
                                        "x-ratelimit-reset",
                                    )
                                    .unwrap(),
                                    snapshot
                                        .quota()
                                        .burst_size_replenished_in()
                                        .as_secs()
                                        .into(),
                                );

                                // Return the modified response as Ok.
                                Ok(service_response.map_into_left_body())
                            }
                            Err(e) => {
                                // Handle error case
                                Err(e)
                            }
                        }
                    })
                }
                Err(negative) => {
                    let wait_time =
                        negative.wait_time_from(DefaultClock::default().now());

                    let mut response = ApiError::RateLimitError(
                        wait_time.as_millis(),
                        negative.quota().burst_size().get(),
                    )
                    .error_response();

                    let headers = response.headers_mut();

                    headers.insert(
                        actix_web::http::header::HeaderName::from_str(
                            "x-ratelimit-limit",
                        )
                        .unwrap(),
                        negative.quota().burst_size().get().into(),
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
                        wait_time.as_secs().into(),
                    );

                    // TODO: Sentralize CORS in the CORS util.
                    headers.insert(
                        actix_web::http::header::HeaderName::from_str(
                            "Access-Control-Allow-Origin",
                        )
                        .unwrap(),
                        "*".parse().unwrap(),
                    );

                    Box::pin(async {
                        Ok(req.into_response(response.map_into_right_body()))
                    })
                }
            }
        } else {
            let response = ApiError::CustomAuthentication(
                "Unable to obtain user IP address!".to_string(),
            )
            .error_response();

            Box::pin(async {
                Ok(req.into_response(response.map_into_right_body()))
            })
        }
    }
}
