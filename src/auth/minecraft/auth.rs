//! Main authentication flow for Hydra
use crate::{auth::minecraft::stages, auth::templates, parse_var};

// use crate::db::RuntimeState;
use crate::database::models::flow_item::Flow;
use crate::queue::socket::ActiveSockets;
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse};
use chrono::Duration;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::RwLock;

macro_rules! ws_conn_try {
    ($ctx:literal $status:path, $res:expr => $ws_conn:expr) => {
        match $res {
            Ok(res) => res,
            Err(err) => {
                let error = format!("In {}: {err}", $ctx);
                let render = super::Error::render_string(&error);
                let _ = $ws_conn.text(render.clone()).await;
                let _ = $ws_conn.close(None).await;
                return Err(templates::ErrorPage {
                    code: $status,
                    message: render,
                });
            }
        }
    };
}

#[derive(Deserialize)]
pub struct Query {
    pub code: String,
    pub state: String,
}

#[get("callback")]
pub async fn route(
    db: web::Data<RwLock<ActiveSockets>>,
    info: web::Query<Query>,
    redis: web::Data<deadpool_redis::Pool>,
) -> Result<HttpResponse, templates::ErrorPage> {
    let public_url = parse_var::<String>("SELF_ADDR").unwrap_or(format!(
        "http://{}",
        parse_var::<String>("BIND_ADDR").unwrap()
    ));
    let client_id = parse_var::<String>("MICROSOFT_CLIENT_ID").unwrap();
    let client_secret = parse_var::<String>("MICROSOFT_CLIENT_SECRET").unwrap();

    let code = &info.code;

    let mut ws_conn = {
        let db = db.read().await;

        let mut x = db
            .auth_sockets
            .get_mut(&info.state)
            .ok_or_else(|| templates::ErrorPage {
                code: StatusCode::BAD_REQUEST,
                message: "Invalid state sent, you probably need to get a new websocket".to_string(),
            })?;

        x.value_mut().clone()
    };

    ws_conn_try!(
        "Removing login flow" StatusCode::INTERNAL_SERVER_ERROR,
        Flow::remove(code, &redis).await
        => ws_conn
    );

    let access_token = ws_conn_try!(
        "OAuth token exchange" StatusCode::INTERNAL_SERVER_ERROR,
        stages::access_token::fetch_token(
            public_url,
            code,
            &client_id,
            &client_secret,
        ).await
        => ws_conn
    );

    let stages::xbl_signin::XBLLogin {
        token: xbl_token,
        uhs,
    } = ws_conn_try!(
        "XBox Live token exchange" StatusCode::INTERNAL_SERVER_ERROR,
        stages::xbl_signin::login_xbl(&access_token.access_token).await
        => ws_conn
    );

    let xsts_response = ws_conn_try!(
        "XSTS token exchange" StatusCode::INTERNAL_SERVER_ERROR,
        stages::xsts_token::fetch_token(&xbl_token).await
        => ws_conn
    );

    match xsts_response {
        stages::xsts_token::XSTSResponse::Unauthorized(err) => {
            let _ = ws_conn
                .text(super::Error::render_string(&format!(
                    "Error getting XBox Live token: {err}"
                )))
                .await;
            let _ = ws_conn.close(None).await;

            Err(templates::ErrorPage {
                code: StatusCode::FORBIDDEN,
                message: err,
            })
        }
        stages::xsts_token::XSTSResponse::Success { token: xsts_token } => {
            let bearer_token = &ws_conn_try!(
                "Bearer token flow" StatusCode::INTERNAL_SERVER_ERROR,
                stages::bearer_token::fetch_bearer(&xsts_token, &uhs)
                    .await
                => ws_conn
            );

            let player_info = &ws_conn_try!(
                "No Minecraft account for profile. Make sure you own the game and have set a username through the official Minecraft launcher." StatusCode::BAD_REQUEST,
                stages::player_info::fetch_info(bearer_token)
                    .await
                => ws_conn
            );

            let flow = &ws_conn_try!(
                "Error creating microsoft login request flow." StatusCode::INTERNAL_SERVER_ERROR,
                Flow::MicrosoftLogin {
                    access_token: access_token.access_token.clone(),
                }
                .insert(Duration::hours(1), &redis)
                .await
                => ws_conn
            );

            ws_conn
                .text(
                    json!({
                        "token": bearer_token,
                        "refresh_token": &access_token.refresh_token,
                        "expires_after": 86400,
                        "flow": flow,
                    }).to_string()
                )
                .await.map_err(|_| templates::ErrorPage {
                code: StatusCode::BAD_REQUEST,
                message: "Failed to send login details to launcher. Try restarting the login process!".to_string(),
            })?;
            let _ = ws_conn.close(None).await;

            Ok(templates::Success {
                name: &player_info.name,
                icon: &format!("https://mc-heads.net/avatar/{}/128", &player_info.id),
            }
            .render())
        }
    }
}
