//! Downloader for Minecraft data

use crate::install::{
    InstallPhaseDetails, InstallPhaseId, InstallProgress,
    InstallProgressReporter,
};
use crate::instance::QuickPlayType;
use crate::launcher::parse_rules;
use crate::{
    event::{
        LoadingBarId,
        emit::{emit_loading, loading_try_for_each_concurrent},
    },
    state::State,
    util::{fetch::*, io},
};
use daedalus::minecraft::{LoggingConfiguration, LoggingSide};
use daedalus::{
    self as d,
    minecraft::{
        Asset, AssetsIndex, Library, Version as GameVersion,
        VersionInfo as GameVersionInfo,
    },
    modded::LoaderVersion,
};
use futures::prelude::*;
use reqwest::Method;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::OnceCell;

#[derive(Clone, Debug)]
pub struct MinecraftDownloadProgress {
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
    current: Arc<AtomicU64>,
    total: u64,
}

impl MinecraftDownloadProgress {
    async fn new(
        reporter: InstallProgressReporter,
        details: InstallPhaseDetails,
        current: u64,
        total: u64,
    ) -> crate::Result<Self> {
        reporter
            .update(
                InstallPhaseId::DownloadingMinecraft,
                Some(InstallProgress { current, total }),
                details.clone(),
            )
            .await?;

        Ok(Self {
            reporter,
            details,
            current: Arc::new(AtomicU64::new(current)),
            total,
        })
    }

    async fn mark_complete(&self) -> crate::Result<()> {
        let current = self.current.fetch_add(1, Ordering::Relaxed) + 1;
        self.reporter
            .update(
                InstallPhaseId::DownloadingMinecraft,
                Some(InstallProgress {
                    current: current.min(self.total),
                    total: self.total,
                }),
                self.details.clone(),
            )
            .await
    }
}

#[tracing::instrument(skip(st, version))]
pub async fn download_minecraft(
    st: &State,
    version: &GameVersionInfo,
    loading_bar: Option<&LoadingBarId>,
    java_arch: &str,
    force: bool,
    minecraft_updated: bool,
    reporter: Option<InstallProgressReporter>,
    phase_details: InstallPhaseDetails,
) -> crate::Result<()> {
    tracing::info!("Downloading Minecraft version {}", version.id);
    let log_config_count = u64::from(
        version
            .logging
            .as_ref()
            .and_then(|x| x.get(&LoggingSide::Client))
            .is_some(),
    );

    // 5
    let assets_index =
        download_assets_index(st, version, loading_bar, force).await?;
    let progress = if let Some(reporter) = reporter {
        Some(
            MinecraftDownloadProgress::new(
                reporter,
                phase_details,
                1,
                2 + log_config_count
                    + assets_index.objects.len() as u64
                    + version.libraries.len() as u64,
            )
            .await?,
        )
    } else {
        None
    };

    let amount = if version.processors.as_ref().is_some_and(|x| !x.is_empty()) {
        25.0
    } else {
        40.0
    };

    tokio::try_join! {
        // Total loading sums to 90/60
        async {
            download_client(st, version, loading_bar, force).await?;
            if let Some(progress) = &progress {
                progress.mark_complete().await?;
            }
            Ok::<_, crate::Error>(())
        }, // 9
        async {
            let had_log_config = download_log_config(st, version, loading_bar, force).await?;
            if had_log_config
                && let Some(progress) = &progress
            {
                progress.mark_complete().await?;
            }
            Ok::<_, crate::Error>(())
        },
        download_assets(st, version.assets == "legacy", &assets_index, loading_bar, amount, force, progress.clone()), // 40
        download_libraries(st, version.libraries.as_slice(), &version.id, loading_bar, amount, java_arch, force, minecraft_updated, progress.clone()) // 40
    }?;

    tracing::info!("Done downloading Minecraft!");
    Ok(())
}

#[tracing::instrument(skip_all, fields(version = version.id.as_str(), loader = ?loader))]

pub async fn download_version_info(
    st: &State,
    version: &GameVersion,
    loader: Option<&LoaderVersion>,
    force: Option<bool>,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<GameVersionInfo> {
    let version_id = loader
        .map_or(version.id.clone(), |it| format!("{}-{}", version.id, it.id));
    tracing::debug!("Loading version info for Minecraft {version_id}");
    let path = st
        .directories
        .version_dir(&version_id)
        .join(format!("{version_id}.json"));

    let res = if path.exists() && !force.unwrap_or(false) {
        io::read(path)
            .err_into::<crate::Error>()
            .await
            .and_then(|ref it| Ok(serde_json::from_slice(it)?))
    } else {
        tracing::info!(
            "Downloading version info for version {} from {}",
            &version.id,
            version.url
        );
        let mut info = fetch_json(
            Method::GET,
            &version.url,
            None,
            None,
            None,
            &st.api_semaphore,
            &st.pool,
        )
        .await?;

        if let Some(loader) = loader {
            let partial: d::modded::PartialVersionInfo = fetch_json(
                Method::GET,
                &loader.url,
                None,
                None,
                None,
                &st.api_semaphore,
                &st.pool,
            )
            .await?;
            info = d::modded::merge_partial_version(partial, info);
        }

        info.id.clone_from(&version_id);

        write(&path, &serde_json::to_vec(&info)?, &st.io_semaphore).await?;
        Ok(info)
    }?;

    if let Some(loading_bar) = loading_bar {
        emit_loading(loading_bar, 5.0, None)?;
    }

    tracing::debug!("Loaded version info for Minecraft {version_id}");
    Ok(res)
}

#[tracing::instrument(skip_all)]

pub async fn download_client(
    st: &State,
    version_info: &GameVersionInfo,
    loading_bar: Option<&LoadingBarId>,
    force: bool,
) -> crate::Result<()> {
    let version = &version_info.id;
    tracing::debug!("Locating client for version {version}");
    let client_download = version_info
        .downloads
        .get(&d::minecraft::DownloadType::Client)
        .ok_or(
            crate::ErrorKind::LauncherError(format!(
                "No client downloads exist for version {version}"
            ))
            .as_error(),
        )?;
    let path = st
        .directories
        .version_dir(version)
        .join(format!("{version}.jar"));

    if !path.exists() || force {
        let bytes = fetch(
            &client_download.url,
            Some(&client_download.sha1),
            None,
            None,
            &st.fetch_semaphore,
            &st.pool,
        )
        .await?;
        write(&path, &bytes, &st.io_semaphore).await?;
        tracing::trace!("Fetched client version {version}");
    }
    if let Some(loading_bar) = loading_bar {
        emit_loading(loading_bar, 9.0, None)?;
    }

    tracing::debug!("Client loaded for version {version}!");
    Ok(())
}

#[tracing::instrument(skip_all)]

pub async fn download_assets_index(
    st: &State,
    version: &GameVersionInfo,
    loading_bar: Option<&LoadingBarId>,
    force: bool,
) -> crate::Result<AssetsIndex> {
    tracing::debug!("Loading assets index");
    let path = st
        .directories
        .assets_index_dir()
        .join(format!("{}.json", &version.asset_index.id));

    let res = if path.exists() && !force {
        io::read(path)
            .err_into::<crate::Error>()
            .await
            .and_then(|ref it| Ok(serde_json::from_slice(it)?))
    } else {
        let index = fetch_json(
            Method::GET,
            &version.asset_index.url,
            None,
            None,
            None,
            &st.fetch_semaphore,
            &st.pool,
        )
        .await?;
        write(&path, &serde_json::to_vec(&index)?, &st.io_semaphore).await?;
        tracing::info!("Fetched assets index");
        Ok(index)
    }?;

    if let Some(loading_bar) = loading_bar {
        emit_loading(loading_bar, 5.0, None)?;
    }
    tracing::debug!("Assets index successfully loaded!");
    Ok(res)
}

#[tracing::instrument(skip(st, index))]

pub async fn download_assets(
    st: &State,
    with_legacy: bool,
    index: &AssetsIndex,
    loading_bar: Option<&LoadingBarId>,
    loading_amount: f64,
    force: bool,
    progress: Option<MinecraftDownloadProgress>,
) -> crate::Result<()> {
    tracing::debug!("Loading assets");
    let num_futs = index.objects.len();
    let assets = stream::iter(index.objects.iter())
        .map(Ok::<(&String, &Asset), crate::Error>);

    loading_try_for_each_concurrent(assets,
            None,
            loading_bar,
            loading_amount,
            num_futs,
            None,
            |(name, asset)| {
                let progress = progress.clone();
                async move {
                let hash = &asset.hash;
                let resource_path = st.directories.object_dir(hash);
                let url = format!(
                    "https://resources.download.minecraft.net/{sub_hash}/{hash}",
                    sub_hash = &hash[..2]
                );

                let fetch_cell = OnceCell::<bytes::Bytes>::new();
                tokio::try_join! {
                    async {
                        if !resource_path.exists() || force {
                            let resource = fetch_cell
                                .get_or_try_init(|| fetch(&url, Some(hash), None, None, &st.fetch_semaphore, &st.pool))
                                .await?;
                            write(&resource_path, resource, &st.io_semaphore).await?;
                            tracing::trace!("Fetched asset with hash {hash}");
                        }
                        Ok::<_, crate::Error>(())
                    },
                    async {
                        let resource_path = st.directories.legacy_assets_dir().join(
                            name.replace('/', &String::from(std::path::MAIN_SEPARATOR))
                        );

                        if with_legacy && !resource_path.exists() || force {
                            let resource = fetch_cell
                                .get_or_try_init(|| fetch(&url, Some(hash), None, None, &st.fetch_semaphore, &st.pool))
                                .await?;
                            write(&resource_path, resource, &st.io_semaphore).await?;
                            tracing::trace!("Fetched legacy asset with hash {hash}");
                        }
                        Ok::<_, crate::Error>(())
                    },
                }?;

                tracing::trace!("Loaded asset with hash {hash}");
                if let Some(progress) = &progress {
                    progress.mark_complete().await?;
                }
                Ok(())
                }
            }).await?;
    tracing::debug!("Done loading assets!");
    Ok(())
}

#[tracing::instrument(skip(st, libraries))]
#[allow(clippy::too_many_arguments)]
pub async fn download_libraries(
    st: &State,
    libraries: &[Library],
    version: &str,
    loading_bar: Option<&LoadingBarId>,
    loading_amount: f64,
    java_arch: &str,
    force: bool,
    minecraft_updated: bool,
    progress: Option<MinecraftDownloadProgress>,
) -> crate::Result<()> {
    tracing::debug!("Loading libraries");

    tokio::try_join! {
        io::create_dir_all(st.directories.libraries_dir()),
        io::create_dir_all(st.directories.version_natives_dir(version))
    }?;
    let num_files = libraries.len();
    loading_try_for_each_concurrent(
        stream::iter(libraries.iter()).map(Ok::<&Library, crate::Error>),
        None,
        loading_bar,
        loading_amount,
        num_files,
        None,
        |library| {
            let progress = progress.clone();
            async move {
            if let Some(rules) = &library.rules
                && !parse_rules(
                    rules,
                    java_arch,
                    &QuickPlayType::None,
                    minecraft_updated,
                )
            {
                tracing::trace!("Skipped library {}", &library.name);
                if let Some(progress) = &progress {
                    progress.mark_complete().await?;
                }
                return Ok(());
            }

            if !library.downloadable {
                tracing::trace!(
                    "Skipped non-downloadable library {}",
                    &library.name
                );
                if let Some(progress) = &progress {
                    progress.mark_complete().await?;
                }
                return Ok(());
            }

            // When a library has natives, we only need to download such natives, as PrismLauncher does
            if let Some((os_key, classifiers)) =
                library.natives_os_key_and_classifiers(java_arch)
            {
                let parsed_key = os_key
                    .replace("${arch}", crate::util::platform::ARCH_WIDTH);

                if let Some(native) = classifiers.get(&parsed_key) {
                    let data = fetch(
                        &native.url,
                        Some(&native.sha1),
                        None,
                        None,
                        &st.fetch_semaphore,
                        &st.pool,
                    )
                    .await?;

                    if let Ok(mut archive) =
                        zip::ZipArchive::new(std::io::Cursor::new(&data))
                    {
                        match archive.extract(
                            st.directories.version_natives_dir(version),
                        ) {
                            Ok(_) => tracing::debug!(
                                "Fetched native {}",
                                &library.name
                            ),
                            Err(err) => tracing::error!(
                                "Failed extracting native {}. err: {err}",
                                &library.name
                            ),
                        }
                    } else {
                        tracing::error!(
                            "Failed extracting native {}",
                            &library.name
                        );
                    }
                }
            } else {
                let artifact_path = d::get_path_from_artifact(&library.name)?;
                let path = st.directories.libraries_dir().join(&artifact_path);

                if path.exists() && !force {
                    if let Some(progress) = &progress {
                        progress.mark_complete().await?;
                    }
                    return Ok(());
                }

                if let Some(d::minecraft::LibraryDownloads {
                    artifact: Some(ref artifact),
                    ..
                }) = library.downloads
                    && !artifact.url.is_empty()
                {
                    let bytes = fetch(
                        &artifact.url,
                        Some(&artifact.sha1),
                        None,
                        None,
                        &st.fetch_semaphore,
                        &st.pool,
                    )
                    .await?;
                    write(&path, &bytes, &st.io_semaphore).await?;

                    tracing::trace!(
                        "Fetched library {} to path {:?}",
                        &library.name,
                        &path
                    );
                } else {
                    // We lack an artifact URL, so fall back to constructing one ourselves.
                    // PrismLauncher just ignores the library if this is the case, so it's
                    // probably not needed, but previous code revisions of the Modrinth App
                    // intended to do this, so we keep that behavior for compatibility.

                    let url = format!(
                        "{}{artifact_path}",
                        library
                            .url
                            .as_deref()
                            .unwrap_or("https://libraries.minecraft.net/")
                    );

                    tracing::trace!(
                        "Attempting to fetch {} from {url}",
                        library.name,
                    );

                    // It's OK for this fetch to fail, since the URL might not even be valid.
                    // We're constructing a download URL basically out of thin air, and hoping
                    // that it's valid. Since PrismLauncher ignores the library (see above), a
                    // failed download here is not a fatal condition.
                    //
                    // See DEV-479.
                    match fetch(
                        &url,
                        None,
                        None,
                        None,
                        &st.fetch_semaphore,
                        &st.pool,
                    )
                    .await
                    {
                        Ok(bytes) => {
                            write(&path, &bytes, &st.io_semaphore).await?;

                            tracing::debug!(
                                "Fetched library {} to path {:?}",
                                &library.name,
                                &path
                            );
                        }
                        Err(err) => {
                            tracing::debug!(
                                "Failed to download library {} from {url} - \
                                this is not necessarily an error: {err:#?}",
                                &library.name
                            );
                        }
                    }
                }
            }

            tracing::debug!("Loaded library {}", library.name);
            if let Some(progress) = &progress {
                progress.mark_complete().await?;
            }
            Ok(())
            }
        },
    )
    .await?;

    tracing::debug!("Done loading libraries!");
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn download_log_config(
    st: &State,
    version_info: &GameVersionInfo,
    loading_bar: Option<&LoadingBarId>,
    force: bool,
) -> crate::Result<bool> {
    let log_download = version_info
        .logging
        .as_ref()
        .and_then(|x| x.get(&LoggingSide::Client));
    let Some(LoggingConfiguration::Log4j2Xml {
        file: log_download, ..
    }) = log_download
    else {
        if let Some(loading_bar) = loading_bar {
            emit_loading(loading_bar, 1.0, None)?;
        }
        return Ok(false);
    };

    let path = st.directories.log_configs_dir().join(&log_download.id);

    if !path.exists() || force {
        let bytes = fetch(
            &log_download.url,
            Some(&log_download.sha1),
            None,
            None,
            &st.fetch_semaphore,
            &st.pool,
        )
        .await?;
        write(&path, &bytes, &st.io_semaphore).await?;
        tracing::trace!("Fetched log config {}", log_download.id);
    }
    if let Some(loading_bar) = loading_bar {
        emit_loading(loading_bar, 1.0, None)?;
    }

    tracing::debug!("Log config {} loaded", log_download.id);
    Ok(true)
}
