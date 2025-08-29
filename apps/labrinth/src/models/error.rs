use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// An error returned by the API
#[derive(Serialize, Deserialize)]
pub struct ApiError<'a> {
    pub error: &'a str,
    pub description: Cow<'a, str>,
}
