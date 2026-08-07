use partially::Partial;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Partial, Default)]
#[partially(skip_attributes, derive(Deserialize, ToSchema))]
pub struct UserPreferences {
    pub appearance: AppearancePreferences,
    pub localization: LocalizationPreferences,
    pub layouts: LayoutPreferences,
    pub sidebars: SidebarPreferences,
    pub social: SocialPreferences,
}

#[derive(Serialize, Deserialize, ToSchema, Partial, Default)]
#[partially(skip_attributes, derive(Deserialize, ToSchema))]
pub struct AppearancePreferences {
    pub theme: Theme
}

#[derive(Serialize, Deserialize, ToSchema, Default)]
pub enum Theme {
    Light,
    #[default]
    Dark,
    Oled,
    Retro,
}

#[derive(Serialize, Deserialize, ToSchema, Partial)]
#[partially(skip_attributes, derive(Deserialize, ToSchema))]
pub struct LocalizationPreferences {
    pub locale: String
}

impl Default for LocalizationPreferences {
    fn default() -> Self {
        Self {
            locale: "en-US".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Partial)]
#[partially(skip_attributes, derive(Deserialize, ToSchema))]
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

#[derive(Serialize, Deserialize, ToSchema)]
pub enum LayoutOption {
    Grid, Rows
}

#[derive(Serialize, Deserialize, ToSchema, Partial, Default)]
#[partially(skip_attributes, derive(Deserialize, ToSchema))]
pub struct SidebarPreferences {
    pub right_aligned_search: bool,
    pub left_aligned_content: bool,
}

#[derive(Serialize, Deserialize, ToSchema, Partial, Default)]
#[partially(skip_attributes, derive(Deserialize, ToSchema))]
pub struct SocialPreferences {
    pub friend_privacy: FriendPrivacy,
    pub shared_instances_privacy: SharedInstancesPrivacy
}

#[derive(Serialize, Deserialize, ToSchema, Default)]
pub enum FriendPrivacy {
    None,
    Mutual,
    #[default]
    Everyone,
}

#[derive(Serialize, Deserialize, ToSchema, Default)]
pub enum SharedInstancesPrivacy {
    None,
    Friends,
    #[default]
    Everyone,
}
