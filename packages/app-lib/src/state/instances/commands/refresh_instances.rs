use crate::State;
use crate::state::LauncherFeatureVersion;

use super::edit_instance::EditInstance;

pub(crate) async fn refresh_all_instances() -> crate::Result<()> {
    let state = State::get().await?;
    let instances = crate::state::instances::adapters::sqlite::instance_rows::list_instances(
		&state.pool,
	)
	.await?;

    for instance in instances {
        let launcher_feature_version = (instance.launcher_feature_version
            < LauncherFeatureVersion::MOST_RECENT)
            .then_some(LauncherFeatureVersion::MOST_RECENT);

        if launcher_feature_version.is_none() {
            continue;
        }

        super::edit_instance::edit_instance(
            &instance.id,
            EditInstance {
                install_stage: None,
                launcher_feature_version,
                ..EditInstance::default()
            },
            &state.pool,
        )
        .await?;
    }

    Ok(())
}
