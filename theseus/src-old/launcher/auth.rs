pub mod api {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GameProfile {
        pub id: Uuid,
        pub name: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct UserProperty {
        pub name: String,
        pub value: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct User {
        pub id: String,
        pub username: String,
        pub properties: Option<Vec<UserProperty>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AuthenticateResponse {
        pub user: Option<User>,
        pub client_token: Uuid,
        pub access_token: String,
        pub available_profiles: Vec<GameProfile>,
        pub selected_profile: Option<GameProfile>,
    }

    pub async fn login(
        username: &str,
        password: &str,
        request_user: bool,
    ) -> Result<AuthenticateResponse, reqwest::Error> {
        let client = reqwest::Client::new();

        client
            .post("https://authserver.mojang.com/authenticate")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(
                serde_json::json!(
                    {
                        "agent": {
                            "name": "Minecraft",
                            "version": 1
                        },
                        "username": username,
                        "password": password,
                        "clientToken": Uuid::new_v4(),
                        "requestUser": request_user
                    }
                )
                .to_string(),
            )
            .send()
            .await?
            .json()
            .await
    }

    pub async fn sign_out(username: &str, password: &str) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();

        client
            .post("https://authserver.mojang.com/signout")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(
                serde_json::json!(
                    {
                        "username": username,
                        "password": password
                    }
                )
                .to_string(),
            )
            .send()
            .await?;

        Ok(())
    }

    pub async fn validate(access_token: &str, client_token: &str) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();

        client
            .post("https://authserver.mojang.com/validate")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(
                serde_json::json!(
                    {
                        "accessToken": access_token,
                        "clientToken": client_token
                    }
                )
                .to_string(),
            )
            .send()
            .await?;

        Ok(())
    }

    pub async fn invalidate(access_token: &str, client_token: &str) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();

        client
            .post("https://authserver.mojang.com/invalidate")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(
                serde_json::json!(
                    {
                        "accessToken": access_token,
                        "clientToken": client_token
                    }
                )
                .to_string(),
            )
            .send()
            .await?;

        Ok(())
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RefreshResponse {
        pub user: Option<User>,
        pub client_token: Uuid,
        pub access_token: String,
        pub selected_profile: Option<GameProfile>,
    }

    pub async fn refresh(
        access_token: &str,
        client_token: &str,
        selected_profile: &GameProfile,
        request_user: bool,
    ) -> Result<RefreshResponse, reqwest::Error> {
        let client = reqwest::Client::new();

        client
            .post("https://authserver.mojang.com/refresh")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(
                serde_json::json!(
                    {
                        "accessToken": access_token,
                        "clientToken": client_token,
                        "selectedProfile": {
                            "id": selected_profile.id,
                            "name": selected_profile.name,
                        },
                        "requestUser": request_user,
                    }
                )
                .to_string(),
            )
            .send()
            .await?
            .json()
            .await
    }
}

pub mod provider {
    use crate::launcher::auth::api::login;
    use crate::launcher::LauncherError;
    use uuid::Uuid;

    #[derive(Debug)]
    /// The credentials of a user
    pub struct Credentials {
        /// The user UUID the credentials belong to
        pub id: Uuid,
        /// The username of the user
        pub username: String,
        /// The access token associated with the credentials
        pub access_token: String,
    }

    impl Credentials {
        /// Gets a credentials instance from a user's login
        pub async fn from_login(username: &str, password: &str) -> Result<Self, LauncherError> {
            let login =
                login(username, password, true)
                    .await
                    .map_err(|err| LauncherError::FetchError {
                        inner: err,
                        item: "authentication credentials".to_string(),
                    })?;

            let profile = login.selected_profile.unwrap();

            Ok(Credentials {
                id: profile.id,
                username: profile.name,
                access_token: login.access_token,
            })
        }
    }
}
