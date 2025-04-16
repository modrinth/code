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
                                profile_path = Some(component.as_os_str());
                                break;
                            }

                            if component.as_os_str()
                                == crate::state::dirs::PROFILES_FOLDER_NAME
                            {
                                found = true;
                            }
                        }

                        if let Some(profile_path) = profile_path {
                            let profile_path_str =
                                profile_path.to_string_lossy().to_string();
                            let first_file_name = e
                                .path
                                .components()
                                .skip_while(|x| x.as_os_str() != profile_path)
                                .nth(1)
                                .map(|x| x.as_os_str());
                            if first_file_name
                                .filter(|x| *x == "crash-reports")
                                .is_some()
                                && e.path
                                    .extension()
                                    .filter(|x| *x == "txt")
                                    .is_some()
                            {
                                crash_task(profile_path_str);
                            } else if !visited_profiles.contains(&profile_path)
                            {
                                let event = if first_file_name
                                    .filter(|x| *x == "servers.dat")
                                    .is_some()
                                {
                                    Some(ProfilePayloadType::ServersUpdated)
                                } else if first_file_name
                                    .filter(|x| {
                                        *x == "saves"
                                            && e.path
                                                .file_name()
                                                .filter(|x| *x == "level.dat")
                                                .is_some()
                                    })
                                    .is_some()
                                {
                                    tracing::info!(
                                        "World updated: {}",
                                        e.path.display()
                                    );
                                    Some(ProfilePayloadType::WorldUpdated {
                                        world: e
                                            .path
                                            .parent()
                                            .unwrap()
                                            .file_name()
                                            .unwrap()
                                            .to_string_lossy()
                                            .to_string(),
                                    })
                                } else if first_file_name
                                    .filter(|x| *x == "saves")
                                    .is_none()
                                {
                                    Some(ProfilePayloadType::Synced)
                                } else {
                                    None
                                };
                                if let Some(event) = event {
                                    tokio::spawn(async move {
                                        let _ = emit_profile(
                                            &profile_path_str,
                                            event,
                                        )
                                        .await;
                                    });
                                    visited_profiles.push(profile_path);
                                }
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
) {
    if let Ok(profiles_dir) = std::fs::read_dir(dirs.profiles_dir()) {
        for profile_dir in profiles_dir {
            if let Ok(file_name) = profile_dir.map(|x| x.file_name()) {
                if let Some(file_name) = file_name.to_str() {
                    if file_name.starts_with(".DS_Store") {
                        continue;
                    };

                    watch_profile(file_name, watcher, dirs).await;
                }
            }
        }
    }
}

pub(crate) async fn watch_profile(
    profile_path: &str,
    watcher: &FileWatcher,
    dirs: &DirectoryInfo,
) {
    let profile_path = dirs.profiles_dir().join(profile_path);

    if profile_path.exists() && profile_path.is_dir() {
        for folder in ProjectType::iterator().map(|x| x.get_folder()).chain([
            "crash-reports",
            "saves",
            "servers.dat",
        ]) {
            let path = profile_path.join(folder);

            if !path.exists() && !path.is_symlink() && !folder.contains(".") {
                if let Err(e) = crate::util::io::create_dir_all(&path).await {
                    tracing::error!(
                        "Failed to create directory for watcher {path:?}: {e}"
                    );
                    return;
                }
            }

            let mut watcher = watcher.write().await;
            if let Err(e) =
                watcher.watcher().watch(&path, RecursiveMode::Recursive)
            {
                tracing::error!(
                    "Failed to watch directory for watcher {path:?}: {e}"
                );
                return;
            }
        }
    }
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
