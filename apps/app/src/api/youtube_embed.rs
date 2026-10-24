//! Loopback HTTP server for YouTube embeds. The webview's `tauri://` origin sends no `Referer`,
//! which YouTube rejects with "Error 153"; serving each embed from a page on `http://127.0.0.1`
//! gives the player a valid referrer.

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
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
            let Ok((socket, _)) = listener.accept().await else {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            };
            tokio::spawn(
                hyper::server::conn::http1::Builder::new().serve_connection(
                    TokioIo::new(socket),
                    hyper::service::service_fn(handle),
                ),
            );
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

    match embed_url {
        Some(url) => Response::builder()
            .header("Content-Type", "text/html;charset=utf-8")
            .body(include_str!("youtube_embed.html").replace("{{src}}", &url)),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(String::new()),
    }
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
