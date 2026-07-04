use serde::{Deserialize, Serialize};

/// An error returned by the API
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct ApiError<'a> {
    #[schema(value_type = String)]
    pub error: &'a str,
    pub description: String,
    #[schema(value_type = Option<serde_json::Value>)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
