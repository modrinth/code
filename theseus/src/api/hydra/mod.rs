pub mod complete;
pub mod init;
pub mod refresh;
pub(crate) mod stages;

use serde::Deserialize;

const MICROSOFT_CLIENT_ID: &str = "c4502edb-87c6-40cb-b595-64a280cf8906";
// TODO: figure out what to do with this
const MICROSOFT_CLIENT_SECRET: &str = "TODOFIGUREOUT";

const REDIRECT_URL: &str = "http://localhost:20123/theseus/callback";
const REQUESTED_SCOPES: &str =
    "XboxLive.signin XboxLive.offline_access profile openid email";

#[derive(Deserialize)]
pub struct MicrosoftError {
    pub error: String,
    pub error_description: String,
    pub error_codes: Vec<u64>,
}
