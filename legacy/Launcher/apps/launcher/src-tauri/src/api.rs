use crate::models::{PackManifest, PacksResponse};
use futures_util::StreamExt;
use reqwest::Url;
use std::time::Duration;

/// Built-in local pack list used when update URL is empty or starts with mock://
const FIXTURE_PACKS: &str = include_str!("../fixtures/v1/packs.json");
const FIXTURE_DEMO_MANIFEST: &str = include_str!("../fixtures/v1/packs/demo-friends/manifest.json");
pub const FIXTURE_DEMO_FILE: &[u8] = b"owyx demo pack file v1\n";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_MAX_DOWNLOAD: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct UpdateClient {
    base_url: String,
    http: reqwest::Client,
}

impl UpdateClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, String> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let http = reqwest::Client::builder()
            .user_agent(concat!("Owyx/", env!("CARGO_PKG_VERSION")))
            .timeout(DEFAULT_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .map_err(|e| format!("HTTP client error: {e}"))?;
        Ok(Self { base_url, http })
    }

    pub fn from_config_url(url: Option<&str>) -> Result<Self, String> {
        let base = match url.map(str::trim).filter(|s| !s.is_empty()) {
            Some(u) => u.to_string(),
            None => "mock://local".to_string(),
        };
        Self::new(base)
    }

    pub fn is_mock(&self) -> bool {
        self.base_url.starts_with("mock://")
    }

    pub async fn fetch_packs(&self) -> Result<PacksResponse, String> {
        if self.is_mock() {
            return serde_json::from_str(FIXTURE_PACKS)
                .map_err(|e| format!("Invalid fixture packs.json: {e}"));
        }

        let url = format!("{}/v1/packs", self.base_url);
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch packs: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Packs API error: {e}"))?;
        response
            .json()
            .await
            .map_err(|e| format!("Invalid packs JSON: {e}"))
    }

    pub async fn fetch_manifest(&self, manifest_url: &str) -> Result<PackManifest, String> {
        if self.is_mock() || manifest_url.starts_with("mock://") {
            if manifest_url.contains("demo-friends") || self.is_mock() {
                return serde_json::from_str(FIXTURE_DEMO_MANIFEST)
                    .map_err(|e| format!("Invalid fixture manifest: {e}"));
            }
            return Err(format!("No mock manifest for {manifest_url}"));
        }

        let url = resolve_url(&self.base_url, manifest_url)?;
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch manifest: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Manifest API error: {e}"))?;
        response
            .json()
            .await
            .map_err(|e| format!("Invalid manifest JSON: {e}"))
    }

    #[allow(dead_code)]
    pub async fn download_bytes(
        &self,
        file_url: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, String> {
        self.download_bytes_with_progress(file_url, max_bytes, None)
            .await
    }

    pub async fn download_bytes_with_progress(
        &self,
        file_url: &str,
        max_bytes: u64,
        mut on_progress: Option<&mut (dyn FnMut(u64, Option<u64>) -> Result<(), String> + Send)>,
    ) -> Result<Vec<u8>, String> {
        let max_bytes = max_bytes.max(1).min(512 * 1024 * 1024);
        if file_url.starts_with("mock://") || self.is_mock() {
            if file_url.contains("demo-friends/mods/readme.txt") || file_url.ends_with("readme.txt")
            {
                let bytes = FIXTURE_DEMO_FILE.to_vec();
                if bytes.len() as u64 > max_bytes {
                    return Err("Mock file exceeds size limit".into());
                }
                if let Some(cb) = on_progress.as_mut() {
                    cb(bytes.len() as u64, Some(bytes.len() as u64))?;
                }
                return Ok(bytes);
            }
            return Err(format!("No mock file for {file_url}"));
        }

        let url = resolve_url(&self.base_url, file_url)?;
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to download file: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Download HTTP error: {e}"))?;

        let content_len = response.content_length();
        if let Some(len) = content_len {
            if len > max_bytes {
                return Err(format!(
                    "Remote file too large: {len} bytes (limit {max_bytes})"
                ));
            }
        }

        let mut data = Vec::new();
        let mut stream = response.bytes_stream();
        let mut last_reported = 0u64;
        const MIN_DELTA: u64 = 256 * 1024;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download stream error: {e}"))?;
            if (data.len() as u64).saturating_add(chunk.len() as u64) > max_bytes {
                return Err(format!("Download exceeded size limit ({max_bytes} bytes)"));
            }
            data.extend_from_slice(&chunk);
            if let Some(cb) = on_progress.as_mut() {
                let received = data.len() as u64;
                let min_delta = content_len
                    .map(|t| (t / 200).max(MIN_DELTA))
                    .unwrap_or(MIN_DELTA);
                let at_end = content_len.is_some_and(|t| received >= t);
                if last_reported == 0
                    || at_end
                    || received.saturating_sub(last_reported) >= min_delta
                {
                    last_reported = received;
                    cb(received, content_len)?;
                }
            }
        }
        if let Some(cb) = on_progress.as_mut() {
            let received = data.len() as u64;
            if received != last_reported {
                cb(received, content_len.or(Some(received)))?;
            }
        }
        Ok(data)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

fn resolve_url(base: &str, maybe_relative: &str) -> Result<Url, String> {
    if maybe_relative.starts_with("http://") || maybe_relative.starts_with("https://") {
        return Url::parse(maybe_relative).map_err(|e| format!("Bad URL: {e}"));
    }
    let base = Url::parse(&format!("{base}/")).map_err(|e| format!("Bad base URL: {e}"))?;
    base.join(maybe_relative.trim_start_matches('/'))
        .map_err(|e| format!("Bad relative URL: {e}"))
}

#[allow(dead_code)]
pub fn default_max_download() -> u64 {
    DEFAULT_MAX_DOWNLOAD
}
