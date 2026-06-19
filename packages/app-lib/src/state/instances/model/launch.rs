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
pub struct InstanceLaunchContext {
    pub instance: Instance,
    pub applied_content_set: ContentSet,
    pub link: InstanceLink,
    pub launch_overrides: InstanceLaunchOverrides,
}
