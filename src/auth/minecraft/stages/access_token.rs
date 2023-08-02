//! Get access token from code
use serde::Deserialize;
use std::collections::HashMap;

const OAUTH_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";

#[derive(Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn fetch_token(
    public_uri: String,
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<Tokens, reqwest::Error> {
    let redirect_uri = format!("{}/v2/auth/minecraft/callback", public_uri);

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("code", code);
    params.insert("grant_type", "authorization_code");
    params.insert("redirect_uri", redirect_uri.as_str());

    let client = reqwest::Client::new();
    let result = client
        .post(OAUTH_TOKEN_URL)
        .form(&params)
        .send()
        .await?
        .json::<Tokens>()
        .await?;

    Ok(result)
}

pub async fn refresh_token(
    public_uri: &str,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<Tokens, reqwest::Error> {
    let redirect_uri = format!("{}/v2/auth/minecraft/callback", public_uri);

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("refresh_token", refresh_token);
    params.insert("grant_type", "refresh_token");
    params.insert("redirect_uri", &redirect_uri);

    let client = reqwest::Client::new();
    let result = client
        .post(OAUTH_TOKEN_URL)
        .form(&params)
        .send()
        .await?
        .json::<Tokens>()
        .await?;

    Ok(result)
}
