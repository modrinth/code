//! Loopback HTTP server that hosts YouTube embeds for the app webview.
//!
//! The webview loads the app from a custom scheme (e.g. `tauri://localhost`), so cross-origin
//! frames get no `Referer` header, which YouTube rejects with "Error 153: Video player
//! configuration error". Wrapping each embed in a page served over `http://127.0.0.1` gives
//! the player a valid referrer.

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioIo, TokioTimer};
use tokio::net::TcpListener;
use tokio::sync::OnceCell;

use super::Result;

static ORIGIN: OnceCell<String> = OnceCell::const_new();

/// Returns the origin of the embed server, starting it on first use.
#[tauri::command]
pub async fn get_youtube_embed_origin() -> Result<String> {
    Ok(ORIGIN.get_or_try_init(start).await?.clone())
}

async fn start() -> std::io::Result<String> {
    let listener =
        TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
    let origin = format!("http://{}", listener.local_addr()?);

    tokio::spawn(async move {
        loop {
            let socket = match listener.accept().await {
                Ok((socket, _)) => socket,
                Err(e) => {
                    tracing::warn!(
                        "Failed to accept YouTube embed request: {e}"
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            tokio::spawn(async move {
                if let Err(e) = hyper::server::conn::http1::Builder::new()
                    .header_read_timeout(Duration::from_secs(5))
                    .timer(TokioTimer::new())
                    .serve_connection(
                        TokioIo::new(socket),
                        hyper::service::service_fn(handle),
                    )
                    .await
                {
                    tracing::warn!("Failed to serve YouTube embed: {e}");
                }
            });
        }
    });

    Ok(origin)
}

async fn handle(
    req: Request<Incoming>,
) -> std::result::Result<Response<String>, hyper::http::Error> {
    let embed_url = if req.method() == Method::GET {
        embed_url(req.uri().path(), req.uri().query())
    } else {
        None
    };

    let Some(embed_url) = embed_url else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(String::new());
    };

    Response::builder()
        .header("Content-Type", "text/html;charset=utf-8")
        .header(
            "Content-Security-Policy",
            "default-src 'none'; style-src 'unsafe-inline'; frame-src https://www.youtube.com https://www.youtube-nocookie.com",
        )
        .body(include_str!("youtube_embed.html").replace("{{src}}", &embed_url))
}

/// Maps `[/nocookie]/embed/<video id>?start=<s>&end=<s>` to the YouTube embed URL, keeping
/// only the parameters the markdown sanitizer allows.
fn embed_url(path: &str, query: Option<&str>) -> Option<String> {
    let (host, path) = match path.strip_prefix("/nocookie") {
        Some(path) => ("www.youtube-nocookie.com", path),
        None => ("www.youtube.com", path),
    };
    let id = path.strip_prefix("/embed/")?;
    if id.len() != 11
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return None;
    }

    let params = url::form_urlencoded::parse(query.unwrap_or("").as_bytes())
        .filter(|(key, value)| {
            matches!(key.as_ref(), "start" | "end")
                && !value.is_empty()
                && value.bytes().all(|b| b.is_ascii_digit())
        })
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();

    let mut url = format!("https://{host}/embed/{id}");
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&amp;"));
    }
    Some(url)
}
