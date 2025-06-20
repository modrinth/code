//! Authentication flow interface
use crate::event::emit::{emit_loading, init_loading};
use crate::state::JavaVersion;
use crate::util::fetch::{fetch_advanced, fetch_json};
use dashmap::DashMap;
use reqwest::Method;
use serde::Deserialize;
use std::path::PathBuf;
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
    let state = State::get().await?;

    let loading_bar = init_loading(
        LoadingBarType::JavaDownload {
            version: java_version,
        },
        100.0,
        "Downloading java version",
    )
    .await?;

    #[derive(Deserialize)]
    struct Package {
        pub download_url: String,
        pub name: PathBuf,
    }

    emit_loading(&loading_bar, 0.0, Some("Fetching java version"))?;
    let packages = fetch_json::<Vec<Package>>(
                Method::GET,
                &format!(
                    "https://api.azul.com/metadata/v1/zulu/packages?arch={}&java_version={}&os={}&archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1",
                    std::env::consts::ARCH, java_version, std::env::consts::OS
                ),
                None,
                None,
                &state.fetch_semaphore,
                &state.pool,
            ).await?;
    emit_loading(&loading_bar, 10.0, Some("Downloading java version"))?;

    if let Some(download) = packages.first() {
        let file = fetch_advanced(
            Method::GET,
            &download.download_url,
            None,
            None,
            None,
            Some((&loading_bar, 80.0)),
            &state.fetch_semaphore,
            &state.pool,
        )
        .await?;

        let path = state.directories.java_versions_dir();

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(file))
            .map_err(|_| {
                crate::Error::from(crate::ErrorKind::InputError(
                    "Failed to read java zip".to_string(),
                ))
            })?;

        // removes the old installation of java
        if let Some(file) = archive.file_names().next() {
            if let Some(dir) = file.split('/').next() {
                let path = path.join(dir);

                if path.exists() {
                    io::remove_dir_all(path).await?;
                }
            }
        }

        emit_loading(&loading_bar, 0.0, Some("Extracting java"))?;
        archive.extract(&path).map_err(|_| {
            crate::Error::from(crate::ErrorKind::InputError(
                "Failed to extract java zip".to_string(),
            ))
        })?;
        emit_loading(&loading_bar, 10.0, Some("Done extracting java"))?;
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
                .join(format!("zulu-{}.jre", java_version))
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
    let Ok(jre) = jre::check_java_at_filepath(&path).await else {
        return Ok(false);
    };
    let version = extract_java_version(&jre.version)?;
    Ok(version == major_version)
}

// Gets maximum memory in KiB.
pub async fn get_max_memory() -> crate::Result<u64> {
    Ok(sysinfo::System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::nothing().with_ram()),
    )
    .total_memory()
        / 1024)
}
