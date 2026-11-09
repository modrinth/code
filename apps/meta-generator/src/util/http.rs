use std::{
    collections::HashMap,
    num::NonZero,
    sync::{LazyLock, Mutex, OnceLock},
    time::Duration,
};

use anyhow::{Result, anyhow};
use reqwest::{RequestBuilder, Response, StatusCode};
use tracing::warn;

static FAILED_HOSTNAMES: LazyLock<Mutex<HashMap<String, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static MAX_ATTEMPTS_PER_HOSTNAME: OnceLock<NonZero<usize>> = OnceLock::new();

pub fn configure_hostname_attempt_limit(limit: NonZero<usize>) -> Result<()> {
    MAX_ATTEMPTS_PER_HOSTNAME
        .set(limit)
        .map_err(|_| anyhow!("hostname attempt limit already configured"))
}

fn check_hostname_limit(hostname: &str, limit: usize) -> Result<()> {
    let failures = FAILED_HOSTNAMES.lock().unwrap();
    let count = failures.get(hostname).copied().unwrap_or_default();
    anyhow::ensure!(
        count < limit,
        "hostname {hostname} reached its failed request attempt limit ({count}/{limit})"
    );
    Ok(())
}

fn record_hostname_failure(hostname: &str) {
    let mut failures = FAILED_HOSTNAMES.lock().unwrap();
    let count = failures.entry(hostname.to_owned()).or_default();
    *count = count.saturating_add(1);
}

pub trait RequestExt: Sized {
    /// Sends a request, retrying the request if it fails.
    ///
    /// If too many requests for the same hostname fail, all future requests
    /// to that hostname will instantly fail.
    async fn send_retry(self) -> Result<Response>;
}

impl RequestExt for RequestBuilder {
    async fn send_retry(self) -> Result<Response> {
        let built = self
            .try_clone()
            .ok_or_else(|| {
                anyhow!("request cannot be cloned for hostname validation")
            })?
            .build()?;
        let hostname = built
            .url()
            .host_str()
            .ok_or_else(|| anyhow!("request URL has no hostname"))?
            .to_owned();
        let limit = MAX_ATTEMPTS_PER_HOSTNAME
            .get()
            .copied()
            .unwrap_or_else(crate::config::default_max_attempts_per_hostname)
            .get();
        let mut request = self;
        for attempt in 0..=3 {
            check_hostname_limit(&hostname, limit)?;
            let next = request.try_clone();
            let result = request.send().await;
            if result.as_ref().map_or(true, |response| {
                response.status().is_client_error()
                    || response.status().is_server_error()
            }) {
                record_hostname_failure(&hostname);
            }
            let retryable = match &result {
                Ok(response) => {
                    response.status() == StatusCode::TOO_MANY_REQUESTS
                        || response.status().is_server_error()
                }
                Err(error) => {
                    error.is_timeout()
                        || error.is_connect()
                        || error.is_request()
                }
            };
            if !retryable || attempt == 3 {
                return result.map_err(anyhow::Error::from);
            }
            let Some(next) = next else {
                return result.map_err(anyhow::Error::from);
            };
            check_hostname_limit(&hostname, limit)?;
            let delay = Duration::from_secs(1 << attempt);
            match &result {
                Ok(response) => {
                    warn!(url = %response.url(), status = %response.status(), retry = attempt + 1, delay_seconds = delay.as_secs(), "retrying HTTP request")
                }
                Err(error) => {
                    warn!(%error, retry = attempt + 1, delay_seconds = delay.as_secs(), "retrying HTTP request")
                }
            }
            drop(result);
            tokio::time::sleep(delay).await;
            request = next;
        }
        unreachable!()
    }
}

pub trait ResponseExt: Sized {
    async fn error_for_status_ext(self) -> Result<Self>;
}

impl ResponseExt for Response {
    async fn error_for_status_ext(self) -> Result<Self> {
        let status = self.status();
        if status.is_client_error() || status.is_server_error() {
            let text = self
                .text()
                .await
                .unwrap_or_else(|_| "(non-UTF-8 response)".to_string());
            Err(anyhow!(
                "HTTP status client error ({}): {text}",
                status.as_u16()
            ))
        } else {
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
    };

    #[test]
    fn failures_accumulate_per_hostname() {
        let host = "failure-counter.test";
        assert!(check_hostname_limit(host, 2).is_ok());
        record_hostname_failure(host);
        assert!(check_hostname_limit(host, 2).is_ok());
        record_hostname_failure(host);
        assert!(check_hostname_limit(host, 2).is_err());
        assert!(check_hostname_limit("other-host.test", 2).is_ok());
    }

    #[tokio::test]
    async fn blocked_hostname_fails_without_sending() {
        let host = "blocked-host.invalid";
        for _ in 0..crate::config::default_max_attempts_per_hostname().get() {
            record_hostname_failure(host);
        }
        let error = reqwest::Client::new()
            .get(format!("http://{host}/"))
            .send_retry()
            .await
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("reached its failed request attempt limit")
        );
    }

    #[tokio::test]
    async fn counts_http_failures_without_resetting_on_success() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for status in [404, 200] {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(&mut stream);
                loop {
                    let mut line = String::new();
                    assert!(reader.read_line(&mut line).unwrap() > 0);
                    if line == "\r\n" {
                        break;
                    }
                }
                write!(stream, "HTTP/1.1 {status} Test\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            }
        });
        let client = reqwest::Client::new();
        assert_eq!(
            client.get(&url).send_retry().await.unwrap().status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            client.get(&url).send_retry().await.unwrap().status(),
            StatusCode::OK
        );
        assert_eq!(FAILED_HOSTNAMES.lock().unwrap().get("127.0.0.1"), Some(&1));
        server.join().unwrap();
    }
}
