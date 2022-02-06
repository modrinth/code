//! RateLimiter middleware for actix application
use crate::ratelimit::errors::ARError;
use crate::ratelimit::{ActorMessage, ActorResponse};
use actix::dev::*;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error as AWError,
    http::header::{HeaderName, HeaderValue},
};
use futures::future::{ok, Ready};
use log::*;
use std::{
    cell::RefCell,
    future::Future,
    ops::Fn,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::Duration,
};

/// Type that implements the ratelimit middleware.
///
/// This accepts _interval_ which specifies the
/// window size, _max_requests_ which specifies the maximum number of requests in that window, and
/// _store_ which is essentially a data store used to store client access information. Entry is removed from
/// the store after _interval_.
///
/// # Example
/// ```rust
/// # use std::time::Duration;
/// use actix_ratelimit::{MemoryStore, MemoryStoreActor};
/// use actix_ratelimit::RateLimiter;
///
/// #[actix_rt::main]
/// async fn main() {
///     let store = MemoryStore::new();
///     let ratelimiter = RateLimiter::new(
///                             MemoryStoreActor::from(store.clone()).start())
///                         .with_interval(Duration::from_secs(60))
///                         .with_max_requests(100);
/// }
/// ```
pub struct RateLimiter<T>
where
    T: Handler<ActorMessage> + Send + Sync + 'static,
    T::Context: ToEnvelope<T, ActorMessage>,
{
    interval: Duration,
    max_requests: usize,
    store: Addr<T>,
    identifier: Rc<Box<dyn Fn(&ServiceRequest) -> Result<String, ARError>>>,
    ignore_ips: Vec<String>,
}

impl<T> RateLimiter<T>
where
    T: Handler<ActorMessage> + Send + Sync + 'static,
    <T as Actor>::Context: ToEnvelope<T, ActorMessage>,
{
    /// Creates a new instance of `RateLimiter` with the provided address of `StoreActor`.
    pub fn new(store: Addr<T>) -> Self {
        let identifier = |req: &ServiceRequest| {
            let connection_info = req.connection_info();
            let ip = connection_info
                .peer_addr()
                .ok_or(ARError::IdentificationError)?;
            Ok(String::from(ip))
        };
        RateLimiter {
            interval: Duration::from_secs(0),
            max_requests: 0,
            store,
            identifier: Rc::new(Box::new(identifier)),
            ignore_ips: Vec::new(),
        }
    }

    /// Specify the interval. The counter for a client is reset after this interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Specify the maximum number of requests allowed in the given interval.
    pub fn with_max_requests(mut self, max_requests: usize) -> Self {
        self.max_requests = max_requests;
        self
    }

    /// Sets IPs that should be ignored by the ratelimiter
    pub fn with_ignore_ips(mut self, ignore_ips: Vec<String>) -> Self {
        self.ignore_ips = ignore_ips;
        self
    }

    /// Function to get the identifier for the client request
    pub fn with_identifier<F: Fn(&ServiceRequest) -> Result<String, ARError> + 'static>(
        mut self,
        identifier: F,
    ) -> Self {
        self.identifier = Rc::new(Box::new(identifier));
        self
    }
}

impl<T, S, B> Transform<S, ServiceRequest> for RateLimiter<T>
where
    T: Handler<ActorMessage> + Send + Sync + 'static,
    T::Context: ToEnvelope<T, ActorMessage>,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = AWError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Transform = RateLimitMiddleware<S, T>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddleware {
            service: Rc::new(RefCell::new(service)),
            store: self.store.clone(),
            max_requests: self.max_requests,
            interval: self.interval.as_secs(),
            identifier: self.identifier.clone(),
            ignore_ips: self.ignore_ips.clone(),
        })
    }
}

/// Service factory for RateLimiter
pub struct RateLimitMiddleware<S, T>
where
    S: 'static,
    T: Handler<ActorMessage> + 'static,
{
    service: Rc<RefCell<S>>,
    store: Addr<T>,
    // Exists here for the sole purpose of knowing the max_requests and interval from RateLimiter
    max_requests: usize,
    interval: u64,
    identifier: Rc<Box<dyn Fn(&ServiceRequest) -> Result<String, ARError> + 'static>>,
    ignore_ips: Vec<String>,
}

impl<T, S, B> Service<ServiceRequest> for RateLimitMiddleware<S, T>
where
    T: Handler<ActorMessage> + 'static,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = AWError> + 'static,
    S::Future: 'static,
    B: 'static,
    T::Context: ToEnvelope<T, ActorMessage>,
{
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let store = self.store.clone();
        let srv = self.service.clone();
        let max_requests = self.max_requests;
        let interval = Duration::from_secs(self.interval);
        let identifier = self.identifier.clone();
        let ignore_ips = self.ignore_ips.clone();
        Box::pin(async move {
            let identifier: String = (identifier)(&req)?;
            if ignore_ips.contains(&identifier) {
                let fut = srv.call(req);
                let res = fut.await?;
                return Ok(res);
            }
            let remaining: ActorResponse = store
                .send(ActorMessage::Get(String::from(&identifier)))
                .await
                .map_err(|_| ARError::IdentificationError)?;
            match remaining {
                ActorResponse::Get(opt) => {
                    let opt = opt.await?;
                    if let Some(c) = opt {
                        // Existing entry in store
                        let expiry = store
                            .send(ActorMessage::Expire(String::from(&identifier)))
                            .await
                            .map_err(|_| ARError::ReadWriteError("Setting timeout".to_string()))?;
                        let reset: Duration = match expiry {
                            ActorResponse::Expire(dur) => dur.await?,
                            _ => unreachable!(),
                        };
                        if c == 0 {
                            info!("Limit exceeded for client: {}", &identifier);
                            Err(ARError::LimitedError {
                                max_requests,
                                remaining: c,
                                reset: reset.as_secs(),
                            }
                            .into())
                        } else {
                            // Decrement value
                            let res: ActorResponse = store
                                .send(ActorMessage::Update {
                                    key: identifier,
                                    value: 1,
                                })
                                .await
                                .map_err(|_| {
                                    ARError::ReadWriteError("Decrementing ratelimit".to_string())
                                })?;
                            let updated_value: usize = match res {
                                ActorResponse::Update(c) => c.await?,
                                _ => unreachable!(),
                            };
                            // Execute the request
                            let fut = srv.call(req);
                            let mut res = fut.await?;
                            let headers = res.headers_mut();
                            // Safe unwraps, since usize is always convertible to string
                            headers.insert(
                                HeaderName::from_static("x-ratelimit-limit"),
                                HeaderValue::from_str(max_requests.to_string().as_str())?,
                            );
                            headers.insert(
                                HeaderName::from_static("x-ratelimit-remaining"),
                                HeaderValue::from_str(updated_value.to_string().as_str())?,
                            );
                            headers.insert(
                                HeaderName::from_static("x-ratelimit-reset"),
                                HeaderValue::from_str(reset.as_secs().to_string().as_str())?,
                            );
                            Ok(res)
                        }
                    } else {
                        // New client, create entry in store
                        let current_value = max_requests - 1;
                        let res = store
                            .send(ActorMessage::Set {
                                key: String::from(&identifier),
                                value: current_value,
                                expiry: interval,
                            })
                            .await
                            .map_err(|_| {
                                ARError::ReadWriteError("Creating store entry".to_string())
                            })?;
                        match res {
                            ActorResponse::Set(c) => c.await?,
                            _ => unreachable!(),
                        }
                        let fut = srv.call(req);
                        let mut res = fut.await?;
                        let headers = res.headers_mut();
                        // Safe unwraps, since usize is always convertible to string
                        headers.insert(
                            HeaderName::from_static("x-ratelimit-limit"),
                            HeaderValue::from_str(max_requests.to_string().as_str()).unwrap(),
                        );
                        headers.insert(
                            HeaderName::from_static("x-ratelimit-remaining"),
                            HeaderValue::from_str(current_value.to_string().as_str()).unwrap(),
                        );
                        headers.insert(
                            HeaderName::from_static("x-ratelimit-reset"),
                            HeaderValue::from_str(interval.as_secs().to_string().as_str()).unwrap(),
                        );
                        Ok(res)
                    }
                }
                _ => {
                    unreachable!();
                }
            }
        })
    }
}
