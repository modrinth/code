use crate::event::emit::{emit_profile, emit_warning};
use crate::event::ProfilePayloadType;
use crate::state::{DirectoryInfo, ProfileInstallStage, ProjectType};
use futures::{channel::mpsc::channel, SinkExt, StreamExt};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use std::time::Duration;
use tokio::sync::RwLock;

pub type FileWatcher = RwLock<Debouncer<RecommendedWatcher>>;

pub async fn init_watcher() -> crate::Result<FileWatcher> {
    let (mut tx, mut rx) = channel(1);

    let file_watcher = new_debouncer(
        Duration::from_secs_f32(1.0),
        move |res: DebounceEventResult| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
    )?;

    tokio::task::spawn(async move {
        let span = tracing::span!(tracing::Level::INFO, "init_watcher");
        tracing::info!(parent: &span, "Initting watcher");
        while let Some(res) = rx.next().await {
            let _span = span.enter();

            match res {
                Ok(events) => {
                    let mut visited_profiles = Vec::new();

                    events.iter().for_each(|e| {
                        let mut profile_path = None;

                        let mut found = false;
                        for component in e.path.components() {
                            if found {
                                profile_path = Some(
                                    component.as_os_str().to_string_lossy(),
                                );
                                break;
                            }

                            if component.as_os_str()
                                == crate::state::dirs::PROFILES_FOLDER_NAME
                            {
                                found = true;
                            }
                        }

                        if let Some(profile_path) = profile_path {
                            if e.path
                                .components()
                                .any(|x| x.as_os_str() == "crash-reports")
                                && e.path
                                    .extension()
                                    .map(|x| x == "txt")
                                    .unwrap_or(false)
                            {
                                crash_task(profile_path.to_string());
                            } else if !visited_profiles.contains(&profile_path)
                            {
                                let path = profile_path.to_string();
                                tokio::spawn(async move {
                                    let _ = emit_profile(
                                        &path,
                                        ProfilePayloadType::Synced,
                                    )
                                    .await;
                                });
                                visited_profiles.push(profile_path);
                            }
                        }
                    });
                }
                Err(error) => tracing::warn!("Unable to watch file: {error}"),
            }
        }
    });

    Ok(RwLock::new(file_watcher))
}

/// Watches all existing profiles
pub(crate) async fn watch_profiles_init(
    watcher: &FileWatcher,
    dirs: &DirectoryInfo,
) -> crate::Result<()> {
    if let Ok(profiles_dir) = std::fs::read_dir(dirs.profiles_dir()) {
        for profile_dir in profiles_dir {
            if let Ok(file_name) = profile_dir.map(|x| x.file_name()) {
                if let Some(file_name) = file_name.to_str() {
                    if file_name.starts_with(".DS_Store") {
                        continue;
                    };

                    watch_profile(file_name, watcher, dirs).await?;
                }
            }
        }
    }

    Ok(())
}

pub(crate) async fn watch_profile(
    profile_path: &str,
    watcher: &FileWatcher,
    dirs: &DirectoryInfo,
) -> crate::Result<()> {
    let profile_path = dirs.profiles_dir().join(profile_path);

    if profile_path.exists() && profile_path.is_dir() {
        for folder in ProjectType::iterator()
            .map(|x| x.get_folder())
            .chain(["crash-reports"])
        {
            let path = profile_path.join(folder);

            if !path.exists() && !path.is_symlink() {
                crate::util::io::create_dir_all(&path).await?;
            }

            let mut watcher = watcher.write().await;
            watcher.watcher().watch(&path, RecursiveMode::Recursive)?;
        }
    }

    Ok(())
}

fn crash_task(path: String) {
    tokio::task::spawn(async move {
        let res = async {
            let profile = crate::api::profile::get(&path).await?;

            if let Some(profile) = profile {
                // Hide warning if profile is not yet installed
                if profile.install_stage == ProfileInstallStage::Installed {
                    emit_warning(&format!("Profile {} has crashed! Visit the logs page to see a crash report.", profile.name)).await?;
                }
            }

            Ok::<(), crate::Error>(())
        }
            .await;

        match res {
            Ok(()) => {}
            Err(err) => {
                tracing::warn!("Unable to send crash report to frontend: {err}")
            }
        };
    });
}
