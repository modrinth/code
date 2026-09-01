//! Writes synced settings and launcher overrides before Minecraft starts.

use super::options_file::{
    GameOptionsDocument, input_error, options_path, read_document, sha1_bytes,
    validate_raw_key_value,
};
use super::write_shared_settings::{
    apply_shared_settings_to_instance, remember_processed_sha,
    sync_is_active_for_instance,
};
use crate::state::{InstanceMetadata, State};
use crate::util::io;
use std::collections::HashSet;

pub(super) fn currently_launcher_owned_keys(
    metadata: &InstanceMetadata,
) -> HashSet<String> {
    let mut keys = HashSet::new();
    if metadata.launch_overrides.force_fullscreen.is_some() {
        keys.insert("fullscreen".to_string());
    }
    keys
}

/// Writes canonical settings to `options.txt` before launch.
pub async fn sync_before_launch(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| input_error("Unknown instance"))?;
    if sync_is_active_for_instance(&metadata, &state).await? {
        let _ =
            apply_shared_settings_to_instance(&metadata, &state, false).await?;
    }
    Ok(())
}

/// Applies launcher settings, such as forced fullscreen, after shared settings.
///
/// Stores the written file's hash so the watcher ignores the app's write.
pub async fn apply_launcher_overrides(
    instance_id: &str,
    overrides: &[(String, String)],
) -> crate::Result<()> {
    if overrides.is_empty() {
        return Ok(());
    }
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| input_error("Unknown instance"))?;
    let path = options_path(&metadata, &state);
    let (mut document, input_bytes) = if path.exists() {
        read_document(&path).await?
    } else {
        (GameOptionsDocument::empty(), Vec::new())
    };
    for (key, value) in overrides {
        validate_raw_key_value(key, value)?;
        document.set(key, value, true)?;
    }
    let output_bytes = document.serialize()?;
    let output_sha1 = sha1_bytes(&output_bytes);
    if output_bytes != input_bytes {
        io::write(&path, output_bytes).await?;
    }
    remember_processed_sha(instance_id, &output_sha1, &state).await?;
    Ok(())
}
