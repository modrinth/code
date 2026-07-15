use crate::State;
use crate::api::instance::{
    CONFIG_DIRECTORY, CONFIG_FILE_EXTENSIONS,
    is_excluded_config_path,
};
use crate::event::InstancePayloadType;
use crate::event::emit::{emit_instance, emit_warning};
use crate::state::{
    DirectoryInfo, InstanceInstallStage, ProjectType, attached_world_data,
};
use crate::worlds::WorldType;
use async_walkdir::{Filtering, WalkDir};
use futures::StreamExt;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{RwLock, mpsc::channel};

use super::adapters::sqlite::instance_rows;

pub struct FileWatcher {
    watcher: RwLock<Debouncer<RecommendedWatcher>>,
    instance_ids: Arc<RwLock<HashMap<String, String>>>,
    known_config_files: Arc<RwLock<HashSet<PathBuf>>>,
}

pub async fn init_watcher() -> crate::Result<FileWatcher> {
    let (tx, mut rx) = channel(1);
    let instance_ids = Arc::new(RwLock::new(HashMap::new()));
    let event_instance_ids = instance_ids.clone();
    let known_config_files = Arc::new(RwLock::new(HashSet::new()));
    let event_known_config_files = known_config_files.clone();

    let file_watcher = new_debouncer(
        Duration::from_secs_f32(1.0),
        move |res: DebounceEventResult| {
            tx.blocking_send(res).ok();
        },
    )?;

    tokio::task::spawn(async move {
        let span = tracing::span!(tracing::Level::INFO, "init_watcher");
        tracing::info!(parent: &span, "Initing watcher");
        while let Some(res) = rx.recv().await {
            let _span = span.enter();

            match res {
                Ok(events) => {
                    let instance_ids = event_instance_ids.read().await;
                    let mut visited_instances = Vec::new();

                    for e in &events {
                        let mut instance_path = None;

                        let mut found = false;
                        for component in e.path.components() {
                            if found {
                                instance_path = Some(component.as_os_str());
                                break;
                            }

                            if component.as_os_str()
                                == crate::state::dirs::INSTANCES_FOLDER_NAME
                            {
                                found = true;
                            }
                        }

                        if let Some(instance_path) = instance_path {
                            let instance_path_str =
                                instance_path.to_string_lossy().to_string();
                            let Some(instance_id) =
                                instance_ids.get(&instance_path_str).cloned()
                            else {
                                continue;
                            };
                            let first_file_name = e
                                .path
                                .components()
                                .skip_while(|x| x.as_os_str() != instance_path)
                                .nth(1)
                                .map(|x| x.as_os_str());
                            let is_config_path = first_file_name
                                .as_ref()
                                .is_some_and(|x| *x == CONFIG_DIRECTORY);
                            let is_new_config_file = if is_config_path
                                && e.path.is_file()
                                && is_supported_config_file(&e.path)
                                && !is_excluded_config_file(&e.path)
                            {
                                event_known_config_files
                                    .write()
                                    .await
                                    .insert(e.path.clone())
                            } else {
                                false
                            };
                            if first_file_name
                                .as_ref()
                                .is_some_and(|x| *x == "crash-reports")
                                && e.path
                                    .extension()
                                    .as_ref()
                                    .is_some_and(|x| *x == "txt")
                            {
                                crash_task(instance_id);
                            } else if !visited_instances.contains(&instance_id)
                            {
                                let event = if first_file_name
                                    .as_ref()
                                    .is_some_and(|x| *x == "servers.dat")
                                {
                                    Some(InstancePayloadType::ServersUpdated)
                                } else if first_file_name.as_ref().is_some_and(
                                    |x| {
                                        *x == "saves"
                                            && e.path
                                                .file_name()
                                                .as_ref()
                                                .is_some_and(|x| {
                                                    *x == "level.dat"
                                                })
                                    },
                                ) {
                                    tracing::info!(
                                        "World updated: {}",
                                        e.path.display()
                                    );
                                    let world = e
                                        .path
                                        .parent()
                                        .unwrap()
                                        .file_name()
                                        .unwrap()
                                        .to_string_lossy()
                                        .to_string();
                                    if !e.path.is_file() {
                                        let instance_id = instance_id.clone();
                                        let world = world.clone();
                                        tokio::spawn(async move {
                                            if let Ok(state) = State::get().await
												&& let Err(e) = attached_world_data::AttachedWorldData::remove_for_world(
													&instance_id,
													WorldType::Singleplayer,
													&world,
													&state.pool
												).await {
													tracing::warn!("Failed to remove AttachedWorldData for '{world}': {e}")
												}
                                        });
                                    }
                                    Some(InstancePayloadType::WorldUpdated {
                                        world,
                                    })
                                } else if first_file_name.as_ref().is_none_or(
                                    |x| {
                                        *x != "saves"
                                            && (*x != CONFIG_DIRECTORY
                                                || is_new_config_file)
                                    },
                                )
                                {
                                    Some(InstancePayloadType::Synced)
                                } else {
                                    None
                                };
                                if let Some(event) = event {
                                    let emit_instance_id = instance_id.clone();
                                    let mark_shared_stale = matches!(
                                        &event,
                                        InstancePayloadType::Synced
                                    );
                                    tokio::spawn(async move {
                                        if mark_shared_stale
                                            && let Ok(state) =
                                                State::get().await
                                            && let Err(error) =
                                                crate::state::mark_shared_instance_stale(
                                                    &emit_instance_id,
                                                    &state.pool,
                                                )
                                                .await
                                        {
                                            tracing::error!(
                                                "Failed to mark shared instance stale after filesystem sync: {error}"
                                            );
                                        }
                                        let _ = emit_instance(
                                            &emit_instance_id,
                                            event,
                                        )
                                        .await;
                                    });
                                    visited_instances.push(instance_id);
                                }
                            }
                        }
                    }
                }
                Err(error) => tracing::warn!("Unable to watch file: {error}"),
            }
        }
    });

    Ok(FileWatcher {
        watcher: RwLock::new(file_watcher),
        instance_ids,
        known_config_files,
    })
}

pub(crate) async fn watch_instances_init(
    watcher: &FileWatcher,
    dirs: &DirectoryInfo,
    pool: &sqlx::SqlitePool,
) {
    let Ok(instances) = instance_rows::list_instances(pool).await else {
        return;
    };

    for instance in instances {
        watch_instance_folder(&instance.id, &instance.path, watcher, dirs)
            .await;
    }
}

pub(crate) async fn watch_instance_folder(
    instance_id: &str,
    instance_path: &str,
    watcher: &FileWatcher,
    dirs: &DirectoryInfo,
) {
    let full_instance_path = dirs.instances_dir().join(instance_path);

    let Ok(metadata) = tokio::fs::metadata(&full_instance_path).await else {
        return;
    };

    if !metadata.is_dir() {
        return;
    }

    let mut to_watch = Vec::new();
    for sub_path in ProjectType::iterator()
        .map(|x| x.get_folder())
        .chain(["crash-reports", "saves", CONFIG_DIRECTORY])
    {
        let full_path = full_instance_path.join(sub_path);

        let meta = tokio::fs::symlink_metadata(&full_path).await;
        let exists = meta.is_ok();
        let is_symlink = meta.ok().is_some_and(|m| m.file_type().is_symlink());

        if !exists
            && !is_symlink
            && !sub_path.contains(".")
            && let Err(e) = crate::util::io::create_dir_all(&full_path).await
        {
            tracing::error!(
                "Failed to create directory for watcher {full_path:?}: {e}"
            );
            return;
        }

        to_watch.push(full_path);
    }

    remember_existing_config_files(
        &full_instance_path.join(CONFIG_DIRECTORY),
        &watcher.known_config_files,
    )
    .await;

    let mut debouncer = watcher.watcher.write().await;
    for full_path in &to_watch {
        if let Err(e) = debouncer
            .watcher()
            .watch(full_path, RecursiveMode::Recursive)
        {
            tracing::error!(
                "Failed to watch directory for watcher {full_path:?}: {e}"
            );
            return;
        }
    }

    if let Err(e) = debouncer
        .watcher()
        .watch(&full_instance_path, RecursiveMode::NonRecursive)
    {
        tracing::error!(
            "Failed to watch root instance directory for watcher {full_instance_path:?}: {e}"
        );
    }

    watcher
        .instance_ids
        .write()
        .await
        .insert(instance_path.to_string(), instance_id.to_string());
}

async fn remember_existing_config_files(
    config_path: &Path,
    known_config_files: &RwLock<HashSet<PathBuf>>,
) {
    let filter_root = config_path.to_path_buf();
    let mut walker = WalkDir::new(config_path).filter(move |entry| {
        let filter_root = filter_root.clone();
        async move {
            let excluded = entry
                .path()
                .strip_prefix(&filter_root)
                .is_ok_and(is_excluded_config_path);
            if excluded {
                Filtering::IgnoreDir
            } else {
                Filtering::Continue
            }
        }
    });
    let mut paths = Vec::new();

    while let Some(entry) = walker.next().await {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type().await else {
            continue;
        };
        if file_type.is_file() && is_supported_config_file(&entry.path()) {
            paths.push(entry.path());
        }
    }

    known_config_files.write().await.extend(paths);
}

fn is_supported_config_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            CONFIG_FILE_EXTENSIONS
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

fn is_excluded_config_file(path: &std::path::Path) -> bool {
    let mut components = path.components();
    while components
        .next()
        .is_some_and(|component| component.as_os_str() != CONFIG_DIRECTORY)
    {}

    is_excluded_config_path(&components.collect::<std::path::PathBuf>())
}

fn crash_task(instance_id: String) {
    tokio::task::spawn(async move {
        let res = async {
            let state = State::get().await?;
            let Some(instance) =
                instance_rows::get_instance_by_id(&instance_id, &state.pool)
                    .await?
            else {
                return Ok(());
            };

            if instance.install_stage == InstanceInstallStage::Installed {
                emit_warning(&format!(
					"Instance {} has crashed! Visit the logs page to see a crash report.",
					instance.name
				))
				.await?;
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
