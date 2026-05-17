use chrono::TimeZone;
use std::path::PathBuf;

use sqlx::SqlitePool;

use crate::{
    api::pack::import::copy_dotminecraft,
    state::{ModLoader, Profile, ProfileInstallStage, LauncherFeatureVersion},
};

pub async fn get_modrinth_instances(base_path: PathBuf) -> crate::Result<Vec<String>> {
    let db_path = base_path.join("app.db");
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let connection_string = format!("sqlite://{}?mode=ro", db_path.display());

    let pool = SqlitePool::connect(&connection_string).await.map_err(|e| {
        crate::ErrorKind::InputError(format!("Could not connect to Modrinth app.db: {e}"))
    })?;

    // ModrinthApp DB has profiles
    let ids: Vec<String> = sqlx::query_scalar("SELECT path FROM profiles")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    Ok(ids)
}

pub async fn import_modrinth(
    base_path: PathBuf,
    instance_folder: String,
    profile_path: &str,
) -> crate::Result<()> {
    let db_path = base_path.join("app.db");
    let connection_string = format!("sqlite://{}?mode=ro", db_path.display());
    let pool = SqlitePool::connect(&connection_string).await.map_err(|e| {
        crate::ErrorKind::InputError(format!("Could not connect to Modrinth app.db: {e}"))
    })?;

    // Query Modrinth DB. We use a raw query because Modrinth's DB schema 
    // lacks Icarus-specific columns like sync_enabled or launcher_feature_version.
    // We use sqlx::query_as instead of the macro so it doesn't try to validate against the Icarus DB at compile time.
    use sqlx::Row;
    let record = sqlx::query(
        r#"
        SELECT
            path, install_stage, name, icon_path,
            game_version, protocol_version, mod_loader, mod_loader_version,
            groups,
            linked_project_id, linked_version_id, locked,
            created, modified, last_played,
            submitted_time_played, recent_time_played,
            override_java_path,
            override_extra_launch_args, override_custom_env_vars,
            override_mc_memory_max, override_mc_force_fullscreen, override_mc_game_resolution_x, override_mc_game_resolution_y,
            override_hook_pre_launch, override_hook_wrapper, override_hook_post_exit
        FROM profiles
        WHERE path = $1
        "#
    )
    .bind(instance_folder.clone())
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError("Instance not found in Modrinth DB".to_string())
    })?;

    let override_extra_launch_args: Option<Vec<u8>> = record.try_get("override_extra_launch_args").unwrap_or_default();
    let override_custom_env_vars: Option<Vec<u8>> = record.try_get("override_custom_env_vars").unwrap_or_default();
    let groups_bin: Option<Vec<u8>> = record.try_get("groups").unwrap_or_default();

    let extra_args_val: Option<serde_json::Value> = override_extra_launch_args.and_then(|v| serde_json::from_slice(&v).ok());
    let custom_env_val: Option<serde_json::Value> = override_custom_env_vars.and_then(|v| serde_json::from_slice(&v).ok());
    let groups_val: Option<serde_json::Value> = groups_bin.and_then(|v| serde_json::from_slice(&v).ok());

    let linked_project_id: Option<String> = record.get("linked_project_id");
    let linked_version_id: Option<String> = record.get("linked_version_id");
    let locked: Option<i64> = record.get("locked");
    
    let created: i64 = record.get("created");
    let modified: i64 = record.get("modified");
    let last_played: Option<i64> = record.get("last_played");
    let submitted_time_played: i64 = record.get("submitted_time_played");
    let recent_time_played: i64 = record.get("recent_time_played");
    
    let override_mc_memory_max: Option<i64> = record.get("override_mc_memory_max");
    let override_mc_force_fullscreen: Option<i64> = record.get("override_mc_force_fullscreen");
    let override_mc_game_resolution_x: Option<i64> = record.get("override_mc_game_resolution_x");
    let override_mc_game_resolution_y: Option<i64> = record.get("override_mc_game_resolution_y");
    let protocol_version: Option<i64> = record.get("protocol_version");

    let install_stage: String = record.get("install_stage");
    let name: String = record.get("name");
    let icon_path: Option<String> = record.get("icon_path");
    let game_version: String = record.get("game_version");
    let mod_loader: String = record.get("mod_loader");
    let mod_loader_version: Option<String> = record.get("mod_loader_version");
    let override_java_path: Option<String> = record.get("override_java_path");
    let override_hook_pre_launch: Option<String> = record.get("override_hook_pre_launch");
    let override_hook_wrapper: Option<String> = record.get("override_hook_wrapper");
    let override_hook_post_exit: Option<String> = record.get("override_hook_post_exit");

    let profile = Profile {
        path: profile_path.to_string(),
        install_stage: ProfileInstallStage::from_str(&install_stage),
        launcher_feature_version: LauncherFeatureVersion::MOST_RECENT,
        name,
        icon_path,
        game_version,
        protocol_version: protocol_version.map(|v| v as u32),
        loader: ModLoader::from_string(&mod_loader),
        loader_version: mod_loader_version,
        groups: groups_val.unwrap_or_default().as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
        linked_data: if let (Some(project_id), Some(version_id)) = (linked_project_id, linked_version_id) {
            locked.map(|locked| crate::state::LinkedData {
                project_id,
                version_id,
                locked: locked == 1,
            })
        } else {
            None
        },
        created: chrono::Utc.timestamp_opt(created, 0).single().unwrap_or_else(chrono::Utc::now),
        modified: chrono::Utc.timestamp_opt(modified, 0).single().unwrap_or_else(chrono::Utc::now),
        last_played: last_played.and_then(|x| chrono::Utc.timestamp_opt(x, 0).single()),
        submitted_time_played: submitted_time_played as u64,
        recent_time_played: recent_time_played as u64,
        java_path: override_java_path,
        extra_launch_args: extra_args_val.and_then(|v| serde_json::from_value(v).ok()),
        custom_env_vars: custom_env_val.and_then(|v| serde_json::from_value(v).ok()),
        memory: override_mc_memory_max.map(|x| crate::state::MemorySettings { maximum: x as u32 }),
        force_fullscreen: override_mc_force_fullscreen.map(|x| x == 1),
        game_resolution: if let (Some(x_res), Some(y_res)) = (override_mc_game_resolution_x, override_mc_game_resolution_y) {
            Some(crate::state::WindowSize(x_res as u16, y_res as u16))
        } else {
            None
        },
        hooks: crate::state::Hooks {
            pre_launch: override_hook_pre_launch,
            wrapper: override_hook_wrapper,
            post_exit: override_hook_post_exit,
        },
        sync_enabled: false,
        sync_overrides: None,
    };

    let state = crate::State::get().await?;
    profile.upsert(&state.pool).await?;

    let src_folder = base_path.join("profiles").join(&instance_folder);
    let mut loading_bar = None;
    if src_folder.exists() {
        loading_bar = Some(copy_dotminecraft(profile_path, src_folder, &state.io_semaphore, None).await?);
    }

    if let Some(profile_val) = crate::api::profile::get(profile_path).await? {
        crate::launcher::install_minecraft(
            &profile_val,
            loading_bar,
            false,
        )
        .await?;
    }

    Ok(())
}
