//! MSA authentication stages

use lazy_static::lazy_static;

pub mod access_token;
pub mod bearer_token;
pub mod login_redirect;
pub mod player_info;
pub mod xbl_signin;
pub mod xsts_token;

lazy_static! {
    static ref REQWEST_CLIENT: reqwest::Client = {
        let mut headers = reqwest::header::HeaderMap::new();
        let header = reqwest::header::HeaderValue::from_str(&format!(
            "modrinth/labrinth/{} (support@modrinth.com)",
            env!("CARGO_PKG_VERSION")
        ))
        .unwrap();
        headers.insert(reqwest::header::USER_AGENT, header);
        reqwest::Client::builder()
            .tcp_keepalive(Some(std::time::Duration::from_secs(10)))
            .default_headers(headers)
            .build()
            .expect("Reqwest Client Building Failed")
    };
}
