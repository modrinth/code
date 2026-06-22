use crate::state::{
    ContentSet, Hooks, Instance, InstanceLink, MemorySettings, WindowSize,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceLaunchOverrides {
    pub instance_id: String,
    pub java_path: Option<String>,
    pub extra_launch_args: Option<Vec<String>>,
    pub custom_env_vars: Option<Vec<(String, String)>>,
    pub memory: Option<MemorySettings>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSize>,
    pub hooks: Hooks,
}

impl InstanceLaunchOverrides {
    pub fn empty(instance_id: String) -> Self {
        Self {
            instance_id,
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
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct InstanceLaunchOverridesData {
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
}

impl InstanceLaunchOverridesData {
    pub(crate) fn into_launch_overrides(
        self,
        instance_id: String,
    ) -> InstanceLaunchOverrides {
        InstanceLaunchOverrides {
            instance_id,
            java_path: self.java_path,
            extra_launch_args: self.extra_launch_args,
            custom_env_vars: self.custom_env_vars,
            memory: self.memory,
            force_fullscreen: self.force_fullscreen,
            game_resolution: self.game_resolution,
            hooks: self.hooks,
        }
    }
}

impl From<&InstanceLaunchOverrides> for InstanceLaunchOverridesData {
    fn from(overrides: &InstanceLaunchOverrides) -> Self {
        Self {
            java_path: overrides.java_path.clone(),
            extra_launch_args: overrides.extra_launch_args.clone(),
            custom_env_vars: overrides.custom_env_vars.clone(),
            memory: overrides.memory,
            force_fullscreen: overrides.force_fullscreen,
            game_resolution: overrides.game_resolution,
            hooks: overrides.hooks.clone(),
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
