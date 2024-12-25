use governor::clock::{Clock, DefaultClock};
use governor::{middleware, state, RateLimiter};
use std::str::FromStr;
use std::sync::Arc;

use crate::routes::ApiError;
use crate::util::env::parse_var;
use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web;
use ntex::web::{WebResponse, WebResponseError};

pub type KeyedRateLimiter<
    K = String,
    MW = middleware::StateInformationMiddleware,
> = Arc<
    RateLimiter<K, state::keyed::DefaultKeyedStateStore<K>, DefaultClock, MW>,
>;

pub struct RateLimit(pub KeyedRateLimiter);

impl<S> Middleware<S> for RateLimit {
    type Service = RateLimitService<S>;

    fn create(&self, service: S) -> Self::Service {
        RateLimitService {
            service,
            rate_limiter: Arc::clone(&self.0),
        }
    }
}

#[doc(hidden)]
pub struct RateLimitService<S> {
    service: S,
    rate_limiter: KeyedRateLimiter,
}

impl<S, Err> Service<web::WebRequest<Err>> for RateLimitService<S>
where
    S: Service<
        web::WebRequest<Err>,
        Response = web::WebResponse,
        Error = web::Error,
    >,
    Err: web::ErrorRenderer,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        req: web::WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        if let Some(key) = req.headers().get("x-ratelimit-key") {
            if key.to_str().ok()
                == dotenvy::var("RATE_LIMIT_IGNORE_KEY").ok().as_deref()
            {
                let res = ctx.call(&self.service, req).await?;
                return Ok(res);
            }
        }

        let ip = if parse_var("CLOUDFLARE_INTEGRATION").unwrap_or(false) {
            if let Some(header) = req.headers().get("CF-Connecting-IP") {
                header.to_str().ok().map(|x| x.to_string())
            } else {
                req.peer_addr().map(|x| x.to_string())
            }
        } else {
            req.peer_addr().map(|x| x.to_string())
        };

        if let Some(ip) = ip {
            let ip = ip.to_string();

            match self.rate_limiter.check_key(&ip) {
                Ok(snapshot) => {
                    let mut service_response =
                        ctx.call(&self.service, req).await?;

                    let headers = service_response.headers_mut();
                    headers.insert(
                        ntex::http::header::HeaderName::from_str(
                            "x-ratelimit-limit",
                        )
                        .unwrap(),
                        snapshot.quota().burst_size().get().into(),
                    );
                    headers.insert(
                        ntex::http::header::HeaderName::from_str(
                            "x-ratelimit-remaining",
                        )
                        .unwrap(),
                        snapshot.remaining_burst_capacity().into(),
                    );

                    headers.insert(
                        ntex::http::header::HeaderName::from_str(
                            "x-ratelimit-reset",
                        )
                        .unwrap(),
                        snapshot
                            .quota()
                            .burst_size_replenished_in()
                            .as_secs()
                            .into(),
                    );

                    Ok(service_response)
                }
                Err(negative) => {
                    let wait_time =
                        negative.wait_time_from(DefaultClock::default().now());

                    let (req, _) = req.into_parts();
                    let mut response = ApiError::RateLimitError(
                        wait_time.as_millis(),
                        negative.quota().burst_size().get(),
                    )
                    .error_response(&req);

                    let headers = response.headers_mut();

                    headers.insert(
                        ntex::http::header::HeaderName::from_str(
                            "x-ratelimit-limit",
                        )
                        .unwrap(),
                        negative.quota().burst_size().get().into(),
                    );
                    headers.insert(
                        ntex::http::header::HeaderName::from_str(
                            "x-ratelimit-remaining",
                        )
                        .unwrap(),
                        0.into(),
                    );
                    headers.insert(
                        ntex::http::header::HeaderName::from_str(
                            "x-ratelimit-reset",
                        )
                        .unwrap(),
                        wait_time.as_secs().into(),
                    );

                    Ok(WebResponse::new(response, req))
                }
            }
        } else {
            let (req, _) = req.into_parts();
            let response = ApiError::CustomAuthentication(
                "Unable to obtain user IP address!".to_string(),
            )
            .error_response(&req);

            Ok(WebResponse::new(response, req))
        }
    }
}
