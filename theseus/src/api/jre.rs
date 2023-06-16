//! Authentication flow interface
use reqwest::Method;
use serde::Deserialize;
use std::path::PathBuf;

use crate::event::emit::{emit_loading, init_loading};
use crate::util::fetch::{fetch_advanced, fetch_json};
use crate::util::jre::extract_java_majorminor_version;
use crate::{
    state::JavaGlobals,
    util::jre::{self, JavaVersion},
    LoadingBarType, State,
};

pub const JAVA_8_KEY: &str = "JAVA_8";
pub const JAVA_17_KEY: &str = "JAVA_17";
pub const JAVA_18PLUS_KEY: &str = "JAVA_18PLUS";

// Prefer using autodetect_java_globals if from Tauri, due to tauri::command bug
pub async fn get_autodetect_java_globals() -> crate::Result<JavaGlobals> {
    let jres = jre::get_all_jre().await?;
    let java_8 = filter_java8_jres(jres.clone()).await?;
    let java_17 = filter_java17_jres(jres.clone()).await?;
    let java_18plus = filter_java18plus_jres(jres).await?;
    let java_globals =
        autodetect_java_globals(java_8, java_17, java_18plus).await?;
    Ok(java_globals)
}

// Autodetect JavaSettings default
// Using the supplied JavaVersions, autodetects the default JavaSettings
// Make a guess for what the default Java global settings should be
// Currently, this just pops the last one found
pub async fn autodetect_java_globals(
    mut java_8: Vec<JavaVersion>,
    mut java_17: Vec<JavaVersion>,
    mut java_18plus: Vec<JavaVersion>,
) -> crate::Result<JavaGlobals> {
    // Simply select last one found for initial guess
    let mut java_globals = JavaGlobals::new();
    if let Some(jre) = java_8.pop() {
        java_globals.insert(JAVA_8_KEY.to_string(), jre);
    }
    if let Some(jre) = java_17.pop() {
        java_globals.insert(JAVA_17_KEY.to_string(), jre);
    }
    if let Some(jre) = java_18plus.pop() {
        java_globals.insert(JAVA_18PLUS_KEY.to_string(), jre);
    }

    Ok(java_globals)
}

// Searches for jres on the system that are 1.18 or higher
pub async fn filter_java18plus_jres(
    jres: Vec<JavaVersion>,
) -> crate::Result<Vec<JavaVersion>> {
    let version = extract_java_majorminor_version("1.18")?;
    // Filter out JREs that are not 1.17 or higher
    Ok(jres
        .into_iter()
        .filter(|jre| {
            let jre_version = extract_java_majorminor_version(&jre.version);
            if let Ok(jre_version) = jre_version {
                jre_version >= version
            } else {
                false
            }
        })
        .collect())
}

// Searches for jres on the system that are 1.8 exactly
pub async fn filter_java8_jres(
    jres: Vec<JavaVersion>,
) -> crate::Result<Vec<JavaVersion>> {
    let version = extract_java_majorminor_version("1.8")?;
    // Filter out JREs that are not 1.8
    Ok(jres
        .into_iter()
        .filter(|jre| {
            let jre_version = extract_java_majorminor_version(&jre.version);
            if let Ok(jre_version) = jre_version {
                jre_version == version
            } else {
                false
            }
        })
        .collect())
}

// Searches for jres on the system that are 1.17 exactly
pub async fn filter_java17_jres(
    jres: Vec<JavaVersion>,
) -> crate::Result<Vec<JavaVersion>> {
    let version = extract_java_majorminor_version("1.17")?;
    // Filter out JREs that are not 1.8
    Ok(jres
        .into_iter()
        .filter(|jre| {
            let jre_version = extract_java_majorminor_version(&jre.version);
            if let Ok(jre_version) = jre_version {
                jre_version == version
            } else {
                false
            }
        })
        .collect())
}

#[theseus_macros::debug_pin]
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

    emit_loading(&loading_bar, 0.0, Some("Fetching java version")).await?;
    let packages = fetch_json::<Vec<Package>>(
                Method::GET,
                &format!(
                    "https://api.azul.com/metadata/v1/zulu/packages?arch={}&java_version={}&os={}&archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1",
                    std::env::consts::ARCH, java_version, std::env::consts::OS
                ),
                None,
                None,
                &state.fetch_semaphore,
            ).await?;
    emit_loading(&loading_bar, 10.0, Some("Downloading java version")).await?;

    if let Some(download) = packages.first() {
        let file = fetch_advanced(
            Method::GET,
            &download.download_url,
            None,
            None,
            None,
            Some((&loading_bar, 80.0)),
            &state.fetch_semaphore,
        )
        .await?;

        let path = state.directories.java_versions_dir();

        if path.exists() {
            tokio::fs::remove_dir_all(&path).await?;
        }

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(file))
            .map_err(|_| {
                crate::Error::from(crate::ErrorKind::InputError(
                    "Failed to read java zip".to_string(),
                ))
            })?;

        emit_loading(&loading_bar, 0.0, Some("Extracting java")).await?;
        archive.extract(&path).map_err(|_| {
            crate::Error::from(crate::ErrorKind::InputError(
                "Failed to extract java zip".to_string(),
            ))
        })?;
        emit_loading(&loading_bar, 10.0, Some("Done extracting java")).await?;
        Ok(path
            .join(
                download
                    .name
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )
            .join(format!("zulu-{}.jre/Contents/Home/bin/java", java_version)))
    } else {
        Err(crate::ErrorKind::LauncherError(format!(
                    "No Java Version found for Java version {}, OS {}, and Architecture {}",
                    java_version, std::env::consts::OS, std::env::consts::ARCH,
                )).into())
    }
}

// Get all JREs that exist on the system
pub async fn get_all_jre() -> crate::Result<Vec<JavaVersion>> {
    Ok(jre::get_all_jre().await?)
}

pub async fn validate_globals() -> crate::Result<bool> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.java_globals.is_all_valid().await)
}

// Validates JRE at a given at a given path
pub async fn check_jre(path: PathBuf) -> crate::Result<Option<JavaVersion>> {
    Ok(jre::check_java_at_filepath(&path).await)
}

// Gets maximum memory in KiB.
pub async fn get_max_memory() -> crate::Result<u64> {
    Ok(sys_info::mem_info()
        .map_err(|_| {
            crate::Error::from(crate::ErrorKind::LauncherError(
                "Unable to get computer memory".to_string(),
            ))
        })?
        .total)
}
