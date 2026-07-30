use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{
    CreateInstance, EditInstance, InstanceLink, InstanceMetadata, ModLoader,
    State,
};

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
    crate::state::edit_instance(instance_id, patch, &state.pool).await?;

    let instance = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
                .as_error()
        })?;

    emit_instance(&instance.instance.id, InstancePayloadType::Edited).await?;

    Ok(instance)
}

#[tracing::instrument]
pub async fn remove(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let instance =
        instance_rows::get_instance_display_info(instance_id, &state.pool)
            .await?;
    crate::install::runner::cancel_jobs_for_instance_deletion(
        instance_id,
        &state,
    )
    .await?;
    crate::state::remove_instance(instance_id, &state).await?;

    if let Some(instance) = instance {
        emit_instance(&instance.id, InstancePayloadType::Removed).await?;
    }

    Ok(())
}
