#![doc = include_str!("../README.md")]

use std::{
    io::{Cursor, Read},
    net::IpAddr,
    path::Path,
    sync::Arc,
};

use flate2::read::GzDecoder;
pub use maxminddb::{self, geoip2};

use bytes::Bytes;
use eyre::{Result, bail, eyre};
use modrinth_util::{Context, env_var};
use tokio::fs;
use tracing::{debug, info, warn};

/// MaxMind GeoIP database reader for use as a `web::Data` parameter.
#[derive(Debug, Clone)]
pub struct MaxMind {
    /// Database reader.
    ///
    /// If the backend was not configured with MaxMind, the reader will not be
    /// available.
    pub reader: Option<Arc<maxminddb::Reader<Bytes>>>,
}

impl MaxMind {
    /// Creates a [`MaxMind`] with no reader.
    #[must_use]
    pub const fn none() -> Self {
        Self { reader: None }
    }

    /// Attempts to create a [`MaxMind`] with a MaxMind GeoIP database reader.
    ///
    /// This reads creation and download parameters from environment variables.
    ///
    /// If the database could not be created or downloaded, this will make a
    /// [`MaxMind`] with no reader.
    pub async fn new() -> Self {
        Self {
            reader: init_reader()
                .await
                .inspect_err(|err| {
                    warn!("Failed to initialize MaxMind: {err:#}");
                })
                .map(Arc::new)
                .ok(),
        }
    }

    /// Queries the MaxMind database for the ISO country code of an IP address.
    ///
    /// If MaxMind is not configured or the database could not be read, returns
    /// [`None`].
    pub async fn query_country(&self, ip: impl Into<IpAddr>) -> Option<String> {
        let reader = self.reader.as_ref()?;
        reader
            .lookup::<geoip2::Country>(ip.into())
            .ok()?
            .and_then(|c| c.country)
            .and_then(|c| c.iso_code.map(|s| s.to_string()))
    }
}

/// Creates a [`maxminddb::Reader`] for use in [`MaxMind::reader`].
///
/// # Errors
///
/// Errors if the database is not present, or could not be downloaded (i.e.
/// missing license key).
pub async fn init_reader() -> Result<maxminddb::Reader<Bytes>> {
    let db = if let Ok(db_path) = env_var("MAXMIND_DB") {
        info!("Using MaxMind database at {db_path:?}");

        fs::read(&db_path)
            .await
            .map(Bytes::from)
            .wrap_err_with(|| {
                eyre!("failed to read database from {db_path:?}")
            })?
    } else {
        let account_id = env_var("MAXMIND_ACCOUNT_ID")?;
        let license_key = env_var("MAXMIND_LICENSE_KEY")?;

        let dirs = directories::ProjectDirs::from(
            "com.modrinth",
            "Modrinth",
            "modrinth-backend",
        )
        .wrap_err("failed to get cache directory")?;
        let cache_dir = dirs.cache_dir();
        let db_path = cache_dir.join("geolite.mmdb");

        match fs::read(&db_path).await {
            Ok(db) => {
                info!("Using cached MaxMind database at {db_path:?}");
                Bytes::from(db)
            }
            Err(err) => {
                debug!(
                    "Failed to read MaxMind database from {db_path:?}, will download: {err}"
                );

                let db = download(&account_id, &license_key).await?;

                match write_to_cache(cache_dir, &db_path, &db).await {
                    Ok(()) => {
                        info!("Wrote GeoIP database cache to {db_path:?}");
                    }
                    Err(err) => warn!(
                        "Failed to write GeoIP database cache to {db_path:?}: {err:?}",
                    ),
                }

                info!("Downloaded and cached database");
                db
            }
        }
    };

    maxminddb::Reader::from_source(db).wrap_err("failed to create reader")
}

async fn download(account_id: &str, license_key: &str) -> Result<Bytes> {
    info!("Downloading MaxMind GeoIP database");
    let db = reqwest::Client::new()
        .get("https://download.maxmind.com/geoip/databases/GeoLite2-Country/download?suffix=tar.gz")
        .basic_auth(account_id, Some(license_key))
        .send()
        .await
        .wrap_err("failed to begin downloading GeoIP database")?
        .error_for_status()
        .wrap_err("failed to download GeoIP database")?
        .bytes()
        .await
        .wrap_err("failed to finish downloading GeoIP database")?;

    let db = GzDecoder::new(Cursor::new(db));
    let mut archive = tar::Archive::new(db);

    let entries = archive.entries().wrap_err("failed to read entries")?;
    for entry in entries {
        let mut entry = entry.wrap_err("failed to read entry")?;
        let Ok(path) = entry.header().path() else {
            continue;
        };
        if path.extension().and_then(|x| x.to_str()) == Some("mmdb") {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .wrap_err("failed to read entry")?;
            return Ok(Bytes::from(buf));
        }
    }

    bail!("no entries in archive");
}

async fn write_to_cache(
    cache_dir: &Path,
    db_path: &Path,
    db: &[u8],
) -> Result<()> {
    fs::create_dir_all(cache_dir)
        .await
        .wrap_err("failed to create parent directories")?;
    fs::write(db_path, db)
        .await
        .wrap_err("failed to write to file")?;
    Ok(())
}
