use super::get::get;
use super::paths::get_full_path;
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::{
    CreateInstance, EditInstance, InstanceLink, InstanceMetadata, ModLoader,
    State,
};
use crate::util::io;
use std::path::Path;

#[tracing::instrument]
#[allow(clippy::too_many_arguments)]
pub async fn create(
    name: String,
    game_version: String,
    modloader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    link: InstanceLink,
    skip_install: Option<bool>,
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

        if !skip_install.unwrap_or(false) {
            let context =
                crate::state::instances::commands::get_instance_launch_context(
                    &instance.id,
                    &state.pool,
                )
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::OtherError(format!(
                        "Missing launch context for instance {}",
                        instance.id
                    ))
                })?;
            crate::launcher::install_minecraft(&context, None, false).await?;
        }

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

pub async fn duplicate(copy_from: &str) -> crate::Result<String> {
    let metadata = get(copy_from).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let created = create(
        metadata.instance.name.clone(),
        metadata.applied_content_set.game_version.clone(),
        metadata.applied_content_set.loader,
        metadata.applied_content_set.loader_version.clone(),
        metadata.instance.icon_path.clone(),
        metadata.link.clone(),
        Some(true),
    )
    .await?;

    let state = State::get().await?;
    let bar = crate::pack::import::copy_dotminecraft(
        &created.instance.id,
        get_full_path(copy_from).await?,
        &state.io_semaphore,
        None,
    )
    .await?;

    let context =
        crate::state::instances::commands::get_instance_launch_context(
            &created.instance.id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    crate::launcher::install_minecraft(&context, Some(bar), false).await?;
    emit_instance(&created.instance.id, InstancePayloadType::Edited).await?;

    Ok(created.instance.id)
}

pub async fn edit(
    instance_id: &str,
    patch: EditInstance,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    crate::state::edit_instance(instance_id, patch, &state.pool).await?;

    crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string()).into()
        })
}

pub async fn edit_icon(
    instance_id: &str,
    icon_path: Option<&Path>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
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
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn remove(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?;
    crate::state::remove_instance(instance_id, &state).await?;

    if let Some(metadata) = metadata {
        emit_instance(&metadata.instance.id, InstancePayloadType::Removed)
            .await?;
    }

    Ok(())
}
