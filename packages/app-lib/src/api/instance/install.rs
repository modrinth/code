use crate::launcher::profile::ResolvedVersion;
use crate::state::{InstanceLaunchContext, JavaVersion, State};
use anyhow::Context;

async fn resolve_launch_version_info(
    instance_id: &str,
    state: &State,
) -> crate::Result<(InstanceLaunchContext, ResolvedVersion)> {
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .context(format!(
            "tried to resolve a nonexistent instance {instance_id}"
        ))?;
    let (minecraft, version_index) =
        crate::launcher::resolve_minecraft_manifest(
            &context.applied_content_set.game_version,
            state,
        )
        .await?;
    let version = &minecraft.versions[version_index];
    let loader_version = crate::launcher::get_loader_version_from_profile(
        &context.applied_content_set.game_version,
        context.applied_content_set.loader,
        context.applied_content_set.loader_version.as_deref(),
        &state.directories,
    )
    .await?;
    let version_info = crate::launcher::download::download_version_info(
        state,
        version,
        loader_version.as_ref(),
        None,
        None,
        None,
    )
    .await?;

    Ok((context, version_info))
}

pub async fn get_optimal_jre_key(
    instance_id: &str,
) -> crate::Result<Option<JavaVersion>> {
    let state = State::get().await?;
    let (context, version_info) =
        resolve_launch_version_info(instance_id, &state).await?;

    crate::launcher::get_java_version_from_launch_context(
        &context,
        &version_info,
        &state.pool,
    )
    .await
}

pub async fn get_recommended_java_version(
    instance_id: &str,
) -> crate::Result<u32> {
    let state = State::get().await?;
    let (_, version_info) =
        resolve_launch_version_info(instance_id, &state).await?;

    Ok(crate::launcher::get_recommended_java_major_version(
        &version_info,
    ))
}
