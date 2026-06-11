use crate::state::{Hooks, MemorySettings, WindowSize};
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
