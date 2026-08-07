use partially::Partial;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
pub struct UserPreferences {
    pub appearance: AppearancePreferences,
    pub localization: LocalizationPreferences,
    pub layouts: LayoutPreferences,
    pub sidebars: SidebarPreferences,
    pub social: SocialPreferences,
}

#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct PartialUserPreferences {
    pub appearance: Option<PartialAppearancePreferences>,
    pub localization: Option<PartialLocalizationPreferences>,
    pub layouts: Option<PartialLayoutPreferences>,
    pub sidebars: Option<PartialSidebarPreferences>,
    pub social: Option<PartialSocialPreferences>,
}

impl Partial for UserPreferences {
    type Item = PartialUserPreferences;

    fn apply_some(&mut self, partial: Self::Item) -> bool {
        let mut applied = false;

        if let Some(appearance) = partial.appearance {
            applied |= self.appearance.apply_some(appearance);
        }
        if let Some(localization) = partial.localization {
            applied |= self.localization.apply_some(localization);
        }
        if let Some(layouts) = partial.layouts {
            applied |= self.layouts.apply_some(layouts);
        }
        if let Some(sidebars) = partial.sidebars {
            applied |= self.sidebars.apply_some(sidebars);
        }
        if let Some(social) = partial.social {
            applied |= self.social.apply_some(social);
        }

        applied
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Partial, Default)]
#[partially(skip_attributes, derive(Debug, Deserialize, ToSchema))]
pub struct AppearancePreferences {
    pub theme: Theme,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    #[default]
    Dark,
    Oled,
    Retro,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Partial)]
#[partially(skip_attributes, derive(Debug, Deserialize, ToSchema))]
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

#[derive(Debug, Serialize, Deserialize, ToSchema, Partial)]
#[partially(skip_attributes, derive(Debug, Deserialize, ToSchema))]
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LayoutOption {
    Grid,
    Rows,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Partial, Default)]
#[partially(skip_attributes, derive(Debug, Deserialize, ToSchema))]
pub struct SidebarPreferences {
    pub right_aligned_search: bool,
    pub left_aligned_content: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Partial, Default)]
#[partially(skip_attributes, derive(Debug, Deserialize, ToSchema))]
pub struct SocialPreferences {
    pub friend_privacy: FriendPrivacy,
    pub shared_instances_privacy: SharedInstancesPrivacy,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum FriendPrivacy {
    None,
    Mutual,
    #[default]
    Everyone,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SharedInstancesPrivacy {
    None,
    Friends,
    #[default]
    Everyone,
}
