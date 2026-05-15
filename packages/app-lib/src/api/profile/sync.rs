use crate::event::{ProfilePayloadType, emit::emit_profile};
use crate::profile;
use crate::state::InstanceSyncOverrides;
use crate::State;

#[tracing::instrument]
pub async fn set_sync_enabled(path: &str, enabled: bool) -> crate::Result<()> {
    let state = State::get().await?;

    if let Some(mut profile) = profile::get(path).await? {
        profile.sync_enabled = enabled;
        profile.upsert(&state.pool).await?;

        if profile.sync_enabled {
            let sync_settings = crate::state::Settings::get(&state.pool).await?;
            let dot_minecraft = profile::get_full_path(path).await?;
            crate::sync::apply_sync_to_instance(
                &sync_settings.sync,
                &dot_minecraft,
                &state.directories.synced_dir(),
                profile.sync_enabled,
                &profile.sync_overrides,
            )?;
        } else {
            let sync_settings = crate::state::Settings::get(&state.pool).await?;
            let dot_minecraft = profile::get_full_path(path).await?;
            crate::sync::apply_sync_to_instance(
                &sync_settings.sync,
                &dot_minecraft,
                &state.directories.synced_dir(),
                false,
                &None,
            )?;
        }

        emit_profile(path, ProfilePayloadType::Edited).await?;
        Ok(())
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(path.to_string()).into())
    }
}

#[tracing::instrument]
pub async fn set_sync_overrides(
    path: &str,
    overrides: Option<InstanceSyncOverrides>,
) -> crate::Result<()> {
    let state = State::get().await?;

    if let Some(mut profile) = profile::get(path).await? {
        profile.sync_overrides = overrides.clone();
        profile.upsert(&state.pool).await?;

        if profile.sync_enabled {
            let sync_settings = crate::state::Settings::get(&state.pool).await?;
            let dot_minecraft = profile::get_full_path(path).await?;
            crate::sync::apply_sync_to_instance(
                &sync_settings.sync,
                &dot_minecraft,
                &state.directories.synced_dir(),
                profile.sync_enabled,
                &overrides,
            )?;
        }

        emit_profile(path, ProfilePayloadType::Edited).await?;
        Ok(())
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(path.to_string()).into())
    }
}
