//! Login redirect step
pub fn get_url(public_uri: &str, conn_id: &str, client_id: &str) -> String {
    format!(
        "https://login.live.com/oauth20_authorize.srf?client_id={client_id}&response_type=code&redirect_uri={}&scope={}&state={conn_id}&prompt=select_account&cobrandid=8058f65d-ce06-4c30-9559-473c9275a65d",
        urlencoding::encode(&format!("{}/v2/auth/minecraft/callback", public_uri)),
        urlencoding::encode("XboxLive.signin offline_access")
    )
}
