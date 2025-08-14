use serde::{Deserialize, Serialize};

/// An error returned by the API
#[derive(Serialize, Deserialize)]
pub struct ApiError<'a> {
    pub error: &'a str,
    pub description: String,
}
