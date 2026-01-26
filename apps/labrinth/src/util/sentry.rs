// large parts are copied from
// <https://github.com/getsentry/sentry-rust/blob/99e1d9d9b78074a9a4c472fa7d2fc0f15c474a4b/sentry-actix/src/lib.rs>
//
// TODO: PR something into sentry_actix to let us customize this

use std::{borrow::Cow, pin::Pin, rc::Rc};

use actix_http::{
    StatusCode,
    header::{self, HeaderMap},
};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use bytes::{Bytes, BytesMut};
use futures::{
    FutureExt, TryStreamExt,
    future::{Ready, ok},
};
use sentry::{
    Hub, MaxRequestBodySize, SentryFutureExt,
    protocol::{self, ClientSdkPackage, Event, Request},
};

use crate::routes::ApiError;

/// Captures errors and reports them to Sentry.
///
/// This rips out the error reporting logic from [`sentry_actix::Sentry`] and
/// customizes the logic to report errors with a proper stack trace.
///
/// Since the error type of responses is [`actix_web::Error`], which implements
/// [`std::error::Error`] by always returning `None` for the source, the
/// reported error will always have no real error stack, which makes Sentry
/// issues a lot less useful. We fix this by manually converting the error to
/// a type which does have a proper error stack.
#[derive(Clone)]
pub struct SentryErrorReporting;

impl<S, B> Transform<S, ServiceRequest> for SentryErrorReporting
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = SentryErrorMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SentryErrorMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct SentryErrorMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SentryErrorMiddleware<S>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let hub = Hub::current();
        let client = hub.client();

        let max_request_body_size = client
            .as_ref()
            .map(|client| client.options().max_request_body_size)
            .unwrap_or(MaxRequestBodySize::None);

        let with_pii = client
            .as_ref()
            .is_some_and(|client| client.options().send_default_pii);

        let mut sentry_req = sentry_request_from_http(&req, with_pii);
        let name = transaction_name_from_http(&req);

        let transaction = {
            let headers = req.headers().iter().filter_map(|(header, value)| {
                value.to_str().ok().map(|value| (header.as_str(), value))
            });

            let ctx = sentry::TransactionContext::continue_from_headers(
                &name,
                "http.server",
                headers,
            );

            let transaction = hub.start_transaction(ctx);
            transaction.set_request(sentry_req.clone());
            transaction.set_origin("auto.http.actix");
            transaction
        };

        let svc = self.service.clone();
        async move {
            let mut req = req;

            if should_capture_request_body(
                req.headers(),
                with_pii,
                max_request_body_size,
            ) {
                sentry_req.data = Some(capture_request_body(&mut req).await);
            }

            let parent_span = hub.configure_scope(|scope| {
                let parent_span = scope.get_span();
                scope.set_span(Some(transaction.clone().into()));
                scope.add_event_processor(move |event| {
                    Some(process_event(event, &sentry_req))
                });
                parent_span
            });

            let fut =
                Hub::run(hub.clone(), || svc.call(req)).bind_hub(hub.clone());
            let res: Self::Response = match fut.await {
                Ok(res) => res,
                Err(actix_err) => {
                    if actix_err.error_response().status().is_server_error() {
                        capture_downcasted_error(&hub, &actix_err);
                    }

                    if transaction.get_status().is_none() {
                        let status = protocol::SpanStatus::UnknownError;
                        transaction.set_status(status);
                    }
                    transaction.finish();
                    hub.configure_scope(|scope| scope.set_span(parent_span));
                    return Err(actix_err);
                }
            };

            // Response errors
            if res.response().status().is_server_error()
                && let Some(actix_err) = res.response().error()
            {
                capture_downcasted_error(&hub, actix_err);
            }

            if transaction.get_status().is_none() {
                let status = map_status(res.status());
                transaction.set_status(status);
            }
            transaction.finish();
            hub.configure_scope(|scope| scope.set_span(parent_span));

            Ok(res)
        }
        .boxed_local()
    }
}

/// Converts an [`actix_web::Error`] into an error which implements
/// [`std::error::Error`] properly, so that Sentry can capture its error stack.
///
/// If the underlying error is of a supported type like [`ApiError`], the error
/// stack will be properly captured. Otherwise, we use some error types to
/// still print the full stack trace, but "improperly". This is due to
/// limitations with Actix boxing the errors and type erasure.
fn capture_downcasted_error(hub: &Hub, actix_err: &actix_web::Error) {
    #[derive(Debug, thiserror::Error)]
    #[error("(note: error stack missing since it is of an unsupported type)")]
    struct ErrorStackMissing;

    #[derive(Debug, thiserror::Error)]
    #[error("{msg}")]
    struct UnknownApiError {
        msg: String,
        source: ErrorStackMissing,
    }

    if let Some(real_err) = actix_err.as_error::<ApiError>() {
        hub.capture_error(real_err);
    } else {
        // due to type erasure, we can't downcast `err`'s underlying error to
        // an error type from which we can fetch stacktrace
        // and, due to type erasure, we don't even know its type name - how sad!
        // use `:#` format to print the error chain, not just the first one
        let err = UnknownApiError {
            msg: format!("{actix_err:#}"),
            source: ErrorStackMissing,
        };
        hub.capture_error(&err);
    }
}

/// Extract a transaction name from the HTTP request
fn transaction_name_from_http(req: &ServiceRequest) -> String {
    let path_part = req.match_pattern().unwrap_or_else(|| "<none>".to_string());
    format!("{} {}", req.method(), path_part)
}

/// Build a Sentry request struct from the HTTP request
fn sentry_request_from_http(
    request: &ServiceRequest,
    with_pii: bool,
) -> Request {
    let mut sentry_req = Request {
        url: format!(
            "{}://{}{}",
            request.connection_info().scheme(),
            request.connection_info().host(),
            request.uri()
        )
        .parse()
        .ok()
        .map(scrub_pii_from_url),
        method: Some(request.method().to_string()),
        headers: request
            .headers()
            .iter()
            .filter(|(_, v)| !v.is_sensitive())
            .filter(|(k, _)| with_pii || !is_sensitive_header(k.as_str()))
            .map(|(k, v)| {
                (k.to_string(), v.to_str().unwrap_or_default().to_string())
            })
            .collect(),
        ..Default::default()
    };

    // If PII is enabled, include the remote address
    if with_pii && let Some(remote) = request.connection_info().peer_addr() {
        sentry_req.env.insert("REMOTE_ADDR".into(), remote.into());
    };

    sentry_req
}

/// Scrub PII (username and password) from the given URL.
pub fn scrub_pii_from_url(mut url: url::Url) -> url::Url {
    // the set calls will fail and return an error if the URL is relative
    // in those cases, just ignore the errors
    if !url.username().is_empty() {
        let _ = url.set_username(PII_REPLACEMENT);
    }
    if url.password().is_some() {
        let _ = url.set_password(Some(PII_REPLACEMENT));
    }
    url
}

async fn capture_request_body(req: &mut ServiceRequest) -> String {
    match body_from_http(req).await {
        Ok(request_body) => String::from_utf8_lossy(&request_body).into_owned(),
        Err(_) => String::new(),
    }
}

/// Extract a body from the HTTP request
async fn body_from_http(req: &mut ServiceRequest) -> actix_web::Result<Bytes> {
    let stream = req.extract::<actix_web::web::Payload>().await?;
    let body = stream.try_collect::<BytesMut>().await?.freeze();

    // put copy of payload back into request for downstream to read
    req.set_payload(actix_web::dev::Payload::from(body.clone()));

    Ok(body)
}

/// Add request data to a Sentry event
fn process_event(
    mut event: Event<'static>,
    request: &Request,
) -> Event<'static> {
    // Request
    if event.request.is_none() {
        event.request = Some(request.clone());
    }

    // SDK
    if let Some(sdk) = event.sdk.take() {
        let mut sdk = sdk.into_owned();
        sdk.packages.push(ClientSdkPackage {
            name: "sentry-actix".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        });
        event.sdk = Some(Cow::Owned(sdk));
    }
    event
}

const SENSITIVE_HEADERS_UPPERCASE: &[&str] = &[
    "AUTHORIZATION",
    "PROXY_AUTHORIZATION",
    "COOKIE",
    "SET_COOKIE",
    "X_FORWARDED_FOR",
    "X_REAL_IP",
    "X_API_KEY",
];

const PII_REPLACEMENT: &str = "[Filtered]";

/// Determines if the HTTP header with the given name shall be considered as potentially carrying
/// sensitive data.
pub fn is_sensitive_header(name: &str) -> bool {
    SENSITIVE_HEADERS_UPPERCASE
        .contains(&name.to_ascii_uppercase().replace("-", "_").as_str())
}

fn should_capture_request_body(
    headers: &HeaderMap,
    with_pii: bool,
    max_request_body_size: MaxRequestBodySize,
) -> bool {
    let is_chunked = headers
        .get(header::TRANSFER_ENCODING)
        .and_then(|h| h.to_str().ok())
        .map(|transfer_encoding| transfer_encoding.contains("chunked"))
        .unwrap_or(false);

    let is_valid_content_type = with_pii
        || headers
            .get(header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .is_some_and(|content_type| {
                matches!(
                    content_type,
                    "application/json" | "application/x-www-form-urlencoded"
                )
            });

    let is_within_size_limit = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|content_length| content_length.parse::<usize>().ok())
        .map(|content_length| {
            max_request_body_size.is_within_size_limit(content_length)
        })
        .unwrap_or(false);

    !is_chunked && is_valid_content_type && is_within_size_limit
}

fn map_status(status: StatusCode) -> protocol::SpanStatus {
    match status {
        StatusCode::UNAUTHORIZED => protocol::SpanStatus::Unauthenticated,
        StatusCode::FORBIDDEN => protocol::SpanStatus::PermissionDenied,
        StatusCode::NOT_FOUND => protocol::SpanStatus::NotFound,
        StatusCode::TOO_MANY_REQUESTS => {
            protocol::SpanStatus::ResourceExhausted
        }
        status if status.is_client_error() => {
            protocol::SpanStatus::InvalidArgument
        }
        StatusCode::NOT_IMPLEMENTED => protocol::SpanStatus::Unimplemented,
        StatusCode::SERVICE_UNAVAILABLE => protocol::SpanStatus::Unavailable,
        status if status.is_server_error() => {
            protocol::SpanStatus::InternalError
        }
        StatusCode::CONFLICT => protocol::SpanStatus::AlreadyExists,
        status if status.is_success() => protocol::SpanStatus::Ok,
        _ => protocol::SpanStatus::UnknownError,
    }
}
