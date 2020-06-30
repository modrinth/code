use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct ApiError<'a> {
    pub error: &'a str,
    pub description: &'a str,
}
