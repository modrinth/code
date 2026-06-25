//! Authentication flow interface
use crate::event::emit::{emit_loading, init_loading};
use crate::install::{
    InstallJavaStep, InstallPhaseDetails, InstallPhaseId, InstallProgress,
    InstallProgressReporter,
};
use crate::state::JavaVersion;
use crate::util::fetch::{
    FetchProgressFn, fetch_advanced, fetch_advanced_with_progress, fetch_json,
};
use dashmap::DashMap;
use reqwest::Method;
use serde::Deserialize;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use sysinfo::{MemoryRefreshKind, RefreshKind};

use crate::util::io;
use crate::util::jre::extract_java_version;
use crate::{
    LoadingBarType, State,
    util::jre::{self},
};

pub async fn get_java_versions() -> crate::Result<DashMap<u32, JavaVersion>> {
    let state = State::get().await?;

    JavaVersion::get_all(&state.pool).await
}

pub async fn set_java_version(java_version: JavaVersion) -> crate::Result<()> {
    let state = State::get().await?;
    java_version.upsert(&state.pool).await?;
    Ok(())
}

// Searches for jres on the system given a java version (ex: 1.8, 1.17, 1.18)
// Allow higher allows for versions higher than the given version to be returned ('at least')
pub async fn find_filtered_jres(
    java_version: Option<u32>,
) -> crate::Result<Vec<JavaVersion>> {
    let jres = jre::get_all_jre().await?;

    // Filter out JREs that are not 1.17 or higher
    Ok(if let Some(java_version) = java_version {
        jres.into_iter()
            .filter(|jre| {
                let jre_version = extract_java_version(&jre.version);
                if let Ok(jre_version) = jre_version {
                    jre_version == java_version
                } else {
                    false
                }
            })
            .collect()
    } else {
        jres
    })
}

pub async fn auto_install_java(java_version: u32) -> crate::Result<PathBuf> {
    auto_install_java_with_loading(java_version, true).await
}

pub async fn auto_install_java_with_loading(
    java_version: u32,
    show_loading: bool,
) -> crate::Result<PathBuf> {
    auto_install_java_inner(java_version, show_loading, None).await
}

pub async fn auto_install_java_with_reporter(
    java_version: u32,
    reporter: InstallProgressReporter,
) -> crate::Result<PathBuf> {
    auto_install_java_inner(java_version, false, Some(reporter)).await
}

const JAVA_INSTALL_STEPS: u64 = 4;
const JAVA_DOWNLOAD_PROGRESS_MIN_BYTES: u64 = 256 * 1024;

async fn update_java_install_progress(
    reporter: Option<&InstallProgressReporter>,
    java_version: u32,
    step: InstallJavaStep,
    progress: Option<InstallProgress>,
) -> crate::Result<()> {
    if let Some(reporter) = reporter {
        reporter
            .update(
                InstallPhaseId::PreparingJava,
                progress,
                InstallPhaseDetails::Java {
                    major_version: java_version,
                    step,
                },
            )
            .await?;
    }

    Ok(())
}

fn java_step_progress(current: u64) -> InstallProgress {
    InstallProgress {
        current,
        total: JAVA_INSTALL_STEPS,
        secondary: None,
    }
}

async fn auto_install_java_inner(
    java_version: u32,
    show_loading: bool,
    reporter: Option<InstallProgressReporter>,
) -> crate::Result<PathBuf> {
    let state = State::get().await?;

    let loading_bar = if show_loading {
        Some(
            init_loading(
                LoadingBarType::JavaDownload {
                    version: java_version,
                },
                100.0,
                "Downloading java version",
            )
            .await?,
        )
    } else {
        None
    };

    #[derive(Deserialize)]
    struct Package {
        pub download_url: String,
        pub name: PathBuf,
    }

    if let Some(loading_bar) = &loading_bar {
        emit_loading(loading_bar, 0.0, Some("Fetching java version"))?;
    }
    update_java_install_progress(
        reporter.as_ref(),
        java_version,
        InstallJavaStep::FetchingMetadata,
        Some(java_step_progress(1)),
    )
    .await?;
    let packages = fetch_json::<Vec<Package>>(
                Method::GET,
                &format!(
                    "https://api.azul.com/metadata/v1/zulu/packages?arch={}&java_version={}&os={}&archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1",
                    std::env::consts::ARCH, java_version, std::env::consts::OS
                ),
                None,
                None,
                None,
                &state.fetch_semaphore,
                &state.pool,
            ).await?;
    if let Some(loading_bar) = &loading_bar {
        emit_loading(loading_bar, 10.0, Some("Downloading java version"))?;
    }

    if let Some(download) = packages.first() {
        update_java_install_progress(
            reporter.as_ref(),
            java_version,
            InstallJavaStep::Downloading,
            None,
        )
        .await?;
        let file = if reporter.is_some() {
            let mut last_reported_bytes = 0_u64;
            let download_reporter = reporter.clone();
            let mut progress =
                move |current: u64,
                      total: u64|
                      -> Pin<
                    Box<dyn Future<Output = crate::Result<()>> + Send>,
                > {
                    let min_delta =
                        (total / 200).max(JAVA_DOWNLOAD_PROGRESS_MIN_BYTES);
                    if current < total
                        && current.saturating_sub(last_reported_bytes)
                            < min_delta
                    {
                        return Box::pin(async { Ok(()) });
                    }

                    last_reported_bytes = current;
                    let reporter = download_reporter.clone();
                    Box::pin(async move {
                        update_java_install_progress(
                            reporter.as_ref(),
                            java_version,
                            InstallJavaStep::Downloading,
                            Some(InstallProgress {
                                current,
                                total,
                                secondary: None,
                            }),
                        )
                        .await
                    })
                };

            fetch_advanced_with_progress(
                Method::GET,
                &download.download_url,
                None,
                None,
                None,
                None,
                loading_bar.as_ref().map(|loading_bar| (loading_bar, 80.0)),
                None,
                &state.fetch_semaphore,
                &state.pool,
                Some(&mut progress as &mut FetchProgressFn<'_>),
            )
            .await?
        } else {
            fetch_advanced(
                Method::GET,
                &download.download_url,
                None,
                None,
                None,
                None,
                loading_bar.as_ref().map(|loading_bar| (loading_bar, 80.0)),
                None,
                &state.fetch_semaphore,
                &state.pool,
            )
            .await?
        };

        let path = state.directories.java_versions_dir();

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(file))
            .map_err(|_| {
                crate::Error::from(crate::ErrorKind::InputError(
                    "Failed to read java zip".to_string(),
                ))
            })?;

        // removes the old installation of java
        if let Some(file) = archive.file_names().next()
            && let Some(dir) = file.split('/').next()
        {
            let path = path.join(dir);

            if path.exists() {
                io::remove_dir_all(path).await?;
            }
        }

        if let Some(loading_bar) = &loading_bar {
            emit_loading(loading_bar, 0.0, Some("Extracting java"))?;
        }
        update_java_install_progress(
            reporter.as_ref(),
            java_version,
            InstallJavaStep::Extracting,
            Some(java_step_progress(3)),
        )
        .await?;
        archive.extract(&path).map_err(|_| {
            crate::Error::from(crate::ErrorKind::InputError(
                "Failed to extract java zip".to_string(),
            ))
        })?;
        if let Some(loading_bar) = &loading_bar {
            emit_loading(loading_bar, 10.0, Some("Done extracting java"))?;
        }
        let mut base_path = path.join(
            download
                .name
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        #[cfg(target_os = "macos")]
        {
            base_path = base_path
                .join("Contents")
                .join("Home")
                .join("bin")
                .join("java")
        }

        #[cfg(not(target_os = "macos"))]
        {
            base_path = base_path.join("bin").join(jre::JAVA_BIN)
        }

        Ok(base_path)
    } else {
        Err(crate::ErrorKind::LauncherError(format!(
                    "No Java Version found for Java version {}, OS {}, and Architecture {}",
                    java_version, std::env::consts::OS, std::env::consts::ARCH,
                )).into())
    }
}

// Validates JRE at a given at a given path
pub async fn check_jre(path: PathBuf) -> crate::Result<JavaVersion> {
    jre::check_java_at_filepath(&path).await
}

// Test JRE at a given path
pub async fn test_jre(
    path: PathBuf,
    major_version: u32,
) -> crate::Result<bool> {
    let jre = match jre::check_java_at_filepath(&path).await {
        Ok(jre) => jre,
        Err(e) => {
            tracing::warn!("Invalid Java at {}: {e}", path.display());
            return Ok(false);
        }
    };
    let version = extract_java_version(&jre.version)?;
    tracing::info!(
        "Expected Java version {major_version}, and found {version} at {}",
        path.display()
    );
    Ok(version == major_version)
}

fn system_memory_bytes() -> u64 {
    sysinfo::System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::nothing().with_ram()),
    )
    .total_memory()
}

/// Recommended default max heap (MiB) for new instances based on system RAM.
pub fn default_memory_max_mb() -> u32 {
    const BYTES_PER_GIB: u64 = 1024 * 1024 * 1024;
    let system_gib = system_memory_bytes() / BYTES_PER_GIB;

    if system_gib < 8 {
        1024 * 2
    } else if system_gib >= 24 {
        1024 * 6
    } else {
        1024 * 4
    }
}

// Gets maximum memory in KiB.
pub async fn get_max_memory() -> crate::Result<u64> {
    Ok(system_memory_bytes() / 1024)
}
