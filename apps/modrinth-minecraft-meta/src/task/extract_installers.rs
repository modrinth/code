use std::{any::type_name, collections::HashMap};

use anyhow::{Context, Result, anyhow};
use jiff::Timestamp;
use serde::{Deserialize, de::DeserializeOwned};
use tracing::{debug, info, info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{
    AppState, model,
    upstream::mojang::{self},
    util::{from_json_slice, from_json_value},
};

pub async fn extract_installers(app: &mut AppState) -> Result<()> {
    let unprocessed_installers = model::ForgelikeInstaller::all()
        .filter(model::ForgelikeInstaller::fields().processed_at().is_none())
        .exec(&mut app.db)
        .context(info_span!("fetching unprocessed Forge-like installers"))
        .await?;
    info!(
        "found {} installers left to process",
        unprocessed_installers.len()
    );

    let mut num_done = 0usize;
    for installer in unprocessed_installers {
        let name = installer.name.clone();
        let sha256 = installer.sha256;
        extract_from_installer(app, installer)
            .context(info_span!("extracting from installer", %name, %sha256))
            .await
            .inspect_err(|err| warn!("error: {err:?}"))
            .ok();

        num_done += 1;
        if num_done.is_multiple_of(10) {
            info!("processed {num_done} installers");
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ModernInstallProfile {
    data: HashMap<String, model::SidedDataEntry>,
    libraries: Vec<mojang::Library>,
    processors: Vec<model::Processor>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyInstallProfile {
    install: LegacyInstall,
    version_info: LegacyVersionInfo,
}

#[derive(Debug, Deserialize)]
struct LegacyInstall {
    minecraft: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyVersionInfo {
    id: String,
    release_time: Timestamp,
    time: Timestamp,
    main_class: Option<String>,
    minecraft_arguments: Option<String>,
    libraries: Vec<mojang::Library>,
    #[serde(rename = "type")]
    ty: mojang::VersionType,
}

impl LegacyInstallProfile {
    fn into_metadata(self) -> model::ProfileMetadata {
        let version = self.version_info;
        let arguments = version.minecraft_arguments.as_ref().map(|args| {
            HashMap::from([(
                mojang::ArgumentType::Game,
                args.split(' ')
                    .map(|arg| mojang::Argument::Normal(arg.to_owned()))
                    .collect(),
            )])
        });

        model::ProfileMetadata {
            id: version.id,
            inherits_from: self.install.minecraft,
            release_time: version.release_time,
            time: version.time,
            main_class: version.main_class,
            minecraft_arguments: version.minecraft_arguments,
            arguments,
            libraries: version.libraries,
            ty: version.ty,
            data: None,
            processors: None,
        }
    }
}

type ZipFileReader = async_zip::base::read::mem::ZipFileReader;

async fn extract_from_installer(
    app: &AppState,
    installer: model::ForgelikeInstaller,
) -> Result<()> {
    let installer_blob = app
        .blobs
        .get(installer.sha256)
        .context(info_span!("fetching installer blob"))
        .await?;
    let zip = ZipFileReader::new(installer_blob)
        .context(info_span!("opening installer zip for reading"))
        .await?;

    let install_profile =
        read_json::<serde_json::Value>(&zip, "install_profile.json")
            .context(info_span!("reading install profile"))
            .await?
            .context("installer is missing install_profile.json")?;
    let version =
        match read_json::<model::ProfileMetadata>(&zip, "version.json")
            .context(info_span!("reading installer version"))
            .await?
        {
            Some(mut version) => {
                let profile: ModernInstallProfile =
                    from_json_value(&install_profile)
                        .context("parsing modern install profile")?;
                version.data = Some(profile.data);
                version.processors = Some(profile.processors);
                version.libraries.extend(profile.libraries.into_iter().map(
                    |mut library| {
                        library.include_in_classpath = false;
                        library
                    },
                ));
                version
            }
            None => {
                let profile: LegacyInstallProfile =
                    from_json_value(&install_profile).context(
                        "parsing legacy install profile without version.json",
                    )?;
                profile.into_metadata()
            }
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
    let t = from_json_slice::<T>(&bytes).with_context(|| {
        anyhow!("entry {path} must be a `{}`", type_name::<T>())
    })?;
    Ok(Some(t))
}
