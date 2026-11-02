use crate::state::{
    ContentSet, Hooks, Instance, InstanceLink, MemorySettings, WindowSize,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct InstanceTabVisibility {
    pub files: bool,
    pub worlds: bool,
    pub screenshots: bool,
}

impl Default for InstanceTabVisibility {
    fn default() -> Self {
        Self {
            files: true,
            worlds: true,
            screenshots: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceLaunchOverrides {
    pub instance_id: String,
    #[serde(default)]
    pub allow_multiple_instances: bool,
    pub java_path: Option<String>,
    pub extra_launch_args: Option<Vec<String>>,
    pub custom_env_vars: Option<Vec<(String, String)>>,
    pub memory: Option<MemorySettings>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSize>,
    pub hooks: Hooks,
    #[serde(default)]
    pub visible_tabs: InstanceTabVisibility,
}

impl InstanceLaunchOverrides {
    pub fn empty(instance_id: String) -> Self {
        Self {
            instance_id,
            allow_multiple_instances: false,
            java_path: None,
            extra_launch_args: None,
            custom_env_vars: None,
            memory: None,
            force_fullscreen: None,
            game_resolution: None,
            hooks: Hooks {
                pre_launch: None,
                wrapper: None,
                post_exit: None,
            },
            visible_tabs: InstanceTabVisibility::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct InstanceLaunchOverridesData {
    #[serde(default)]
    pub allow_multiple_instances: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_launch_args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_env_vars: Option<Vec<(String, String)>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemorySettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_fullscreen: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_resolution: Option<WindowSize>,
    #[serde(default)]
    pub hooks: Hooks,
    #[serde(default)]
    pub visible_tabs: InstanceTabVisibility,
}

impl InstanceLaunchOverridesData {
    pub(crate) fn into_launch_overrides(
        self,
        instance_id: String,
    ) -> InstanceLaunchOverrides {
        InstanceLaunchOverrides {
            instance_id,
            allow_multiple_instances: self.allow_multiple_instances,
            java_path: self.java_path,
            extra_launch_args: self.extra_launch_args,
            custom_env_vars: self.custom_env_vars,
            memory: self.memory,
            force_fullscreen: self.force_fullscreen,
            game_resolution: self.game_resolution,
            hooks: self.hooks,
            visible_tabs: self.visible_tabs,
        }
    }
}

impl From<&InstanceLaunchOverrides> for InstanceLaunchOverridesData {
    fn from(overrides: &InstanceLaunchOverrides) -> Self {
        Self {
            allow_multiple_instances: overrides.allow_multiple_instances,
            java_path: overrides.java_path.clone(),
            extra_launch_args: overrides.extra_launch_args.clone(),
            custom_env_vars: overrides.custom_env_vars.clone(),
            memory: overrides.memory,
            force_fullscreen: overrides.force_fullscreen,
            game_resolution: overrides.game_resolution,
            hooks: overrides.hooks.clone(),
            visible_tabs: overrides.visible_tabs,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceLaunchContext {
    pub instance: Instance,
    pub applied_content_set: ContentSet,
    pub link: InstanceLink,
    pub launch_overrides: InstanceLaunchOverrides,
}

#[cfg(test)]
mod tests {
    use super::{InstanceLaunchOverrides, InstanceLaunchOverridesData};

    #[test]
    fn old_launch_settings_default_to_single_instance() {
        let data: InstanceLaunchOverridesData =
            serde_json::from_str("{}").unwrap();
        assert!(
            !data
                .into_launch_overrides("local:test".into())
                .allow_multiple_instances
        );

        let mut old_settings = serde_json::to_value(
            InstanceLaunchOverrides::empty("local:test".into()),
        )
        .unwrap();
        old_settings
            .as_object_mut()
            .unwrap()
            .remove("allow_multiple_instances");
        let restored: InstanceLaunchOverrides =
            serde_json::from_value(old_settings).unwrap();
        assert!(!restored.allow_multiple_instances);
    }

    #[test]
    fn multiple_instance_setting_survives_storage_round_trip() {
        for enabled in [true, false] {
            let mut overrides =
                InstanceLaunchOverrides::empty("local:test".into());
            overrides.allow_multiple_instances = enabled;
            overrides.extra_launch_args = Some(vec!["-Dtest=true".into()]);
            let stored = serde_json::to_string(
                &InstanceLaunchOverridesData::from(&overrides),
            )
            .unwrap();
            let restored =
                serde_json::from_str::<InstanceLaunchOverridesData>(&stored)
                    .unwrap()
                    .into_launch_overrides("local:test".into());
            assert_eq!(restored.allow_multiple_instances, enabled);
            assert_eq!(restored.extra_launch_args, overrides.extra_launch_args);
        }
    }
}
