use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{
    CreateInstance, EditInstance, InstanceLink, InstanceMetadata, ModLoader,
    State,
};
use crate::util::io;
use std::path::Path;

#[tracing::instrument]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn create(
    name: String,
    game_version: String,
    modloader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    link: InstanceLink,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    let instance = crate::state::create_instance(
        CreateInstance {
            name,
            path: None,
            game_version,
            loader: modloader,
            loader_version,
            icon_path,
            link,
        },
        &state,
    )
    .await?;

    let result = async {
        emit_instance(&instance.id, InstancePayloadType::Created).await?;

        crate::state::get_instance(&instance.id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(
                    "Created instance could not be loaded".to_string(),
                )
                .into()
            })
    }
    .await;

    if result.is_err() {
        let _ = crate::state::remove_instance(&instance.id, &state).await;
    }

    result
}

pub async fn edit(
    instance_id: &str,
    patch: EditInstance,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    let should_reconcile_shared_publish = patch.link.is_some();
    crate::state::edit_instance(instance_id, patch, &state.pool).await?;

    if should_reconcile_shared_publish {
        super::shared::mark_shared_instance_stale(instance_id, &state).await?;
    }

    let instance = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
                .as_error()
        })?;

    emit_instance(&instance.instance.id, InstancePayloadType::Edited).await?;

    Ok(instance)
}

pub async fn edit_icon(
    instance_id: &str,
    icon_path: Option<&Path>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let instance =
        instance_rows::get_instance_display_info(instance_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;
    let icon_path = if let Some(icon) = icon_path {
        let bytes = io::read(icon).await?;
        let file = crate::util::fetch::write_cached_icon(
            &icon.to_string_lossy(),
            &state.directories.caches_dir(),
            bytes::Bytes::from(bytes),
            &state.io_semaphore,
        )
        .await?;
        Some(file.to_string_lossy().to_string())
    } else {
        None
    };

    crate::state::edit_instance(
        instance_id,
        EditInstance {
            icon_path: Some(icon_path),
            ..EditInstance::default()
        },
        &state.pool,
    )
    .await?;
    emit_instance(&instance.id, InstancePayloadType::Edited).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn remove(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let instance =
        instance_rows::get_instance_display_info(instance_id, &state.pool)
            .await?;
    crate::state::remove_instance(instance_id, &state).await?;

    if let Some(instance) = instance {
        emit_instance(&instance.id, InstancePayloadType::Removed).await?;
    }

    Ok(())
}
