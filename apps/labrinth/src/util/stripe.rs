use anyhow::anyhow;
use std::fmt::{Debug, Display, Write as _};

use crate::routes::ApiError;
use crate::util::error::Context as _;

/// Converts a Stripe client error into a report with a concise message.
///
/// The `Display` of [`stripe::StripeError`] dumps the whole error object Stripe
/// returned, which can include the payment intent, payment method and client
/// secret involved. Error chains end up in logs and in the `details` of API
/// error responses, so only the type, codes and message are kept.
pub fn stripe_error_report(error: ::stripe::StripeError) -> anyhow::Error {
    let ::stripe::StripeError::Stripe(error, status) = error else {
        return anyhow::Error::new(error);
    };

    let mut message = format!("Stripe returned {} ({status})", error.type_);
    if let Some(code) = &error.code {
        let _ = write!(message, " [{code}]");
    }
    if let Some(decline_code) = &error.decline_code {
        let _ = write!(message, " [{decline_code}]");
    }
    if let Some(param) = &error.param {
        let _ = write!(message, " for `{param}`");
    }
    if let Some(text) = &error.message {
        let _ = write!(message, ": {text}");
    }

    anyhow!(message)
}

/// Adds context to the result of a Stripe request.
#[allow(
    clippy::missing_errors_doc,
    reason = "this trait's purpose is improving error handling"
)]
pub trait StripeContext<T>: Sized {
    /// Maps the error variant into an [`ApiError::FailedDependency`] with the
    /// given message, describing the Stripe error concisely.
    fn stripe_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static;
}

impl<T> StripeContext<T> for Result<T, ::stripe::StripeError> {
    fn stripe_context<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Send + Sync + Debug + Display + 'static,
    {
        self.map_err(stripe_error_report)
            .failed_dependency_context(msg)
    }
}
