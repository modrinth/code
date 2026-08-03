use eyre::{Result, eyre};
use language_tags::LanguageTag;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreferenceValue {
    Auto,
    Light,
    Dark,
    Oled,
    Retro,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum LayoutPreferenceValue {
    Grid,
    Rows,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum FriendRequestSource {
    None,
    Mutuals,
    Everyone,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum SharedInstanceSource {
    None,
    Friends,
    Everyone,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ThemePreference {
    pub value: ThemePreferenceValue,
    pub sync: bool,
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self {
            value: ThemePreferenceValue::Auto,
            sync: false,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(deny_unknown_fields)]
pub struct LanguagePreference {
    pub value: String,
    pub sync: bool,
}

impl Default for LanguagePreference {
    fn default() -> Self {
        Self {
            value: "en-US".to_string(),
            sync: false,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(deny_unknown_fields)]
pub struct LayoutPreferences {
    pub sync: bool,
    pub mods: LayoutPreferenceValue,
    pub plugins: LayoutPreferenceValue,
    pub datapacks: LayoutPreferenceValue,
    pub shaders: LayoutPreferenceValue,
    pub resourcepacks: LayoutPreferenceValue,
    pub modpacks: LayoutPreferenceValue,
    pub servers: LayoutPreferenceValue,
    pub users: LayoutPreferenceValue,
}

impl Default for LayoutPreferences {
    fn default() -> Self {
        Self {
            sync: true,
            mods: LayoutPreferenceValue::Rows,
            plugins: LayoutPreferenceValue::Rows,
            datapacks: LayoutPreferenceValue::Rows,
            shaders: LayoutPreferenceValue::Grid,
            resourcepacks: LayoutPreferenceValue::Grid,
            modpacks: LayoutPreferenceValue::Rows,
            servers: LayoutPreferenceValue::Rows,
            users: LayoutPreferenceValue::Rows,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SidebarPreferences {
    pub sync: bool,
    pub right_aligned_search: bool,
    pub left_aligned_content: bool,
}

impl Default for SidebarPreferences {
    fn default() -> Self {
        Self {
            sync: false,
            right_aligned_search: false,
            left_aligned_content: false,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SocialPreferences {
    pub friend_request_sources: FriendRequestSource,
    pub shared_instance_sources: SharedInstanceSource,
}

impl Default for SocialPreferences {
    fn default() -> Self {
        Self {
            friend_request_sources: FriendRequestSource::Everyone,
            shared_instance_sources: SharedInstanceSource::Friends,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(deny_unknown_fields)]
pub struct UserPreferences {
    pub theme: ThemePreference,
    pub language: LanguagePreference,
    pub layouts: LayoutPreferences,
    pub sidebars: SidebarPreferences,
    pub social: SocialPreferences,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: ThemePreference::default(),
            language: LanguagePreference::default(),
            layouts: LayoutPreferences::default(),
            sidebars: SidebarPreferences::default(),
            social: SocialPreferences::default(),
        }
    }
}

impl UserPreferences {
    pub fn validate(&self) -> Result<()> {
        validate_language(&self.language.value)
    }

    pub fn apply_patch(&mut self, patch: UserPreferencesPatch) {
        if let Some(theme) = patch.theme {
            if let Some(value) = theme.value {
                self.theme.value = value;
            }
            if let Some(sync) = theme.sync {
                self.theme.sync = sync;
            }
        }

        if let Some(language) = patch.language {
            if let Some(value) = language.value {
                self.language.value = value;
            }
            if let Some(sync) = language.sync {
                self.language.sync = sync;
            }
        }

        if let Some(layouts) = patch.layouts {
            if let Some(sync) = layouts.sync {
                self.layouts.sync = sync;
            }
            if let Some(mods) = layouts.mods {
                self.layouts.mods = mods;
            }
            if let Some(plugins) = layouts.plugins {
                self.layouts.plugins = plugins;
            }
            if let Some(datapacks) = layouts.datapacks {
                self.layouts.datapacks = datapacks;
            }
            if let Some(shaders) = layouts.shaders {
                self.layouts.shaders = shaders;
            }
            if let Some(resourcepacks) = layouts.resourcepacks {
                self.layouts.resourcepacks = resourcepacks;
            }
            if let Some(modpacks) = layouts.modpacks {
                self.layouts.modpacks = modpacks;
            }
            if let Some(servers) = layouts.servers {
                self.layouts.servers = servers;
            }
            if let Some(users) = layouts.users {
                self.layouts.users = users;
            }
        }

        if let Some(sidebars) = patch.sidebars {
            if let Some(sync) = sidebars.sync {
                self.sidebars.sync = sync;
            }
            if let Some(right_aligned_search) = sidebars.right_aligned_search {
                self.sidebars.right_aligned_search = right_aligned_search;
            }
            if let Some(left_aligned_content) = sidebars.left_aligned_content {
                self.sidebars.left_aligned_content = left_aligned_content;
            }
        }

        if let Some(social) = patch.social {
            if let Some(friend_request_sources) = social.friend_request_sources
            {
                self.social.friend_request_sources = friend_request_sources;
            }
            if let Some(shared_instance_sources) =
                social.shared_instance_sources
            {
                self.social.shared_instance_sources = shared_instance_sources;
            }
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UserPreferencesPatch {
    pub theme: Option<ThemePreferencePatch>,
    pub language: Option<LanguagePreferencePatch>,
    pub layouts: Option<LayoutPreferencesPatch>,
    pub sidebars: Option<SidebarPreferencesPatch>,
    pub social: Option<SocialPreferencesPatch>,
}

impl UserPreferencesPatch {
    pub fn validate(&self) -> Result<()> {
        if let Some(language) = self
            .language
            .as_ref()
            .and_then(|language| language.value.as_ref())
        {
            validate_language(language)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ThemePreferencePatch {
    pub value: Option<ThemePreferenceValue>,
    pub sync: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LanguagePreferencePatch {
    pub value: Option<String>,
    pub sync: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LayoutPreferencesPatch {
    pub sync: Option<bool>,
    pub mods: Option<LayoutPreferenceValue>,
    pub plugins: Option<LayoutPreferenceValue>,
    pub datapacks: Option<LayoutPreferenceValue>,
    pub shaders: Option<LayoutPreferenceValue>,
    pub resourcepacks: Option<LayoutPreferenceValue>,
    pub modpacks: Option<LayoutPreferenceValue>,
    pub servers: Option<LayoutPreferenceValue>,
    pub users: Option<LayoutPreferenceValue>,
}

#[derive(Debug, Clone, Default, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SidebarPreferencesPatch {
    pub sync: Option<bool>,
    pub right_aligned_search: Option<bool>,
    pub left_aligned_content: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SocialPreferencesPatch {
    pub friend_request_sources: Option<FriendRequestSource>,
    pub shared_instance_sources: Option<SharedInstanceSource>,
}

fn validate_language(language: &str) -> Result<()> {
    LanguageTag::parse(language).map(|_| ()).map_err(|_| {
        eyre!("`language.value` must be a valid BCP 47 language tag")
    })
}
