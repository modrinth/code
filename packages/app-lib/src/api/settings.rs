//! Theseus settings management interface

pub use crate::state::content_store::{StoreUsage, StoreVerification};
pub use crate::{
    State,
    state::{Hooks, MemorySettings, Settings, WindowSize},
};

pub async fn store_usage() -> crate::Result<StoreUsage> {
    let state = State::get().await?;
    state.content_store.usage(&state).await
}

pub async fn store_cleanup() -> crate::Result<u64> {
    let state = State::get().await?;
    state.content_store.cleanup(&state, true).await
}

pub async fn store_set_cache_limit(bytes: u64) -> crate::Result<()> {
    State::get()
        .await?
        .content_store
        .set_cache_limit(bytes)
        .await
}

pub async fn store_verify(repair: bool) -> crate::Result<StoreVerification> {
    let state = State::get().await?;
    state.content_store.verify(&state, repair).await
}

pub async fn store_verify_with_progress(
    repair: bool,
    on_progress: &(dyn Fn(u64, u64) + Send + Sync),
) -> crate::Result<StoreVerification> {
    let state = State::get().await?;
    if repair {
        let processes = state.process_manager.get_all();
        for process in &processes {
            state.process_manager.kill(process.uuid).await?;
        }
        let deadline =
            tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        while state.process_manager.get_all().iter().any(|running| {
            processes.iter().any(|process| process.uuid == running.uuid)
        }) {
            if tokio::time::Instant::now() >= deadline {
                return Err(crate::ErrorKind::InputError(
                    "Timed out closing running instances".into(),
                )
                .as_error());
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }
    state
        .content_store
        .verify_with_progress(&state, repair, on_progress)
        .await
}

/// Gets entire settings
#[tracing::instrument]
pub async fn get() -> crate::Result<Settings> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let mut settings = Settings::get(&state.pool).await?;
    if crate::state::game_options_sync_is_enabled(&state.pool).await?
        && let Some(fullscreen) =
            crate::api::instance::shared_fullscreen_value(&state).await?
        && settings.force_fullscreen != fullscreen
    {
        settings.force_fullscreen = fullscreen;
        settings.update(&state.pool).await?;
    }
    Ok(settings)
}

/// Sets entire settings
#[tracing::instrument]
pub async fn set(settings: Settings) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let current_settings = Settings::get(&state.pool).await?;
    let game_options_sync_enabled =
        crate::state::game_options_sync_is_enabled(&state.pool).await?;
    let current_fullscreen = if game_options_sync_enabled {
        crate::api::instance::shared_fullscreen_value(&state)
            .await?
            .unwrap_or(current_settings.force_fullscreen)
    } else {
        current_settings.force_fullscreen
    };
    let fullscreen_changed = current_fullscreen != settings.force_fullscreen;
    let shared_fullscreen_changed =
        if game_options_sync_enabled && fullscreen_changed {
            crate::api::instance::update_shared_fullscreen_from_app(
                &state,
                settings.force_fullscreen,
            )
            .await?
        } else {
            false
        };
    settings.update(&state.pool).await?;

    if shared_fullscreen_changed {
        let result =
            crate::api::instance::sync_all_participating_instances(&state)
                .await;
        if result.failed > 0 {
            tracing::warn!(
                "Fullscreen was saved, but {} synced instance(s) could not be updated",
                result.failed
            );
        }
    }

    Ok(())
}

#[tracing::instrument]
pub async fn cancel_directory_change(
    app_identifier: &str,
) -> crate::Result<()> {
    // This is called to handle state initialization errors due to folder migrations
    // failing, so fetching a DB connection pool from `State::get` is not reliable here
    let pool = crate::state::db::connect(app_identifier).await?;
    let mut settings = Settings::get(&pool).await?;
    crate::state::content_store::migration::cancel_move(&pool).await?;

    if let Some(prev_custom_dir) = settings.prev_custom_dir {
        settings.prev_custom_dir = None;
        settings.custom_dir = Some(prev_custom_dir);
    }

    settings.update(&pool).await?;

    Ok(())
}
