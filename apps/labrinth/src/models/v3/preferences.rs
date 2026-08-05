use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserPreferences {
    pub appearance: AppearancePreferences,
    pub localization: LocalizationPreferences,
    pub layouts: LayoutPreferences,
    pub sidebars: SidebarPreferences,
    pub social: SocialPreferences
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AppearancePreferences {
    pub theme: Theme
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum Theme {
    Light, Dark, Oled, Retro
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LocalizationPreferences {
    pub locale: String // FIXME: validate input
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LayoutPreferences {
    pub mods: LayoutOption,
    pub plugins: LayoutOption,
    pub datapacks: LayoutOption,
    pub shaders: LayoutOption,
    pub resourcepacks: LayoutOption,
    pub modpacks: LayoutOption,
    pub servers: LayoutOption,
    pub users: LayoutOption,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum LayoutOption {
    Grid, Rows
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SidebarPreferences {
    pub right_aligned_search: bool,
    pub left_aligned_content: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SocialPreferences {
    pub friend_privacy: FriendPrivacy,
    pub shared_instances_privacy: SharedInstancesPrivacy
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum FriendPrivacy {
    None, Mutual, Everyone
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum SharedInstancesPrivacy {
    None, Friends, Everyone
}
