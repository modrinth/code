use component_derive::Component;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Component)]
pub struct UserPreferences {
    #[component(nested)]
    pub appearance: AppearancePreferences,
    #[component(nested)]
    pub behavior: BehaviorPreferences,
    #[component(nested)]
    pub localization: LocalizationPreferences,
    #[component(nested)]
    pub layouts: LayoutPreferences,
    #[component(nested)]
    pub sidebars: SidebarPreferences,
    #[component(nested)]
    pub social: SocialPreferences,
}

impl UserPreferences {
    pub fn resolve(overrides: Option<PartialUserPreferences>) -> Self {
        let mut preferences = Self::default();
        if let Some(overrides) = overrides {
            overrides.apply_to(&mut preferences);
        }
        preferences
    }
}

#[derive(
    Debug, Serialize, Deserialize, ToSchema, Default, PartialEq, Component,
)]
pub struct AppearancePreferences {
    pub auto: bool,
    pub theme: Theme,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Component)]
pub struct BehaviorPreferences {
    pub minimize_app: bool,
    pub hide_right_sidebar: bool,
    pub show_jump_in: bool,
    pub compact_instance_cards: bool,
    pub show_play_time: bool,
    pub hide_nametag: bool,
    pub show_all_screenshots: bool,
    pub warn_on_unknown_modpacks: bool,
    pub skip_non_essential_warnings: bool,
}

impl Default for BehaviorPreferences {
    fn default() -> Self {
        Self {
            minimize_app: false,
            hide_right_sidebar: false,
            show_jump_in: true,
            compact_instance_cards: false,
            show_play_time: true,
            hide_nametag: false,
            show_all_screenshots: true,
            warn_on_unknown_modpacks: true,
            skip_non_essential_warnings: false,
        }
    }
}

#[derive(
    Debug, Serialize, Deserialize, ToSchema, Default, Clone, PartialEq,
)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    #[default]
    Dark,
    Oled,
    Retro,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Component)]
pub struct LocalizationPreferences {
    pub locale: String,
}

impl Default for LocalizationPreferences {
    fn default() -> Self {
        Self {
            locale: "en-US".to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Component)]
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

impl Default for LayoutPreferences {
    fn default() -> Self {
        Self {
            mods: LayoutOption::Rows,
            plugins: LayoutOption::Rows,
            datapacks: LayoutOption::Rows,
            shaders: LayoutOption::Grid,
            resourcepacks: LayoutOption::Grid,
            modpacks: LayoutOption::Rows,
            servers: LayoutOption::Rows,
            users: LayoutOption::Rows,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LayoutOption {
    Grid,
    Rows,
}

#[derive(
    Debug, Serialize, Deserialize, ToSchema, Default, PartialEq, Component,
)]
pub struct SidebarPreferences {
    pub right_aligned_search: bool,
    pub left_aligned_content: bool,
}

#[derive(
    Debug, Serialize, Deserialize, ToSchema, Default, PartialEq, Component,
)]
pub struct SocialPreferences {
    pub friend_privacy: FriendPrivacy,
    pub shared_instances_privacy: InvitePrivacy,
    pub hosting_access_privacy: InvitePrivacy,
}

#[derive(
    Debug, Serialize, Deserialize, ToSchema, Default, Clone, PartialEq,
)]
#[serde(rename_all = "snake_case")]
pub enum FriendPrivacy {
    None,
    Mutual,
    #[default]
    Everyone,
}

#[derive(
    Debug, Serialize, Deserialize, ToSchema, Default, Clone, PartialEq,
)]
#[serde(rename_all = "snake_case")]
pub enum InvitePrivacy {
    None,
    Friends,
    #[default]
    Everyone,
}
