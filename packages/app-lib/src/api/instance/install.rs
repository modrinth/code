use crate::{
    InvocationContext,
    state::{JavaVersion, State},
};

pub async fn get_optimal_jre_key(
    invocation_context: &InvocationContext,
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
            invocation_context,
            &context.applied_content_set.game_version,
            &state,
        )
        .await?;
    let version = &minecraft.versions[version_index];
    let loader_version = crate::launcher::get_loader_version_from_profile(
        invocation_context,
        &context.applied_content_set.game_version,
        context.applied_content_set.loader,
        context.applied_content_set.loader_version.as_deref(),
    )
    .await?;
    let version_info = crate::launcher::download::download_version_info(
        invocation_context,
        &state,
        version,
        loader_version.as_ref(),
        None,
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
