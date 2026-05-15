use crate::state::{InstanceSyncOverrides, SyncSettings};
use crate::sync::linking;
use crate::sync::safe_path::SafePath;
use std::path::Path;

const FOLDER_TARGETS: &[&str] = &[
    "saves",
    "screenshots",
    "resourcepacks",
    "shaderpacks",
    "schematics",
];
const FILE_TARGETS: &[&str] = &["options.txt", "servers.dat"];

fn folder_enabled(name: &str, ov: &Option<InstanceSyncOverrides>) -> bool {
    match ov {
        None => true,
        Some(ov) => match name {
            "saves" => ov.saves.unwrap_or(true),
            "screenshots" => ov.screenshots.unwrap_or(true),
            "resourcepacks" => ov.resourcepacks.unwrap_or(true),
            "shaderpacks" => ov.shaderpacks.unwrap_or(true),
            "schematics" => ov.schematics.unwrap_or(true),
            _ => true,
        },
    }
}

fn file_enabled(name: &str, ov: &Option<InstanceSyncOverrides>) -> bool {
    match ov {
        None => true,
        Some(ov) => match name {
            "options.txt" => ov.options_txt.unwrap_or(true),
            "servers.dat" => ov.servers_dat.unwrap_or(true),
            _ => true,
        },
    }
}

pub fn apply_sync_to_instance(
    _sync_settings: &SyncSettings,
    instance_dot_minecraft: &Path,
    synced_dir: &Path,
    sync_enabled: bool,
    sync_overrides: &Option<InstanceSyncOverrides>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(instance_dot_minecraft)?;
    std::fs::create_dir_all(synced_dir)?;

    for folder in FOLDER_TARGETS {
        let Some(path) = SafePath::new(folder) else {
            continue;
        };
        let src = path.to_path(synced_dir);
        let dst = path.to_path(instance_dot_minecraft);
        let enabled = sync_enabled && folder_enabled(folder, sync_overrides);

        if enabled {
            if dst.exists() && !linking::is_targeting(&src, &dst) {
                let _ = std::fs::remove_dir_all(&dst);
            }
            std::fs::create_dir_all(&src)?;
            if !dst.exists() {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                linking::link_dir(&src, &dst)?;
            }
        } else if linking::is_targeting(&src, &dst) {
            let _ = std::fs::remove_dir(&dst);
            let _ = std::fs::create_dir_all(&dst);
        }
    }

    for file in FILE_TARGETS {
        let Some(path) = SafePath::new(file) else {
            continue;
        };
        let src = path.to_path(synced_dir);
        let dst = path.to_path(instance_dot_minecraft);
        let enabled = sync_enabled && file_enabled(file, sync_overrides);

        if enabled {
            if !src.exists() {
                if let Some(parent) = src.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                if *file != "servers.dat" {
                    let _ = std::fs::File::create(&src);
                }
            }
            if dst.exists() && !linking::is_targeting_file(&src, &dst) {
                let _ = std::fs::remove_file(&dst);
            }
            if !dst.exists() && src.exists() {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                linking::link_file(&src, &dst)?;
            }
        } else if linking::is_targeting_file(&src, &dst) {
            let _ = std::fs::remove_file(&dst);
        }
    }

    Ok(())
}
