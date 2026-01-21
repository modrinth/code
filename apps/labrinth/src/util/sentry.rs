use std::{pin::Pin, rc::Rc, sync::Arc};

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::{
    FutureExt,
    future::{Ready, ok},
};
use sentry::{Hub, SentryFutureExt};

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
        let hub = Arc::new(Hub::new_from_top(Hub::main()));

        let svc = self.service.clone();
        async move {
            let fut =
                Hub::run(hub.clone(), || svc.call(req)).bind_hub(hub.clone());
            let res: Self::Response = match fut.await {
                Ok(res) => res,
                Err(actix_err) => {
                    if actix_err.error_response().status().is_server_error() {
                        capture_downcasted_error(&hub, &actix_err);
                    }

                    return Err(actix_err);
                }
            };

            if res.response().status().is_server_error()
                && let Some(actix_err) = res.response().error()
            {
                capture_downcasted_error(&hub, actix_err);
            }

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
