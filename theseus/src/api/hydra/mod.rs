pub mod complete;
pub mod init;
pub mod refresh;
pub(crate) mod stages;

use serde::Deserialize;

const MICROSOFT_CLIENT_ID: &str = "c4502edb-87c6-40cb-b595-64a280cf8906";

#[derive(Deserialize)]
pub struct MicrosoftError {
    pub error: String,
    pub error_description: String,
    pub error_codes: Vec<u64>,
}
