use std::time::Duration;

use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::OnceCell};
use url::Url;

static EMBED_ORIGIN: OnceCell<String> = OnceCell::const_new();

#[tauri::command]
pub async fn youtube_embed_origin() -> super::Result<String> {
	let origin = EMBED_ORIGIN
		.get_or_try_init(|| async {
			let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
			let origin = format!("http://{}", listener.local_addr()?);
			tauri::async_runtime::spawn(async move {
				loop {
					let socket = match listener.accept().await {
						Ok((socket, _)) => socket,
						Err(error) => {
							tracing::warn!(
								"Failed to accept YouTube embed connection: {error}"
							);
							tokio::time::sleep(Duration::from_secs(1)).await;
							continue;
						}
					};
					tauri::async_runtime::spawn(async move {
						let connection =
							hyper::server::conn::http1::Builder::new()
								.keep_alive(false)
								.serve_connection(
									TokioIo::new(socket),
									hyper::service::service_fn(embed_response),
								);
						let _ = tokio::time::timeout(
							Duration::from_secs(5),
							connection,
						)
						.await;
					});
				}
			});
			Ok::<_, std::io::Error>(origin)
		})
		.await?;
	Ok(origin.clone())
}

async fn embed_response(
	request: Request<Incoming>,
) -> Result<Response<String>, hyper::http::Error> {
	let uri = request.uri();
	let id = uri.path().strip_prefix('/').unwrap_or_default();
	if request.method() != Method::GET
		|| id.len() != 11
		|| !id.bytes().all(|byte| {
			byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
		}) {
		return Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(String::new());
	}

	let mut url =
		Url::parse(&format!("https://www.youtube-nocookie.com/embed/{id}"))
			.unwrap();
	for (key, value) in
		url::form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
	{
		if matches!(key.as_ref(), "start" | "end")
			&& !value.is_empty()
			&& value.bytes().all(|byte| byte.is_ascii_digit())
		{
			url.query_pairs_mut().append_pair(&key, &value);
		}
	}

	Response::builder()
		.header("Content-Type", "text/html; charset=utf-8")
		.header("Referrer-Policy", "strict-origin-when-cross-origin")
		.header("Content-Security-Policy", "default-src 'none'")
		.body(format!(
			"<!doctype html><meta http-equiv=\"refresh\" content=\"0;url={}\">",
			url.as_str().replace('&', "&amp;")
		))
}
