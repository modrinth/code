use anyhow::{Context, Result, anyhow};
use async_zip::base::read::mem::ZipFileReader;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use tracing::{debug, info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{AppState, model};

pub async fn extract_installers(app: &mut AppState) -> Result<()> {
    let unprocessed_installers = model::ForgeInstaller::all()
        .filter(model::ForgeInstaller::fields().processed_at().is_none())
        .exec(&mut app.db)
        .context(info_span!("fetching unprocessed forge installers"))
        .await?;

    for installer in unprocessed_installers {
        let name = installer.name.clone();
        extract_from_installer(app, installer)
            .context(info_span!("extracting from installer", ?name))
            .await
            .inspect_err(|err| warn!("error: {err:?}"))
            .ok();
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionInfo {}

async fn extract_from_installer(
    app: &AppState,
    installer: model::ForgeInstaller,
) -> Result<()> {
    let installer_blob = app
        .blobs
        .get(installer.sha256)
        .context(info_span!("fetching installer blob"))
        .await?;
    let zip = ZipFileReader::new(installer_blob)
        .context(info_span!("opening installer zip for reading"))
        .await?;

    let install_profile = read_json::<Value>(&zip, "install_profile.json")
        .context(info_span!("reading install profile"))
        .await?
        .context("installer is missing install_profile.json")?;
    let version = match read_json::<Value>(&zip, "version.json")
        .context(info_span!("reading installer version"))
        .await?
    {
        Some(version) => version,
        None => install_profile
            .get("versionInfo")
            .filter(|version| version.is_object())
            .cloned()
            .context(
                "installer is missing both version.json and versionInfo",
            )?,
    };
    let embedded_maven_artifacts = zip
        .file()
        .entries()
        .iter()
        .map(|entry| entry.filename().as_str())
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|path| path.starts_with("maven/") && !path.ends_with('/'))
        .map(str::to_owned)
        .collect::<Vec<_>>();

    debug!(
        ?install_profile,
        ?version,
        ?embedded_maven_artifacts,
        "read installer metadata"
    );

    Ok(())
}

async fn read_json<T: DeserializeOwned>(
    zip: &ZipFileReader,
    path: &str,
) -> Result<Option<T>> {
    let index = zip
        .file()
        .entries()
        .iter()
        .position(|entry| entry.filename().as_str().ok() == Some(path));
    let Some(index) = index else {
        return Ok(None);
    };

    let mut reader = zip
        .reader_with_entry(index)
        .context(info_span!("opening installer entry", path))
        .await?;
    let mut bytes = Vec::new();
    reader
        .read_to_end_checked(&mut bytes)
        .context(info_span!("reading installer entry", path))
        .await?;
    let value: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing installer entry {path}"))?;
    if !value.is_object() {
        return Err(anyhow!("installer entry {path} must be a JSON object"));
    }

    let parsed = crate::util::from_json_value(&value)
        .with_context(|| format!("deserializing installer entry {path}"))?;
    Ok(Some(parsed))
}
