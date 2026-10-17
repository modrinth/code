//! A minimal OAuth 2.0 authorization code grant flow redirection/reply loopback URI HTTP
//! server implementation, compliant with [RFC 6749]'s authorization code grant flow and
//! [RFC 8252]'s best current practices for OAuth 2.0 in native apps.
//!
//! This server is needed for the step 4 of the OAuth authentication dance represented in
//! figure 1 of [RFC 8252].
//!
//! Further reading: https://www.oauth.com/oauth2-servers/oauth-native-apps/redirect-urls-for-native-apps/
//!
//! [RFC 6749]: https://datatracker.ietf.org/doc/html/rfc6749
//! [RFC 8252]: https://datatracker.ietf.org/doc/html/rfc8252

use std::{net::SocketAddr, sync::Mutex, time::Duration};

use hyper::body::Incoming;
use hyper_util::rt::{TokioIo, TokioTimer};
use theseus::ErrorKind;
use theseus::prelude::tcp_listen_any_loopback;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use url::Url;

const MAX_AUTH_PARAM_LEN: usize = 4096;

struct PendingAuth {
    nonce: String,
    deeplink_tx: Option<oneshot::Sender<String>>,
    cancel_tx: Option<oneshot::Sender<()>>,
}

static PENDING_AUTH: Mutex<Option<PendingAuth>> = Mutex::new(None);

pub struct AuthReplySession {
    nonce: String,
    socket_rx: Option<oneshot::Receiver<Result<SocketAddr, theseus::Error>>>,
    listener: JoinHandle<Result<Option<String>, theseus::Error>>,
    deeplink_rx: oneshot::Receiver<String>,
}

impl AuthReplySession {
    pub fn start(nonce: String) -> Self {
        let (socket_tx, socket_rx) = oneshot::channel();
        let (deeplink_tx, deeplink_rx) = oneshot::channel();
        let (cancel_tx, cancel_rx) = oneshot::channel();

        let previous = PENDING_AUTH.lock().unwrap().replace(PendingAuth {
            nonce: nonce.clone(),
            deeplink_tx: Some(deeplink_tx),
            cancel_tx: Some(cancel_tx),
        });
        if let Some(previous) = previous {
            previous.cancel();
        }

        let listener =
            tokio::spawn(listen(socket_tx, nonce.clone(), cancel_rx));

        Self {
            nonce,
            socket_rx: Some(socket_rx),
            listener,
            deeplink_rx,
        }
    }

    pub async fn socket_addr(&mut self) -> Result<SocketAddr, theseus::Error> {
        let mut socket_rx =
            self.socket_rx.take().ok_or_else(auth_listener_stopped)?;

        tokio::select! {
            socket = &mut socket_rx => {
                socket.map_err(|_| auth_listener_stopped())?
            }
            _ = &mut self.deeplink_rx => Err(auth_listener_stopped()),
        }
    }

    pub async fn wait(&mut self) -> Result<Option<String>, theseus::Error> {
        let result = tokio::select! {
            localhost_result = &mut self.listener => {
                localhost_result.map_err(|error| {
                    ErrorKind::OtherError(format!(
                        "Auth code listener stopped: {error}"
                    ))
                })?
            }
            deeplink_result = &mut self.deeplink_rx => {
                self.listener.abort();
                Ok(deeplink_result.ok())
            }
        };

        clear_pending_auth(&self.nonce);
        result
    }
}

impl Drop for AuthReplySession {
    fn drop(&mut self) {
        clear_pending_auth(&self.nonce);
        self.listener.abort();
    }
}

impl PendingAuth {
    fn cancel(mut self) {
        if let Some(cancel_tx) = self.cancel_tx.take() {
            cancel_tx.send(()).ok();
        }
    }
}

/// cancels the currently active app authentication callback, if any
pub fn cancel_pending_auth() {
    if let Some(pending) = PENDING_AUTH.lock().unwrap().take() {
        pending.cancel();
    }
}

/// delivers a `modrinth://auth` callback to the in-flight protocol 2 sign-in.
/// returns false when no sign-in is waiting on this nonce
pub fn submit_deeplink(code: String, nonce: &str) -> bool {
    if code.len() > MAX_AUTH_PARAM_LEN || nonce.len() > MAX_AUTH_PARAM_LEN {
        return false;
    }

    let deeplink_tx = {
        let mut pending = PENDING_AUTH.lock().unwrap();
        pending
            .as_mut()
            .filter(|pending| pending.nonce == nonce)
            .and_then(|pending| pending.deeplink_tx.take())
    };

    deeplink_tx.is_some_and(|tx| tx.send(code).is_ok())
}

pub fn parse_auth_deeplink(command: &str) -> Option<(String, String)> {
    let url = Url::parse(command.trim()).ok()?;
    if url.scheme() != "modrinth" || url.host_str() != Some("auth") {
        return None;
    }

    parse_callback_query(url.query().unwrap_or(""))
}

fn parse_callback_query(query: &str) -> Option<(String, String)> {
    let mut code = None;
    let mut nonce = None;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if value.len() > MAX_AUTH_PARAM_LEN {
            return None;
        }

        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "nonce" => nonce = Some(value.into_owned()),
            _ => {}
        }
    }

    let code = code.filter(|code| !code.is_empty())?;
    let nonce = nonce.filter(|nonce| !nonce.is_empty())?;
    Some((code, nonce))
}

fn clear_pending_auth(nonce: &str) {
    let pending = {
        let mut current = PENDING_AUTH.lock().unwrap();
        if current
            .as_ref()
            .is_some_and(|pending| pending.nonce == nonce)
        {
            current.take()
        } else {
            None
        }
    };
    if let Some(pending) = pending {
        pending.cancel();
    }
}

fn auth_listener_stopped() -> theseus::Error {
    ErrorKind::OtherError("Auth code listener stopped".into()).into()
}

pub fn normalize_deep_link_command(command: &str) -> String {
    let trimmed = command.trim();
    if let Ok(urls) = serde_json::from_str::<Vec<String>>(trimmed)
        && let Some(url) = urls.into_iter().next()
    {
        return url;
    }

    if let Ok(url) = serde_json::from_str::<String>(trimmed) {
        return url;
    }

    trimmed.to_string()
}

async fn listen(
    listen_socket_tx: oneshot::Sender<Result<SocketAddr, theseus::Error>>,
    expected_nonce: String,
    mut cancel_rx: oneshot::Receiver<()>,
) -> Result<Option<String>, theseus::Error> {
    let listener = match tcp_listen_any_loopback().await {
        Ok(listener) => {
            listen_socket_tx
                .send(listener.local_addr().map_err(|e| {
                    ErrorKind::OtherError(format!(
                        "Failed to get auth code reply socket address: {e}"
                    ))
                    .into()
                }))
                .ok();

            listener
        }
        Err(e) => {
            let error_msg =
                format!("Failed to bind auth code reply socket: {e}");

            listen_socket_tx
                .send(Err(ErrorKind::OtherError(error_msg.clone()).into()))
                .ok();

            return Err(ErrorKind::OtherError(error_msg).into());
        }
    };

    let mut auth_code = Mutex::new(None);

    while auth_code.get_mut().unwrap().is_none() {
        let client_socket = tokio::select! {
            biased;
            _ = &mut cancel_rx => {
                break;
            }
            conn_accept_result = listener.accept() => {
                match conn_accept_result {
                    Ok((socket, _)) => socket,
                    Err(e) => {
                        tracing::warn!("Failed to accept auth code reply: {e}");
                        continue;
                    }
                }
            }
        };

        if let Err(e) = hyper::server::conn::http1::Builder::new()
            .keep_alive(false)
            .header_read_timeout(Duration::from_secs(5))
            .timer(TokioTimer::new())
            .auto_date_header(false)
            .serve_connection(
                TokioIo::new(client_socket),
                hyper::service::service_fn(|req| {
                    handle_reply(req, &auth_code, &expected_nonce)
                }),
            )
            .await
        {
            tracing::warn!("Failed to handle auth code reply: {e}");
        }
    }

    Ok(auth_code.into_inner().unwrap())
}

async fn handle_reply(
    req: hyper::Request<Incoming>,
    auth_code_out: &Mutex<Option<String>>,
    expected_nonce: &str,
) -> Result<hyper::Response<String>, hyper::http::Error> {
    if req.method() != hyper::Method::GET {
        return hyper::Response::builder()
            .status(hyper::StatusCode::METHOD_NOT_ALLOWED)
            .header("Allow", "GET")
            .body("".into());
    }

    // The authorization code is guaranteed to be sent as a "code" query parameter
    // in the request URI query string as per RFC 6749 § 4.1.2
    // Protocol 2 also sends the sign-in nonce so a stale callback cannot finish a later attempt.
    let callback = parse_callback_query(req.uri().query().unwrap_or(""));

    let response = if let Some((auth_code, _)) =
        callback.filter(|(_, nonce)| nonce == expected_nonce)
    {
        *auth_code_out.lock().unwrap() = Some(auth_code);

        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "text/html;charset=utf-8")
            .body(
                include_str!("auth_code_reply/page.html")
                    .replace("{{title}}", "Success")
                    .replace("{{message}}", "You have successfully signed in! You can close this page now."),
            )
    } else {
        hyper::Response::builder()
            .status(hyper::StatusCode::BAD_REQUEST)
            .header("Content-Type", "text/html;charset=utf-8")
            .body(
                include_str!("auth_code_reply/page.html")
                    .replace("{{title}}", "Error")
                    .replace("{{message}}", "Authorization code not found. Please try signing in again."),
            )
    }?;

    Ok(response)
}
