use crate::state::{InstanceInstallStage, JavaVersion, State};

pub async fn get_optimal_jre_key(
    instance_id: &str,
) -> crate::Result<Option<JavaVersion>> {
    let state = State::get().await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to resolve a nonexistent instance {instance_id}!"
            ))
        })?;
    let (minecraft, version_index) =
        crate::launcher::resolve_minecraft_manifest(
            &context.applied_content_set.game_version,
            &state,
        )
        .await?;
    let version = &minecraft.versions[version_index];
    let loader_version = crate::launcher::get_loader_version_from_profile(
        &context.applied_content_set.game_version,
        context.applied_content_set.loader,
        context.applied_content_set.loader_version.as_deref(),
    )
    .await?;
    let version_info = crate::launcher::download::download_version_info(
        &state,
        version,
        loader_version.as_ref(),
        None,
        None,
    )
    .await?;

    crate::launcher::get_java_version_from_launch_context(
        &context,
        &version_info,
    )
    .await
}

#[tracing::instrument]
pub async fn install(instance_id: &str, force: bool) -> crate::Result<()> {
    let state = State::get().await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to install a nonexistent instance {instance_id}!"
            ))
        })?;
    let result =
        crate::launcher::install_minecraft(&context, None, force).await;
    if result.is_err() {
        let current_stage =
            crate::state::instances::commands::get_instance_launch_context(
                instance_id,
                &state.pool,
            )
            .await
            .ok()
            .flatten()
            .map(|context| context.instance.install_stage)
            .unwrap_or(InstanceInstallStage::NotInstalled);
        if current_stage != InstanceInstallStage::Installed {
            crate::state::instances::commands::set_instance_install_stage(
                &context.instance.id,
                InstanceInstallStage::NotInstalled,
                &state.pool,
            )
            .await?;
        }
    }

    result
}
