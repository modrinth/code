//! Functions for fetching information from the Internet
use super::io::{self, IOError};
use crate::event::LoadingBarId;
use crate::event::emit::emit_loading;
use crate::{ErrorKind, LabrinthError};
use bytes::Bytes;
use chrono::{DateTime, TimeDelta, Utc};
use eyre::{Context, eyre};
use governor::clock::{Clock, DefaultClock};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use parking_lot::Mutex;
use rand::Rng;
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::num::NonZeroU32;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Semaphore;
use tokio::{fs::File, io::AsyncReadExt, io::AsyncWriteExt};
use tracing::{debug, info};

pub const DOWNLOAD_META_HEADER: &str = "modrinth-download-meta";

#[derive(Debug, derive_more::Display, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[display(rename_all = "snake_case")]
pub enum DownloadReason {
    Standalone,
    Dependency,
    Modpack,
    Update,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadMeta {
    pub reason: DownloadReason,
    pub game_version: String,
    pub loader: String,
    pub dependent_on: Option<String>,
}

impl DownloadMeta {
    pub fn to_header_value(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct IoSemaphore(pub Semaphore);
#[derive(Debug)]
pub struct FetchSemaphore(pub Semaphore);

struct FetchFence {
    inner: Mutex<HashMap<&'static str, FenceInner>>,
}

impl FetchFence {
    pub fn is_blocked(&self, key: &'static str) -> bool {
        self.inner
            .lock()
            .entry(key)
            .or_insert_with(FenceInner::new)
            .is_blocked()
    }

    pub fn record_ok(&self, key: &'static str) {
        self.inner
            .lock()
            .entry(key)
            .or_insert_with(FenceInner::new)
            .record_ok()
    }

    pub fn record_fail(&self, key: &'static str) {
        self.inner
            .lock()
            .entry(key)
            .or_insert_with(FenceInner::new)
            .record_fail()
    }

    pub fn latest_block_minutes(&self) -> u32 {
        let now = Utc::now();

        self.inner
            .lock()
            .values()
            .filter_map(|fence| fence.block_until)
            .filter(|until| *until > now)
            .max()
            .map(|until| {
                let seconds = until.signed_duration_since(now).num_seconds();
                (seconds.max(0) as u32).div_ceil(60).max(1)
            })
            .unwrap_or(1)
    }
}

struct FenceInner {
    failures: VecDeque<DateTime<Utc>>,
    block_until: Option<DateTime<Utc>>,
    block_factor: i32,
}

impl FenceInner {
    const FAILURE_WINDOW: TimeDelta = TimeDelta::minutes(3);
    const FAILURE_THRESHOLD: usize = 4;
    const BLOCK_DURATION_MIN_BASE: TimeDelta = TimeDelta::minutes(2);
    const BLOCK_DURATION_MAX_BASE: TimeDelta = TimeDelta::minutes(5);
    const BLOCK_DURATION_MAX_FACTOR: i32 = 3;

    pub fn new() -> Self {
        Self {
            failures: VecDeque::new(),
            block_until: None,
            block_factor: 0,
        }
    }

    pub fn is_blocked(&mut self) -> bool {
        if let Some(until) = self.block_until {
            if until > Utc::now() {
                return true;
            } else {
                self.block_until = None;
            }
        }

        false
    }

    pub fn record_ok(&mut self) {
        self.prune(Utc::now());
    }

    pub fn record_fail(&mut self) {
        self.prune(Utc::now());
        self.failures.push_back(Utc::now());

        if self.failures.len() >= Self::FAILURE_THRESHOLD {
            self.trigger_block();
        }
    }

    /// Blocks further requests for a random duration between the min and max base durations, scaled by a factor
    /// of how many blocks have been triggered in this session.
    ///
    /// As such, for the first block, the duration will be between 2 and 5 minutes.
    /// - For the second block, between 4 and 10 minutes.
    /// - For the third block and any further blocks, between 6 and 15 minutes.
    fn trigger_block(&mut self) {
        self.block_factor =
            i32::min(self.block_factor + 1, Self::BLOCK_DURATION_MAX_FACTOR);

        let min = Self::BLOCK_DURATION_MIN_BASE
            .checked_mul(self.block_factor)
            .unwrap_or(Self::BLOCK_DURATION_MIN_BASE);
        let max = Self::BLOCK_DURATION_MAX_BASE
            .checked_mul(self.block_factor)
            .unwrap_or(Self::BLOCK_DURATION_MAX_BASE);

        let delta_seconds = (max - min).as_seconds_f64()
            * rand::thread_rng().gen_range(0.0..=1.0);
        let duration =
            min + TimeDelta::milliseconds((delta_seconds * 1000.0) as i64);

        self.block_until = Some(Utc::now() + duration);
    }

    /// Removes all failure points older than the failure window
    fn prune(&mut self, now: DateTime<Utc>) {
        let cutoff = now - Self::FAILURE_WINDOW;

        while let Some(&front) = self.failures.front() {
            if front < cutoff {
                self.failures.pop_front();
            } else {
                break;
            }
        }
    }
}

static GLOBAL_FETCH_FENCE: LazyLock<FetchFence> =
    LazyLock::new(|| FetchFence {
        inner: Mutex::new(HashMap::new()),
    });

const API_RETRY_AFTER_FALLBACK: Duration = Duration::from_secs(60);

// This means the unit recovery time will be:
// replenish one unit time in seconds = (60 / (units recovered per minute))
// smooth recovery time = replenish one unit time in seconds * API_RATE_LIMIT_RECOVERY_SIZE.
//
// At 20 it means (60 / 120) * 20 = 10 seconds.
const API_RATE_LIMIT_RECOVERY_SIZE: u32 = 20;

/// Implements request-rate-limit handling as well as a local rate limiter of 120 RPM + 50 burst.
struct ApiRateLimit {
    block_until: Mutex<Option<Instant>>,
    check_lock: Mutex<()>,
    local: Arc<DefaultDirectRateLimiter>,
    recovery_padding: Duration,
}

impl ApiRateLimit {
    fn new() -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(120).unwrap())
            .allow_burst(NonZeroU32::new(50).unwrap());
        let recovery_size =
            API_RATE_LIMIT_RECOVERY_SIZE.min(quota.burst_size().get());

        Self {
            block_until: Mutex::new(None),
            check_lock: Mutex::new(()),
            local: Arc::new(RateLimiter::direct(quota)),
            recovery_padding: quota
                .replenish_interval()
                .saturating_mul(recovery_size.saturating_sub(1)),
        }
    }

    fn check(&self) -> crate::Result<()> {
        let _check_guard = self.check_lock.lock();
        self.ensure_not_blocked()?;

        if let Err(not_until) = self.local.check() {
            // Adds hysteresis to the rate limiting system, ensuring recovery happens
            // for longer but for more units, avoiding the "flapping" effect when running
            // out of units.

            let retry_after = not_until
                .wait_time_from(DefaultClock::default().now())
                .saturating_add(self.recovery_padding);
            info!(
                ?retry_after,
                "Hit builtin rate limiter; waiting for recovery"
            );
            self.block_for(retry_after);

            let retry_in_seconds = self
                .retry_in_seconds()
                .unwrap_or_else(|| duration_seconds_ceil(retry_after));

            return Err(ErrorKind::Ratelimited { retry_in_seconds }.into());
        }

        self.ensure_not_blocked()
    }

    fn handle_response(
        &self,
        response: &reqwest::Response,
    ) -> Option<ErrorKind> {
        if response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS {
            return None;
        }

        debug!("Received 429 response; blocking");

        let retry_after = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| parse_retry_after(value, SystemTime::now()))
            .unwrap_or(API_RETRY_AFTER_FALLBACK);

        self.block_for(retry_after);

        Some(ErrorKind::Ratelimited {
            retry_in_seconds: self.retry_in_seconds().unwrap_or(0),
        })
    }

    fn ensure_not_blocked(&self) -> crate::Result<()> {
        if let Some(retry_in_seconds) = self.retry_in_seconds() {
            debug!("Hit builtin rate limiter; blocking");
            return Err(ErrorKind::Ratelimited { retry_in_seconds }.into());
        }

        Ok(())
    }

    fn block_for(&self, duration: Duration) {
        let Some(block_until) = Instant::now().checked_add(duration) else {
            return;
        };
        let mut current_block = self.block_until.lock();

        if current_block.is_none_or(|current| current < block_until) {
            *current_block = Some(block_until);
        }
    }

    fn retry_in_seconds(&self) -> Option<u64> {
        let mut block_until = self.block_until.lock();
        let remaining = (*block_until)?.checked_duration_since(Instant::now());

        if let Some(remaining) = remaining
            && !remaining.is_zero()
        {
            return Some(duration_seconds_ceil(remaining));
        }

        *block_until = None;
        None
    }
}

static GLOBAL_API_RATE_LIMIT: LazyLock<ApiRateLimit> =
    LazyLock::new(ApiRateLimit::new);

fn parse_retry_after(value: &str, now: SystemTime) -> Option<Duration> {
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    let retry_at = httpdate::parse_http_date(value).ok()?;
    Some(retry_at.duration_since(now).unwrap_or(Duration::ZERO))
}

fn duration_seconds_ceil(duration: Duration) -> u64 {
    duration
        .as_secs()
        .saturating_add(u64::from(duration.subsec_nanos() > 0))
}

fn reqwest_client_builder_with_timeout() -> reqwest::ClientBuilder {
    reqwest_client_builder().read_timeout(Duration::from_secs(30))
}

fn reqwest_client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .tcp_keepalive(Some(Duration::from_secs(10)))
        .user_agent(crate::launcher_user_agent())
}

pub static INSECURE_REQWEST_CLIENT: LazyLock<reqwest::Client> =
    LazyLock::new(|| {
        reqwest_client_builder_with_timeout()
            .build()
            .expect("client configuration should be valid")
    });

pub static REQWEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest_client_builder_with_timeout()
        .https_only(true)
        .build()
        .expect("client configuration should be valid")
});

pub static INSECURE_NO_TIMEOUT_REQWEST_CLIENT: LazyLock<reqwest::Client> =
    LazyLock::new(|| {
        reqwest_client_builder()
            .build()
            .expect("client configuration should be valid")
    });

pub static NO_TIMEOUT_REQWEST_CLIENT: LazyLock<reqwest::Client> =
    LazyLock::new(|| {
        reqwest_client_builder()
            .https_only(true)
            .build()
            .expect("client configuration should be valid")
    });

const FETCH_ATTEMPTS: usize = 2;

pub type FetchProgressFn<'a> = dyn FnMut(
        u64,
        u64,
    ) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + 'a>>
    + Send
    + 'a;

#[derive(Clone, Debug)]
pub struct DownloadedFile {
    path: DownloadedFilePath,
    pub size: u64,
    pub sha1: String,
    pub sha512: String,
    pub reused: bool,
    archive: bool,
}

#[derive(Clone, Debug)]
enum DownloadedFilePath {
    Temporary(Arc<tempfile::TempPath>),
    Stored(crate::state::content_store::BlobLease),
}

impl DownloadedFile {
    pub fn path(&self) -> &Path {
        match &self.path {
            DownloadedFilePath::Temporary(path) => path.as_ref().as_ref(),
            DownloadedFilePath::Stored(blob) => &blob.path,
        }
    }

    pub(crate) fn from_blob(
        blob: crate::state::content_store::BlobLease,
        reused: bool,
    ) -> Self {
        Self {
            reused,
            size: blob.blob.size as u64,
            sha1: blob.blob.sha1.clone(),
            sha512: blob.blob.sha512.clone(),
            archive: blob.blob.relative_path.ends_with("/payload.jar"),
            path: DownloadedFilePath::Stored(blob),
        }
    }

    pub(crate) fn into_staged(self) -> crate::Result<StagedDownload> {
        let DownloadedFilePath::Temporary(path) = self.path else {
            return Err(ErrorKind::InputError(
                "Stored content cannot be staged again".to_string(),
            )
            .into());
        };
        let path = Arc::try_unwrap(path).map_err(|_| {
            ErrorKind::InputError(
                "Downloaded content is still in use and cannot be published"
                    .to_string(),
            )
        })?;
        Ok(StagedDownload {
            path,
            size: self.size,
            sha1: self.sha1,
            sha512: self.sha512,
            archive: self.archive,
        })
    }

    pub(crate) async fn store_blob(
        &self,
        state: &crate::State,
    ) -> crate::Result<crate::state::content_store::BlobLease> {
        match &self.path {
            DownloadedFilePath::Stored(blob) => Ok(blob.clone()),
            DownloadedFilePath::Temporary(_) => {
                state.content_store.ingest_file(self.path()).await
            }
        }
    }

    pub async fn copy_to(
        &self,
        path: &Path,
        semaphore: &IoSemaphore,
    ) -> crate::Result<()> {
        let _permit = semaphore.0.acquire().await?;
        let parent = path.parent().ok_or_else(|| {
            ErrorKind::InputError(
                "Download destination has no parent directory".to_string(),
            )
        })?;
        io::create_dir_all(parent).await?;
        let parent = parent.to_path_buf();
        let temporary = tokio::task::spawn_blocking(move || {
            tempfile::NamedTempFile::new_in(parent)
                .map(|file| file.into_temp_path())
        })
        .await??;
        io::copy(self.path(), &temporary).await?;
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            temporary
                .persist(&path)
                .map_err(|error| IOError::with_path(error.error, &path))
        })
        .await??;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct StagedDownload {
    pub(crate) path: tempfile::TempPath,
    pub(crate) size: u64,
    pub(crate) sha1: String,
    pub(crate) sha512: String,
    pub(crate) archive: bool,
}

enum FetchBody {
    Memory(Bytes),
    File(DownloadedFile),
}

/// Downloads and verifies a temporary file, which is removed when its last owner is dropped.
pub async fn fetch_file(
    url: &str,
    sha1: Option<&str>,
    download_meta: Option<&DownloadMeta>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    progress: Option<&mut FetchProgressFn<'_>>,
) -> crate::Result<DownloadedFile> {
    let body = fetch_advanced_with_target(
        Method::GET,
        url,
        sha1,
        None,
        None,
        None,
        download_meta,
        None,
        uri_path,
        semaphore,
        exec,
        &INSECURE_REQWEST_CLIENT,
        progress,
        true,
        None,
    )
    .await?;
    match body {
        FetchBody::File(file) => Ok(file),
        FetchBody::Memory(_) => unreachable!("requested a file download"),
    }
}

pub async fn fetch_file_mirrors(
    mirrors: &[&str],
    sha1: Option<&str>,
    download_meta: Option<&DownloadMeta>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    progress: Option<&mut FetchProgressFn<'_>>,
) -> crate::Result<DownloadedFile> {
    fetch_file_mirrors_in(
        mirrors,
        sha1,
        download_meta,
        uri_path,
        semaphore,
        exec,
        progress,
        None,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn fetch_file_mirrors_in(
    mirrors: &[&str],
    sha1: Option<&str>,
    download_meta: Option<&DownloadMeta>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    mut progress: Option<&mut FetchProgressFn<'_>>,
    staging: Option<&Path>,
) -> crate::Result<DownloadedFile> {
    if mirrors.is_empty() {
        return Err(
            ErrorKind::InputError("No mirrors provided!".to_string()).into()
        );
    }
    for (index, mirror) in mirrors.iter().enumerate() {
        let body = fetch_advanced_with_target(
            Method::GET,
            mirror,
            sha1,
            None,
            None,
            None,
            download_meta,
            None,
            uri_path,
            semaphore,
            exec,
            &REQWEST_CLIENT,
            progress.as_deref_mut(),
            true,
            staging,
        )
        .await;
        match body {
            Ok(FetchBody::File(file)) => return Ok(file),
            Ok(FetchBody::Memory(_)) => {
                unreachable!("requested a file download")
            }
            Err(error) if index + 1 == mirrors.len() => return Err(error),
            Err(_) => {}
        }
    }
    unreachable!()
}

async fn read_file_response(
    response: reqwest::Response,
    mut progress: Option<&mut FetchProgressFn<'_>>,
    staging: Option<&Path>,
) -> crate::Result<DownloadedFile> {
    use futures::StreamExt;
    let staging = staging.map(Path::to_path_buf).or_else(|| {
        crate::State::get_if_initialized()
            .map(|state| state.directories.store_staging_dir())
    });
    let path = tokio::task::spawn_blocking(move || {
        match staging {
            Some(staging) => tempfile::NamedTempFile::new_in(staging),
            None => tempfile::NamedTempFile::new(),
        }
        .map(|file| file.into_temp_path())
    })
    .await??;
    let mut file = File::create(&path).await?;
    let total = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut hasher = sha1_smol::Sha1::new();
    let mut sha512 = Sha512::new();
    let mut size = 0_u64;
    let mut prefix = [0_u8; 2];
    let mut prefix_len = 0;
    while let Some(chunk) =
        crate::install::control::download_step(stream.next()).await?
    {
        let chunk = chunk?;
        let prefix_count = (prefix.len() - prefix_len).min(chunk.len());
        prefix[prefix_len..prefix_len + prefix_count]
            .copy_from_slice(&chunk[..prefix_count]);
        prefix_len += prefix_count;
        file.write_all(&chunk).await?;
        hasher.update(&chunk);
        sha512.update(&chunk);
        size += chunk.len() as u64;
        if let Some(progress) = progress.as_mut() {
            progress(size, total).await?;
        }
    }
    file.sync_all().await?;
    drop(file);
    Ok(DownloadedFile {
        path: DownloadedFilePath::Temporary(Arc::new(path)),
        reused: false,
        size,
        sha1: hasher.hexdigest(),
        sha512: format!("{:x}", sha512.finalize()),
        archive: prefix_len == prefix.len() && prefix == *b"PK",
    })
}

pub(crate) async fn fetch_content_file(
    state: &crate::State,
    mirrors: &[&str],
    sha512: Option<&str>,
    sha1: Option<&str>,
    size: Option<u64>,
    download_meta: Option<&DownloadMeta>,
    progress: Option<&mut FetchProgressFn<'_>>,
) -> crate::Result<DownloadedFile> {
    let acquired = state
        .content_store
        .acquire(
            mirrors,
            sha512,
            sha1,
            size,
            download_meta,
            &state.fetch_semaphore,
            progress,
        )
        .await?;
    Ok(DownloadedFile::from_blob(acquired.blob, acquired.reused))
}

async fn read_memory_response(
    response: reqwest::Response,
    url: &str,
    loading_bar: Option<(&LoadingBarId, f64)>,
    mut progress: Option<&mut FetchProgressFn<'_>>,
) -> crate::Result<Bytes> {
    let total_size = response.content_length();
    if ((loading_bar.is_none() && progress.is_none()) || total_size.is_none())
        && crate::install::control::CURRENT_INSTALL
            .try_with(|_| ())
            .is_err()
    {
        return response
            .bytes()
            .await
            .wrap_err_with(|| eyre!("failed to read response body from {url}"))
            .map_err(Into::into);
    }
    use futures::StreamExt;
    let total_size = total_size.unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) =
        crate::install::control::download_step(stream.next()).await?
    {
        let chunk = chunk.wrap_err_with(|| {
            eyre!("failed to read response body from {url}")
        })?;
        bytes.extend_from_slice(&chunk);
        if let Some((bar, total)) = loading_bar
            && total_size > 0
        {
            emit_loading(
                bar,
                chunk.len() as f64 / total_size as f64 * total,
                None,
            )?;
        }
        if let Some(progress) = progress.as_mut() {
            progress(bytes.len() as u64, total_size).await?;
        }
    }
    Ok(Bytes::from(bytes))
}

#[tracing::instrument(skip(semaphore))]
pub async fn fetch(
    url: &str,
    sha1: Option<&str>,
    download_meta: Option<&DownloadMeta>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<Bytes> {
    fetch_advanced(
        Method::GET,
        url,
        sha1,
        None,
        None,
        download_meta,
        None,
        uri_path,
        semaphore,
        exec,
    )
    .await
}

#[tracing::instrument(skip(semaphore))]
pub async fn fetch_with_client(
    url: &str,
    sha1: Option<&str>,
    download_meta: Option<&DownloadMeta>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    client: &reqwest::Client,
) -> crate::Result<Bytes> {
    fetch_advanced_with_client(
        Method::GET,
        url,
        sha1,
        None,
        None,
        download_meta,
        None,
        uri_path,
        semaphore,
        exec,
        client,
    )
    .await
}

#[tracing::instrument(skip(json_body, semaphore))]
pub async fn fetch_json<T>(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let result = fetch_advanced(
        method, url, sha1, json_body, None, None, None, uri_path, semaphore,
        exec,
    )
    .await?;
    let value = serde_json::from_slice(&result)?;
    Ok(value)
}

/// Downloads a file with retry and checksum functionality, and a specific
/// [`reqwest::Client`].
#[tracing::instrument(skip(json_body, semaphore))]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_advanced(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    header: Option<(&str, &str)>,
    download_meta: Option<&DownloadMeta>,
    loading_bar: Option<(&LoadingBarId, f64)>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<Bytes> {
    fetch_advanced_with_client(
        method,
        url,
        sha1,
        json_body,
        header,
        download_meta,
        loading_bar,
        uri_path,
        semaphore,
        exec,
        &INSECURE_REQWEST_CLIENT,
    )
    .await
}

#[tracing::instrument(skip(body, semaphore))]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_advanced_bytes(
    method: Method,
    url: &str,
    body: Bytes,
    header: Option<(&str, &str)>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<Bytes> {
    fetch_advanced_with_client_and_progress(
        method,
        url,
        None,
        None,
        Some(body),
        header,
        None,
        None,
        uri_path,
        semaphore,
        exec,
        &INSECURE_REQWEST_CLIENT,
        None,
    )
    .await
}

#[tracing::instrument(skip(json_body, semaphore, progress))]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_advanced_with_progress(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    header: Option<(&str, &str)>,
    download_meta: Option<&DownloadMeta>,
    loading_bar: Option<(&LoadingBarId, f64)>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    progress: Option<&mut FetchProgressFn<'_>>,
) -> crate::Result<Bytes> {
    fetch_advanced_with_client_and_progress(
        method,
        url,
        sha1,
        json_body,
        None,
        header,
        download_meta,
        loading_bar,
        uri_path,
        semaphore,
        exec,
        &INSECURE_REQWEST_CLIENT,
        progress,
    )
    .await
}

/// Downloads a file with retry and checksum functionality
#[tracing::instrument(skip(json_body, semaphore))]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_advanced_with_client(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    header: Option<(&str, &str)>,
    download_meta: Option<&DownloadMeta>,
    loading_bar: Option<(&LoadingBarId, f64)>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    client: &reqwest::Client,
) -> crate::Result<Bytes> {
    fetch_advanced_with_client_and_progress(
        method,
        url,
        sha1,
        json_body,
        None,
        header,
        download_meta,
        loading_bar,
        uri_path,
        semaphore,
        exec,
        client,
        None,
    )
    .await
}

#[tracing::instrument(skip(
    json_body, bytes_body, semaphore, client, progress
))]
#[allow(clippy::too_many_arguments)]
async fn fetch_advanced_with_client_and_progress(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    bytes_body: Option<Bytes>,
    header: Option<(&str, &str)>,
    download_meta: Option<&DownloadMeta>,
    loading_bar: Option<(&LoadingBarId, f64)>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    client: &reqwest::Client,
    progress: Option<&mut FetchProgressFn<'_>>,
) -> crate::Result<Bytes> {
    let body = fetch_advanced_with_target(
        method,
        url,
        sha1,
        json_body,
        bytes_body,
        header,
        download_meta,
        loading_bar,
        uri_path,
        semaphore,
        exec,
        client,
        progress,
        false,
        None,
    )
    .await?;
    match body {
        FetchBody::Memory(bytes) => Ok(bytes),
        FetchBody::File(_) => unreachable!("requested an in-memory response"),
    }
}

async fn fetch_advanced_with_target(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    bytes_body: Option<Bytes>,
    header: Option<(&str, &str)>,
    download_meta: Option<&DownloadMeta>,
    loading_bar: Option<(&LoadingBarId, f64)>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    client: &reqwest::Client,
    mut progress: Option<&mut FetchProgressFn<'_>>,
    to_file: bool,
    file_staging: Option<&Path>,
) -> crate::Result<FetchBody> {
    let _permit =
        crate::install::control::download_step(semaphore.0.acquire()).await??;

    let is_api_url = url.starts_with(env!("MODRINTH_API_URL"))
        || url.starts_with(env!("MODRINTH_API_URL_V3"));
    let fence_key = if is_api_url { uri_path } else { None };

    let creds = if header
        .as_ref()
        .is_none_or(|x| &*x.0.to_lowercase() != "authorization")
        && (url.starts_with("https://cdn.modrinth.com") || is_api_url)
    {
        crate::state::ModrinthCredentials::get_active(exec).await?
    } else {
        None
    };

    let download_meta_header = download_meta
        .map(|m| (DOWNLOAD_META_HEADER.to_string(), m.to_header_value()));

    for attempt in 1..=(FETCH_ATTEMPTS + 1) {
        if is_api_url {
            GLOBAL_API_RATE_LIMIT.check()?;
        }

        if let Some(fence_key) = fence_key
            && GLOBAL_FETCH_FENCE.is_blocked(fence_key)
        {
            return Err(ErrorKind::ApiIsDownError(
                GLOBAL_FETCH_FENCE.latest_block_minutes(),
            )
            .into());
        }

        let mut req = client.request(method.clone(), url);

        if let Some(body) = &json_body {
            req = req.json(body);
        } else if let Some(body) = bytes_body.clone() {
            req = req.body(body);
        }

        if let Some(header) = header {
            req = req.header(header.0, header.1);
        }

        if let Some(ref creds) = creds {
            req = req.header("Authorization", &creds.session);
        }

        if let Some((name, value)) = &download_meta_header {
            tracing::debug!("Sending download analytics: {value}");
            req = req.header(name.as_str(), value.as_str());
        }

        let result = crate::install::control::download_step(req.send()).await?;
        match result {
            Ok(resp) => {
                if is_api_url
                    && let Some(error) =
                        GLOBAL_API_RATE_LIMIT.handle_response(&resp)
                {
                    return Err(error.into());
                }

                if resp.status().is_server_error() {
                    if let Some(fence_key) = fence_key {
                        GLOBAL_FETCH_FENCE.record_fail(fence_key);
                    }

                    if attempt <= FETCH_ATTEMPTS {
                        continue;
                    }
                }

                if resp.status().is_client_error()
                    || resp.status().is_server_error()
                {
                    let status = resp.status();
                    let backup_error = resp.error_for_status_ref().unwrap_err();
                    if let Ok(mut error) = resp.json::<LabrinthError>().await {
                        error.status = Some(status.as_u16());
                        error.method = Some(method.as_str().to_string());
                        error.url = Some(url.to_string());
                        error.route = uri_path.map(str::to_string);
                        return Err(ErrorKind::LabrinthError(error).into());
                    }
                    return Err(backup_error.into());
                }

                let bytes = if to_file {
                    read_file_response(
                        resp,
                        progress.as_deref_mut(),
                        file_staging,
                    )
                    .await
                    .map(FetchBody::File)
                } else {
                    read_memory_response(
                        resp,
                        url,
                        loading_bar,
                        progress.as_deref_mut(),
                    )
                    .await
                    .map(FetchBody::Memory)
                };

                if let Ok(bytes) = bytes {
                    if let Some(sha1) = sha1 {
                        let hash = match &bytes {
                            FetchBody::Memory(bytes) => {
                                sha1_async(bytes.clone()).await?
                            }
                            FetchBody::File(file) => file.sha1.clone(),
                        };
                        if &*hash != sha1 {
                            if attempt <= FETCH_ATTEMPTS {
                                continue;
                            } else {
                                return Err(ErrorKind::HashError(
                                    sha1.to_string(),
                                    hash,
                                )
                                .into());
                            }
                        }
                    }

                    tracing::trace!("Done downloading URL {url}");

                    if let Some(fence_key) = fence_key {
                        GLOBAL_FETCH_FENCE.record_ok(fence_key);
                    }

                    return Ok(bytes);
                } else if attempt <= FETCH_ATTEMPTS {
                    continue;
                } else {
                    bytes?;
                }
            }
            Err(_) if attempt <= FETCH_ATTEMPTS => continue,
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    unreachable!()
}

/// Downloads a file from specified mirrors
#[tracing::instrument(skip(semaphore))]
pub async fn fetch_mirrors(
    mirrors: &[&str],
    sha1: Option<&str>,
    download_meta: Option<&DownloadMeta>,
    uri_path: Option<&'static str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<Bytes> {
    if mirrors.is_empty() {
        return Err(
            ErrorKind::InputError("No mirrors provided!".to_string()).into()
        );
    }

    for (index, mirror) in mirrors.iter().enumerate() {
        let result = fetch_with_client(
            mirror,
            sha1,
            download_meta,
            uri_path,
            semaphore,
            exec,
            &REQWEST_CLIENT,
        )
        .await;

        if result.is_ok() || (result.is_err() && index == (mirrors.len() - 1)) {
            return result;
        }
    }

    unreachable!()
}

/// Posts a JSON to a URL
#[tracing::instrument(skip(json_body, semaphore))]
pub async fn post_json(
    url: &str,
    json_body: serde_json::Value,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<()> {
    let _permit = semaphore.0.acquire().await?;

    let mut req = INSECURE_REQWEST_CLIENT.post(url).json(&json_body);

    if let Some(creds) =
        crate::state::ModrinthCredentials::get_active(exec).await?
    {
        req = req.header("Authorization", &creds.session);
    }

    req.send().await?.error_for_status()?;
    Ok(())
}

pub async fn read_json<T>(
    path: &Path,
    semaphore: &IoSemaphore,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let _permit = semaphore.0.acquire().await?;

    let json = io::read(path).await?;
    let json = serde_json::from_slice::<T>(&json)?;

    Ok(json)
}

#[tracing::instrument(skip(bytes, semaphore))]
pub async fn write(
    path: &Path,
    bytes: &[u8],
    semaphore: &IoSemaphore,
) -> crate::Result<()> {
    let _permit = semaphore.0.acquire().await?;

    if let Some(parent) = path.parent() {
        io::create_dir_all(parent).await?;
    }

    let mut file = File::create(path)
        .await
        .map_err(|e| IOError::with_path(e, path))?;
    file.write_all(bytes)
        .await
        .map_err(|e| IOError::with_path(e, path))?;
    tracing::trace!("Done writing file {}", path.display());
    Ok(())
}

pub async fn copy(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    semaphore: &IoSemaphore,
) -> crate::Result<()> {
    let src: &Path = src.as_ref();
    let dest = dest.as_ref();

    let _permit = semaphore.0.acquire().await?;

    if let Some(parent) = dest.parent() {
        io::create_dir_all(parent).await?;
    }

    io::copy(src, dest).await?;
    tracing::trace!(
        "Done copying file {} to {}",
        src.display(),
        dest.display()
    );
    Ok(())
}

pub async fn sha1_async(bytes: Bytes) -> crate::Result<String> {
    let hash = tokio::task::spawn_blocking(move || {
        sha1_smol::Sha1::from(bytes).hexdigest()
    })
    .await?;

    Ok(hash)
}

pub async fn sha1_file_async(
    path: impl AsRef<Path>,
) -> crate::Result<(u64, String)> {
    sha1_file_async_with_progress(path, |_, _| Ok(())).await
}

pub async fn sha1_file_async_with_progress(
    path: impl AsRef<Path>,
    mut progress: impl FnMut(u64, u64) -> crate::Result<()>,
) -> crate::Result<(u64, String)> {
    let path = path.as_ref();
    // Local files can be multi-gigabyte .mrpacks, so hash them without materializing bytes.
    let mut file = File::open(path)
        .await
        .map_err(|e| IOError::with_path(e, path))?;
    let total = file
        .metadata()
        .await
        .map_err(|e| IOError::with_path(e, path))?
        .len();
    let mut hasher = sha1_smol::Sha1::new();
    let mut size = 0;
    let mut buffer = vec![0; 262144];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .await
            .map_err(|e| IOError::with_path(e, path))?;
        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
        size += bytes_read as u64;
        progress(size, total)?;
    }

    Ok((size, hasher.digest().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeDelta, Utc};

    #[test]
    fn test_fence_block_after_4_fails() {
        // Update tests if the FenceInner constants change

        let mut fence = FenceInner::new();

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(fence.is_blocked());
    }

    #[test]
    fn test_fetch_fence_keys_are_independent() {
        let fence = FetchFence {
            inner: Mutex::new(HashMap::new()),
        };

        for _ in 0..FenceInner::FAILURE_THRESHOLD {
            fence.record_fail("/v3/version_file/:sha1/update");
        }

        assert!(fence.is_blocked("/v3/version_file/:sha1/update"));
        assert!(!fence.is_blocked("/v3/project/:id"));
    }

    #[test]
    fn test_fetch_fence_latest_block_minutes() {
        let fence = FetchFence {
            inner: Mutex::new(HashMap::new()),
        };

        {
            let mut inner = fence.inner.lock();
            inner.insert("/expired", FenceInner::new());
            inner.get_mut("/expired").unwrap().block_until =
                Some(Utc::now() - TimeDelta::minutes(1));
            inner.insert("/short", FenceInner::new());
            inner.get_mut("/short").unwrap().block_until =
                Some(Utc::now() + TimeDelta::seconds(61));
            inner.insert("/long", FenceInner::new());
            inner.get_mut("/long").unwrap().block_until =
                Some(Utc::now() + TimeDelta::seconds(140));
        }

        assert_eq!(fence.latest_block_minutes(), 3);
    }

    #[test]
    fn test_fence_block_after_4_fails_with_oks() {
        // Update tests if the FenceInner constants change

        let mut fence = FenceInner::new();

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_ok();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(fence.is_blocked());
    }

    #[test]
    fn test_fence_not_blocked_after_fails_expire() {
        // Update tests if the FenceInner constants change

        let mut fence = FenceInner::new();

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.prune(Utc::now() + TimeDelta::seconds(60 * 3 + 55)); // Should prune all failures

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(!fence.is_blocked());

        fence.record_fail();
        assert!(fence.is_blocked());
    }

    #[test]
    fn test_fence_trigger_block_windows() {
        // brute force flukes
        for i in 0..128 {
            let mut fence = FenceInner::new();

            fence.trigger_block();
            assert!(fence.is_blocked(), "Should be blocked (attempt {i})");

            let block_until = fence.block_until.unwrap();
            assert!(
                block_until > Utc::now() + TimeDelta::seconds(60 + 55),
                "Should be more than 2 minutes (with some leeway) (attempt {i})"
            ); // more than 2 minutes (with some leeway)
            assert!(
                block_until < Utc::now() + TimeDelta::seconds(60 * 5),
                "Should be less than 5 minutes (attempt {i})"
            ); // less than 5 minutes

            fence.block_until = None;

            fence.trigger_block();
            let block_until = fence.block_until.unwrap();
            assert!(
                block_until > Utc::now() + TimeDelta::seconds(60 * 3 + 55),
                "Should be more than 4 minutes (with some leeway) (attempt {i})"
            ); // more than 4 minutes (with some leeway)
            assert!(
                block_until < Utc::now() + TimeDelta::seconds(60 * 10),
                "Should be less than 10 minutes (attempt {i})"
            ); // less than 10 minutes

            fence.block_until = None;

            fence.trigger_block();
            let block_until = fence.block_until.unwrap();
            assert!(
                block_until > Utc::now() + TimeDelta::seconds(60 * 5 + 55),
                "Should be more than 6 minutes (with some leeway) (attempt {i})"
            ); // more than 6 minutes (with some leeway)
            assert!(
                block_until < Utc::now() + TimeDelta::seconds(60 * 15),
                "Should be less than 15 minutes (attempt {i})"
            ); // less than 15 minutes
        }
    }
}
