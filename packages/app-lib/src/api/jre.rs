//! Authentication flow interface
use crate::event::emit::{emit_loading, init_loading};
use crate::install::{
    InstallErrorContext, InstallJavaStep, InstallPhaseDetails, InstallPhaseId,
    InstallProgress, InstallProgressReporter,
};
use crate::state::JavaVersion;
use crate::util::fetch::{
    FetchProgressFn, fetch_advanced, fetch_advanced_with_progress, fetch_json,
};
use bytes::Bytes;
use dashmap::DashMap;
use reqwest::Method;
use serde::Deserialize;
use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use sysinfo::{MemoryRefreshKind, RefreshKind};

use crate::util::io;
use crate::util::jre::extract_java_version;
use crate::{
    ErrorKind, LoadingBarType, State,
    util::jre::{self},
};
use anyhow::{Context, bail};

pub async fn get_java_versions() -> crate::Result<DashMap<u32, JavaVersion>> {
    let state = State::get().await?;

    JavaVersion::get_all(&state.pool).await
}

pub async fn set_java_version(java_version: JavaVersion) -> crate::Result<()> {
    let state = State::get().await?;
    let _runtime_lease = state.content_store.runtime_cache_lock.read().await;
    java_version.upsert(&state.pool).await?;
    Ok(())
}

// Searches for jres on the system given a java version (ex: 1.8, 1.17, 1.18)
// Allow higher allows for versions higher than the given version to be returned ('at least')
pub async fn find_filtered_jres(
    java_version: Option<u32>,
) -> crate::Result<Vec<JavaVersion>> {
    let state = State::get().await?;
    let jres = jre::get_all_jre(&state.directories.java_versions_dir()).await?;

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

/// Older Java runs on x64 under emulation, as Mojang ships no Windows arm64
/// runtime for it.
const WINDOWS_ARM64_MIN_NATIVE_JAVA: u32 = 17;
/// Azul publishes a Windows arm64 JDK for Java 17, but no JRE.
const WINDOWS_ARM64_JDK_ONLY_JAVA: u32 = 17;

#[derive(Debug, PartialEq, Eq)]
struct JavaPackageQuery {
    os: &'static str,
    arch: &'static str,
    archive_type: &'static str,
    package_type: &'static str,
}

impl JavaPackageQuery {
    fn new(os: &'static str, arch: &'static str, java_version: u32) -> Self {
        match (os, arch) {
            ("linux", "aarch64") => Self {
                os: "linux-glibc",
                arch,
                archive_type: "tar.gz",
                package_type: "jre",
            },
            ("linux", "x86_64") => Self {
                os,
                arch,
                archive_type: "tar.gz",
                package_type: "jre",
            },
            ("windows", "aarch64")
                if java_version < WINDOWS_ARM64_MIN_NATIVE_JAVA =>
            {
                Self {
                    os,
                    arch: "x86_64",
                    archive_type: "zip",
                    package_type: "jre",
                }
            }
            ("windows", "aarch64")
                if java_version == WINDOWS_ARM64_JDK_ONLY_JAVA =>
            {
                Self {
                    os,
                    arch,
                    archive_type: "zip",
                    package_type: "jdk",
                }
            }
            _ => Self {
                os,
                arch,
                archive_type: "zip",
                package_type: "jre",
            },
        }
    }
}

static JAVA_INSTALL_LOCK: tokio::sync::Mutex<()> =
    tokio::sync::Mutex::const_new(());

#[cfg(any(not(target_os = "macos"), test))]
fn java_executable(install_dir: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        install_dir
            .join("Contents")
            .join("Home")
            .join("bin")
            .join("java")
    }

    #[cfg(not(target_os = "macos"))]
    {
        install_dir.join("bin").join(jre::JAVA_BIN)
    }
}

/// Resolves Java within an extracted macOS package.
///
/// Most Azul packages place `Contents/Home/bin/java` directly inside the package directory,
/// while Java 16 and 18 nest it inside a bundle such as `zulu-16.jre/Contents/Home/bin/java`.
/// Prefers the direct layout, otherwise requires exactly one matching `.jre` bundle.
#[cfg(any(target_os = "macos", test))]
fn macos_java_executable(install_dir: &Path) -> crate::Result<PathBuf> {
    let direct = install_dir.join("Contents/Home/bin/java");
    if direct.is_file() {
        return Ok(direct);
    }

    let entries = std::fs::read_dir(install_dir)
        .map_err(|err| io::IOError::with_path(err, install_dir))?;
    let mut candidate = None;
    for entry in entries {
        let entry =
            entry.map_err(|err| io::IOError::with_path(err, install_dir))?;
        let bundle = entry.path();
        if bundle
            .extension()
            .is_some_and(|extension| extension == "jre")
            && bundle.is_dir()
        {
            let java = bundle.join("Contents/Home/bin/java");
            if java.is_file() {
                if candidate.is_some() {
                    bail!(ErrorKind::InputError(
                        "Java archive contains multiple Java runtime bundles"
                            .to_string()
                    ));
                }
                candidate = Some(java);
            }
        }
    }
    candidate.context(ErrorKind::InputError(
        "Java archive does not contain the expected executable".to_string(),
    ))
}

const INCOMPLETE_INSTALL_MARKER: &str = ".modrinth-incomplete";

/// Extracts a Java archive into place unless a complete, working copy of the
/// same package is already installed.
///
/// A marker file inside the package directory flags an extraction in progress,
/// so an install interrupted by a crash or a full disk is never reused.
async fn install_java_archive(
    file: Bytes,
    archive_type: &'static str,
    directory_name: &str,
    java_versions_dir: &Path,
) -> crate::Result<PathBuf> {
    io::create_dir_all(java_versions_dir).await?;
    let install_dir = java_versions_dir.join(directory_name);
    let marker = install_dir.join(INCOMPLETE_INSTALL_MARKER);
    let _install_guard = JAVA_INSTALL_LOCK.lock().await;
    if !marker.exists()
        && let Ok(java_path) = installed_java_executable(&install_dir)
        && jre::check_java_at_filepath(&java_path).await.is_ok()
    {
        return Ok(java_path);
    }
    if install_dir.exists() {
        io::remove_dir_all(&install_dir).await?;
    }

    io::create_dir_all(&install_dir).await?;
    io::write(&marker, b"").await?;
    let result = extract_java_archive(
        file,
        archive_type,
        directory_name,
        java_versions_dir,
        &install_dir,
    )
    .await;
    match result {
        Ok(java_path) => {
            io::remove_file(&marker).await?;
            Ok(java_path)
        }
        Err(err) => {
            if let Err(cleanup_err) = io::remove_dir_all(&install_dir).await {
                tracing::warn!(
                    "Failed to remove incomplete Java install at {}: {cleanup_err}",
                    install_dir.display()
                );
            }
            Err(err)
        }
    }
}

/// Extracts a Java archive into `java_versions_dir`, after checking that every
/// entry lies inside `directory_name` so no other installed package is touched.
async fn extract_java_archive(
    file: Bytes,
    archive_type: &'static str,
    directory_name: &str,
    java_versions_dir: &Path,
    install_dir: &Path,
) -> crate::Result<PathBuf> {
    let target = java_versions_dir.to_path_buf();
    let directory_name = directory_name.to_string();
    tokio::task::spawn_blocking(move || {
        if archive_type == "tar.gz" {
            let read_error = || {
                ErrorKind::InputError("failed to read java tar.gz".to_string())
            };
            let mut archive =
                tar::Archive::new(flate2::read::GzDecoder::new(file.as_ref()));
            for entry in archive.entries().with_context(read_error)? {
                let entry = entry.with_context(read_error)?;
                let kind = entry.header().entry_type();
                if kind.is_pax_global_extensions()
                    || kind.is_pax_local_extensions()
                {
                    continue;
                }
                ensure_entry_in_package(
                    &entry.path().with_context(read_error)?,
                    &directory_name,
                )?;
            }

            let decoder = flate2::read::GzDecoder::new(file.as_ref());
            tar::Archive::new(decoder).unpack(&target).context(
                ErrorKind::InputError(
                    "failed to extract java tar.gz".to_string(),
                ),
            )?;
        } else {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(file))
                .context(ErrorKind::InputError(
                    "failed to read java zip".to_string(),
                ))?;
            for index in 0..archive.len() {
                let entry = archive.by_index_raw(index).context(
                    ErrorKind::InputError(
                        "failed to read java zip".to_string(),
                    ),
                )?;
                let path =
                    entry.enclosed_name().context(ErrorKind::InputError(
                        "Java archive contains an entry with an unsafe path"
                            .to_string(),
                    ))?;
                ensure_entry_in_package(&path, &directory_name)?;
            }
            archive.extract(&target).context(ErrorKind::InputError(
                "failed to extract java zip".to_string(),
            ))?;
        }
        crate::Result::Ok(())
    })
    .await??;

    installed_java_executable(install_dir)
}

fn ensure_entry_in_package(
    path: &Path,
    directory_name: &str,
) -> crate::Result<()> {
    let mut components = path
        .components()
        .filter(|component| !matches!(component, Component::CurDir));
    match components.next() {
        None => Ok(()),
        Some(Component::Normal(first)) if first == directory_name => Ok(()),
        _ => bail!(ErrorKind::InputError(format!(
            "Java archive entry {} is outside the {directory_name} package",
            path.display()
        ))),
    }
}

fn installed_java_executable(install_dir: &Path) -> crate::Result<PathBuf> {
    #[cfg(target_os = "macos")]
    let java = macos_java_executable(install_dir)?;
    #[cfg(not(target_os = "macos"))]
    let java = java_executable(install_dir);
    if !java.is_file() {
        bail!(ErrorKind::InputError(
            "Java archive does not contain the expected executable".to_string()
        ));
    }
    Ok(java)
}

async fn auto_install_java_inner(
    java_version: u32,
    show_loading: bool,
    reporter: Option<InstallProgressReporter>,
) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let _runtime_lease = state.content_store.runtime_cache_lock.read().await;

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
    let JavaPackageQuery {
        os,
        arch,
        archive_type,
        package_type,
    } = JavaPackageQuery::new(
        std::env::consts::OS,
        std::env::consts::ARCH,
        java_version,
    );
    let metadata_url = format!(
        "https://api.azul.com/metadata/v1/zulu/packages?arch={arch}&java_version={java_version}&os={os}&archive_type={archive_type}&javafx_bundled=false&java_package_type={package_type}&crac_supported=false&release_status=ga&availability_types=CA&page_size=1",
    );
    if let Some(reporter) = &reporter {
        reporter
            .set_context(
                InstallErrorContext::new("fetch Java package metadata")
                    .urls(vec![metadata_url.clone()])
                    .java_version(java_version)
                    .os(std::env::consts::OS)
                    .arch(std::env::consts::ARCH)
                    .build(),
            )
            .await?;
    }
    let packages = fetch_json::<Vec<Package>>(
        Method::GET,
        &metadata_url,
        None,
        None,
        None,
        &state.fetch_semaphore,
        &state.pool,
    )
    .await?;
    if let Some(loading_bar) = &loading_bar {
        emit_loading(loading_bar, 10.0, Some("Downloading java version"))?;
    }

    if let Some(download) = packages.first() {
        if let Some(reporter) = &reporter {
            reporter
                .set_context(
                    InstallErrorContext::new("download Java archive")
                        .urls(vec![download.download_url.clone()])
                        .file_path(download.name.display().to_string())
                        .java_version(java_version)
                        .os(std::env::consts::OS)
                        .arch(std::env::consts::ARCH)
                        .build(),
                )
                .await?;
        }
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
            let mut progress = move |current: u64,
                                     total: u64|
                  -> Pin<
                Box<dyn Future<Output = crate::Result<()>> + Send>,
            > {
                let min_delta =
                    (total / 200).max(JAVA_DOWNLOAD_PROGRESS_MIN_BYTES);
                if current < total
                    && current.saturating_sub(last_reported_bytes) < min_delta
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

        if let Some(reporter) = &reporter {
            reporter
                .set_context(
                    InstallErrorContext::new("read Java archive")
                        .urls(vec![download.download_url.clone()])
                        .file_path(download.name.display().to_string())
                        .target_path(path.display().to_string())
                        .java_version(java_version)
                        .os(std::env::consts::OS)
                        .arch(std::env::consts::ARCH)
                        .build(),
                )
                .await?;
        }
        let directory_name = download
            .name
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(&format!(".{archive_type}")))
            .filter(|dir| {
                !dir.contains(['\\', ':'])
                    && matches!(
                        std::path::Path::new(dir).components().next(),
                        Some(std::path::Component::Normal(_))
                    )
            })
            .context(ErrorKind::InputError(
                "invalid Java archive directory".to_string(),
            ))?;

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
        if let Some(reporter) = &reporter {
            reporter
                .set_context(
                    InstallErrorContext::new("extract Java archive")
                        .urls(vec![download.download_url.clone()])
                        .file_path(download.name.display().to_string())
                        .target_path(path.display().to_string())
                        .java_version(java_version)
                        .os(std::env::consts::OS)
                        .arch(std::env::consts::ARCH)
                        .build(),
                )
                .await?;
        }
        let java_path =
            install_java_archive(file, archive_type, directory_name, &path)
                .await?;
        if let Some(loading_bar) = &loading_bar {
            emit_loading(loading_bar, 10.0, Some("Done extracting java"))?;
        }

        Ok(java_path)
    } else {
        bail!(ErrorKind::LauncherError(format!(
            "no Java Version found for Java version {java_version}, OS {os}, and Architecture {arch}",
        )))
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

#[cfg(test)]
mod tests {
    use super::JavaPackageQuery;
    #[cfg(unix)]
    use super::{
        INCOMPLETE_INSTALL_MARKER, install_java_archive, java_executable,
    };
    #[cfg(unix)]
    use bytes::Bytes;
    #[cfg(unix)]
    use std::{io::Write, os::unix::fs::PermissionsExt, path::Path};

    #[cfg(unix)]
    const PACKAGE: &str = "zulu-test";
    #[cfg(unix)]
    const WORKING_JAVA: &[u8] =
        b"#!/bin/sh\necho os.arch=aarch64\necho java.version=21.0.1\n";
    #[cfg(unix)]
    const BROKEN_JAVA: &[u8] = b"#!/bin/sh\nexit 1\n";

    #[cfg(unix)]
    fn java_entry() -> String {
        java_executable(Path::new(PACKAGE))
            .to_string_lossy()
            .into_owned()
    }

    #[cfg(unix)]
    fn tar_gz(path: &str, contents: &[u8]) -> Bytes {
        tar_gz_entries(&[(path, contents)])
    }

    #[cfg(unix)]
    fn tar_gz_entries(entries: &[(&str, &[u8])]) -> Bytes {
        let encoder = flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        );
        let mut builder = tar::Builder::new(encoder);
        for (path, contents) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, path, *contents).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap().into()
    }

    #[cfg(unix)]
    fn zip_archive(path: &str, contents: &[u8]) -> Bytes {
        zip_entries(&[(path, contents)])
    }

    #[cfg(unix)]
    fn zip_entries(entries: &[(&str, &[u8])]) -> Bytes {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);
        for (path, contents) in entries {
            writer.start_file(*path, options).unwrap();
            writer.write_all(contents).unwrap();
        }
        writer.finish().unwrap().into_inner().into()
    }

    #[cfg(unix)]
    fn write_java(
        java_versions_dir: &Path,
        contents: &[u8],
    ) -> std::path::PathBuf {
        let java = java_executable(&java_versions_dir.join(PACKAGE));
        std::fs::create_dir_all(java.parent().unwrap()).unwrap();
        std::fs::write(&java, contents).unwrap();
        std::fs::set_permissions(&java, std::fs::Permissions::from_mode(0o755))
            .unwrap();
        java
    }

    #[cfg(unix)]
    fn assert_only_package_installed(java_versions_dir: &Path) {
        let entries = std::fs::read_dir(java_versions_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(entries, [PACKAGE]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn installs_archives_with_executable_java() {
        for (archive_type, file) in [
            ("tar.gz", tar_gz(&java_entry(), WORKING_JAVA)),
            ("zip", zip_archive(&java_entry(), WORKING_JAVA)),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let java =
                install_java_archive(file, archive_type, PACKAGE, dir.path())
                    .await
                    .unwrap();
            assert_eq!(java, java_executable(&dir.path().join(PACKAGE)));
            assert_eq!(std::fs::read(&java).unwrap(), WORKING_JAVA);
            let mode = std::fs::metadata(&java).unwrap().permissions().mode();
            assert_ne!(mode & 0o111, 0);
            assert_only_package_installed(dir.path());
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn broken_install_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let java = write_java(dir.path(), BROKEN_JAVA);
        let stale = dir.path().join(PACKAGE).join("stale.txt");
        std::fs::write(&stale, "stale").unwrap();
        let file = tar_gz(&java_entry(), WORKING_JAVA);
        install_java_archive(file, "tar.gz", PACKAGE, dir.path())
            .await
            .unwrap();
        assert_eq!(std::fs::read(java).unwrap(), WORKING_JAVA);
        assert!(!stale.exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn working_install_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        write_java(dir.path(), WORKING_JAVA);
        let marker = dir.path().join(PACKAGE).join("in-use.txt");
        std::fs::write(&marker, "in use").unwrap();
        let file = tar_gz(&java_entry(), WORKING_JAVA);
        install_java_archive(file, "tar.gz", PACKAGE, dir.path())
            .await
            .unwrap();
        assert!(marker.exists());
        assert_only_package_installed(dir.path());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn incomplete_install_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        write_java(dir.path(), WORKING_JAVA);
        let marker = dir.path().join(PACKAGE).join(INCOMPLETE_INSTALL_MARKER);
        std::fs::write(&marker, "").unwrap();
        let stale = dir.path().join(PACKAGE).join("stale.txt");
        std::fs::write(&stale, "stale").unwrap();
        let file = tar_gz(&java_entry(), WORKING_JAVA);
        install_java_archive(file, "tar.gz", PACKAGE, dir.path())
            .await
            .unwrap();
        assert!(!stale.exists());
        assert!(!marker.exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn failed_extraction_leaves_no_partial_install() {
        let dir = tempfile::tempdir().unwrap();
        let file = tar_gz(&format!("{PACKAGE}/readme.txt"), b"no java here");
        install_java_archive(file, "tar.gz", PACKAGE, dir.path())
            .await
            .unwrap_err();
        assert!(!dir.path().join(PACKAGE).exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn entries_outside_package_are_rejected_before_writing() {
        let java = java_entry();
        let outside = "other-package/bin/java";
        for (archive_type, file) in [
            (
                "tar.gz",
                tar_gz_entries(&[
                    (&java, WORKING_JAVA),
                    (outside, BROKEN_JAVA),
                ]),
            ),
            (
                "zip",
                zip_entries(&[(&java, WORKING_JAVA), (outside, BROKEN_JAVA)]),
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            install_java_archive(file, archive_type, PACKAGE, dir.path())
                .await
                .unwrap_err();
            assert!(!dir.path().join("other-package").exists());
            assert!(!dir.path().join(PACKAGE).exists());
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn parallel_installs_of_one_package_all_succeed() {
        let dir = tempfile::tempdir().unwrap();
        let file = tar_gz(&java_entry(), WORKING_JAVA);
        let (first, second, third) = tokio::join!(
            install_java_archive(file.clone(), "tar.gz", PACKAGE, dir.path()),
            install_java_archive(file.clone(), "tar.gz", PACKAGE, dir.path()),
            install_java_archive(file, "tar.gz", PACKAGE, dir.path()),
        );
        let java = first.unwrap();
        assert_eq!(second.unwrap(), java);
        assert_eq!(third.unwrap(), java);
        assert_only_package_installed(dir.path());
    }

    #[test]
    fn resolves_macos_java_layouts() {
        for relative in [
            "Contents/Home/bin/java",
            "zulu-16.jre/Contents/Home/bin/java",
            "zulu-18.jre/Contents/Home/bin/java",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let java = dir.path().join(relative);
            std::fs::create_dir_all(java.parent().unwrap()).unwrap();
            std::fs::write(&java, b"java").unwrap();
            assert_eq!(super::macos_java_executable(dir.path()).unwrap(), java);
        }
    }

    #[test]
    fn rejects_missing_or_ambiguous_macos_java() {
        let dir = tempfile::tempdir().unwrap();
        assert!(super::macos_java_executable(dir.path()).is_err());
        for bundle in ["first.jre", "second.jre"] {
            let java = dir.path().join(bundle).join("Contents/Home/bin/java");
            std::fs::create_dir_all(java.parent().unwrap()).unwrap();
            std::fs::write(java, b"java").unwrap();
        }
        assert!(
            super::macos_java_executable(dir.path())
                .unwrap_err()
                .to_string()
                .contains("multiple")
        );
    }

    #[test]
    fn linux_arm64_uses_glibc_tarballs() {
        assert_eq!(
            JavaPackageQuery::new("linux", "aarch64", 16),
            JavaPackageQuery {
                os: "linux-glibc",
                arch: "aarch64",
                archive_type: "tar.gz",
                package_type: "jre",
            }
        );
    }

    #[test]
    fn linux_x64_uses_tarballs() {
        for java_version in [8, 16, 17, 21, 25] {
            assert_eq!(
                JavaPackageQuery::new("linux", "x86_64", java_version),
                JavaPackageQuery {
                    os: "linux",
                    arch: "x86_64",
                    archive_type: "tar.gz",
                    package_type: "jre",
                }
            );
        }
    }

    #[test]
    fn windows_arm64_runs_older_java_under_emulation() {
        for java_version in [8, 16] {
            let query =
                JavaPackageQuery::new("windows", "aarch64", java_version);
            assert_eq!(query.arch, "x86_64");
            assert_eq!(query.package_type, "jre");
        }
    }

    #[test]
    fn windows_arm64_uses_native_java_from_17() {
        let query = JavaPackageQuery::new("windows", "aarch64", 17);
        assert_eq!(query.arch, "aarch64");
        assert_eq!(query.package_type, "jdk");
        for java_version in [21, 25] {
            let query =
                JavaPackageQuery::new("windows", "aarch64", java_version);
            assert_eq!(query.arch, "aarch64");
            assert_eq!(query.package_type, "jre");
        }
    }

    #[test]
    fn other_platforms_use_native_zips() {
        for (os, arch) in [("windows", "x86_64"), ("macos", "aarch64")] {
            assert_eq!(
                JavaPackageQuery::new(os, arch, 8),
                JavaPackageQuery {
                    os,
                    arch,
                    archive_type: "zip",
                    package_type: "jre",
                }
            );
        }
    }
}
