use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{
    CreateInstance, EditInstance, InstanceIconConfig, InstanceLink,
    InstanceMetadata, InstanceSyncedOption, ModLoader, State,
};

#[tracing::instrument]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn create(
    name: String,
    game_version: String,
    modloader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    icon_config: Option<InstanceIconConfig>,
    link: InstanceLink,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    if let Some(icon_config) = &icon_config {
        super::icon::validate_generated_icon_config(icon_config)?;
    }
    let instance = crate::state::create_instance(
        CreateInstance {
            name,
            path: None,
            game_version,
            loader: modloader,
            loader_version,
            icon_path,
            icon_config,
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
    } else if let Err(error) =
        crate::onboarding_checklist::mark_created_instance().await
    {
        tracing::warn!(
            "Failed to mark instance creation in onboarding checklist: {error}"
        );
    }

    result
}

pub async fn edit(
    instance_id: &str,
    patch: EditInstance,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    if patch.content_set_patch.is_some()
        || patch.link.is_some()
        || patch.update_channel.is_some()
        || patch.install_stage.is_some()
    {
        let instance =
            instance_rows::get_instance_by_id(instance_id, &state.pool)
                .await?
                .ok_or_else(|| {
                    crate::state::content_store::input("Unknown instance")
                })?;
        super::projects::ensure_installation_content_unlocked(
            instance.install_stage,
        )?;
    }
    crate::state::edit_instance(instance_id, patch, &state.pool).await?;

    let instance = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
                .as_error()
        })?;

    super::reconcile_instance_synced_options(instance_id).await?;

    emit_instance(&instance.instance.id, InstancePayloadType::Edited).await?;

    Ok(instance)
}

pub async fn set_synced_option(
    instance_id: &str,
    option: InstanceSyncedOption,
    enabled: bool,
    resolution: Option<super::SyncedOptionJoinResolution>,
) -> crate::Result<InstanceMetadata> {
    let instance = super::synced_options::set_instance_option(
        instance_id,
        option,
        enabled,
        resolution,
    )
    .await?;

    emit_instance(&instance.instance.id, InstancePayloadType::Edited).await?;

    Ok(instance)
}

#[tracing::instrument]
pub async fn remove(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let instance =
        instance_rows::get_instance_display_info(instance_id, &state.pool)
            .await?;
    let _install_guards =
        crate::install::runner::cancel_jobs_for_instance_deletion(
            instance_id,
            &state,
        )
        .await?;
    if instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .is_some()
    {
        crate::state::remove_instance(instance_id, &state).await?;
    }

    if let Some(instance) = instance {
        emit_instance(&instance.id, InstancePayloadType::Removed).await?;
    }

    Ok(())
}
