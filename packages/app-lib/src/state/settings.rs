//! Theseus settings file

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;

// Types
/// Global Theseus settings
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub max_concurrent_downloads: usize,
    pub max_concurrent_writes: usize,

    pub theme: Theme,
    pub locale: String,
    pub default_page: DefaultPage,
    pub collapsed_navigation: bool,
    pub hide_nametag_skins_page: bool,
    pub advanced_rendering: bool,
    pub native_decorations: bool,
    pub toggle_sidebar: bool,
    pub sync_theme_across_devices: bool,
    pub sync_behavior_across_devices: bool,
    #[serde(default = "default_true")]
    pub sync_features_across_devices: bool,
    #[serde(default = "default_true")]
    pub show_files_tab_in_instances: bool,
    #[serde(default = "default_true")]
    pub show_worlds_tab_in_instances: bool,
    #[serde(default)]
    pub show_screenshots_tab_in_instances: bool,
    #[serde(default = "default_true")]
    pub show_skin_selector_in_sidebar: bool,

    #[serde(default = "default_true")]
    pub show_jump_in: bool,
    #[serde(default)]
    pub always_show_copy_details: bool,
    #[serde(default)]
    pub hide_installed_modpacks: bool,
    #[serde(default = "default_true")]
    pub advanced_filters_collapsed: bool,
    #[serde(default)]
    pub dismissed_photosensitivity_filter_warning: bool,
    #[serde(default)]
    pub friends_active_collapsed: bool,
    #[serde(default)]
    pub friends_online_collapsed: bool,
    #[serde(default = "default_true")]
    pub friends_offline_collapsed: bool,
    #[serde(default = "default_true")]
    pub friends_pending_collapsed: bool,

    pub telemetry: bool,
    pub discord_rpc: bool,
    pub personalized_ads: bool,

    pub extra_launch_args: Vec<String>,
    pub custom_env_vars: Vec<(String, String)>,
    pub memory: MemorySettings,
    pub force_fullscreen: bool,
    pub game_resolution: WindowSize,
    pub hide_on_process_start: bool,
    #[serde(default)]
    pub refocus_on_game_close: bool,
    #[serde(default)]
    pub compact_instance_cards: bool,
    #[serde(default = "default_true")]
    pub show_play_time: bool,
    #[serde(default = "default_true")]
    pub warn_on_unknown_modpacks: bool,
    #[serde(default)]
    pub skip_non_essential_warnings: bool,
    pub hooks: Hooks,

    pub custom_dir: Option<String>,
    pub prev_custom_dir: Option<String>,
    pub migrated: bool,

    pub developer_mode: bool,
    pub feature_flags: HashMap<FeatureFlag, bool>,

    pub skipped_update: Option<String>,
    pub pending_update_toast_for_version: Option<String>,
    pub auto_download_updates: Option<bool>,

    pub version: usize,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlag {
    PagePath,
    ProjectBackground,
    ServerRamAsBytesAlwaysOn,
    AlwaysShowAppControls,
    ShowSyncInstancesUpdateModal,
    PrideFundraiser,
    ServersInApp,
    ServerProjectQa,
    I18nDebug,
    LocalhostSignIn,
}

impl Settings {
    const CURRENT_VERSION: usize = 3;

    pub async fn get(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Self> {
        let res = sqlx::query!(
            "
            SELECT
                max_concurrent_writes, max_concurrent_downloads,
                theme, locale, default_page, collapsed_navigation, hide_nametag_skins_page, advanced_rendering, native_decorations,
                discord_rpc, developer_mode, telemetry, personalized_ads,
                json(extra_launch_args) extra_launch_args, json(custom_env_vars) custom_env_vars,
                mc_memory_max, mc_force_fullscreen, mc_game_resolution_x, mc_game_resolution_y, hide_on_process_start,
                hook_pre_launch, hook_wrapper, hook_post_exit,
                custom_dir, prev_custom_dir, migrated, json(feature_flags) feature_flags, toggle_sidebar,
                skipped_update, pending_update_toast_for_version, auto_download_updates,
				sync_theme_across_devices, sync_behavior_across_devices, sync_features_across_devices,
				show_files_tab_in_instances, show_worlds_tab_in_instances,
				show_screenshots_tab_in_instances, show_skin_selector_in_sidebar,
				refocus_on_game_close, compact_instance_cards, show_play_time, warn_on_unknown_modpacks, skip_non_essential_warnings,
				show_jump_in,
				always_show_copy_details,
				hide_installed_modpacks,
				advanced_filters_collapsed,
				dismissed_photosensitivity_filter_warning,
				friends_active_collapsed,
				friends_online_collapsed,
				friends_offline_collapsed,
				friends_pending_collapsed,
                version
            FROM settings
            "
        )
            .fetch_one(exec)
            .await?;

        Ok(Self {
            max_concurrent_downloads: res.max_concurrent_downloads as usize,
            max_concurrent_writes: res.max_concurrent_writes as usize,
            theme: Theme::from_string(&res.theme),
            locale: res.locale,
            default_page: DefaultPage::from_string(&res.default_page),
            collapsed_navigation: res.collapsed_navigation == 1,
            hide_nametag_skins_page: res.hide_nametag_skins_page == 1,
            advanced_rendering: res.advanced_rendering == 1,
            native_decorations: res.native_decorations == 1,
            toggle_sidebar: res.toggle_sidebar == 1,
            show_jump_in: res.show_jump_in == 1,
            always_show_copy_details: res.always_show_copy_details == 1,
            hide_installed_modpacks: res.hide_installed_modpacks == 1,
            advanced_filters_collapsed: res.advanced_filters_collapsed == 1,
            dismissed_photosensitivity_filter_warning: res
                .dismissed_photosensitivity_filter_warning
                == 1,
            friends_active_collapsed: res.friends_active_collapsed == 1,
            friends_online_collapsed: res.friends_online_collapsed == 1,
            friends_offline_collapsed: res.friends_offline_collapsed == 1,
            friends_pending_collapsed: res.friends_pending_collapsed == 1,
            telemetry: res.telemetry == 1,
            discord_rpc: res.discord_rpc == 1,
            developer_mode: res.developer_mode == 1,
            personalized_ads: res.personalized_ads == 1,
            extra_launch_args: res
                .extra_launch_args
                .as_ref()
                .and_then(|x| serde_json::from_str(x).ok())
                .unwrap_or_default(),
            custom_env_vars: res
                .custom_env_vars
                .as_ref()
                .and_then(|x| serde_json::from_str(x).ok())
                .unwrap_or_default(),
            memory: MemorySettings {
                maximum: res.mc_memory_max as u32,
            },
            force_fullscreen: res.mc_force_fullscreen == 1,
            game_resolution: WindowSize(
                res.mc_game_resolution_x as u16,
                res.mc_game_resolution_y as u16,
            ),
            hide_on_process_start: res.hide_on_process_start == 1,
            refocus_on_game_close: res.refocus_on_game_close == 1,
            compact_instance_cards: res.compact_instance_cards == 1,
            show_play_time: res.show_play_time == 1,
            warn_on_unknown_modpacks: res.warn_on_unknown_modpacks == 1,
            skip_non_essential_warnings: res.skip_non_essential_warnings == 1,
            hooks: Hooks {
                pre_launch: res.hook_pre_launch,
                wrapper: res.hook_wrapper,
                post_exit: res.hook_post_exit,
            },
            custom_dir: res.custom_dir,
            prev_custom_dir: res.prev_custom_dir,
            migrated: res.migrated == 1,
            feature_flags: res
                .feature_flags
                .as_ref()
                .and_then(|x| serde_json::from_str(x).ok())
                .unwrap_or_default(),
            skipped_update: res.skipped_update,
            pending_update_toast_for_version: res
                .pending_update_toast_for_version,
            auto_download_updates: res.auto_download_updates.map(|x| x == 1),
            sync_theme_across_devices: res.sync_theme_across_devices == 1,
            sync_behavior_across_devices: res.sync_behavior_across_devices == 1,
            sync_features_across_devices: res.sync_features_across_devices == 1,
            show_files_tab_in_instances: res.show_files_tab_in_instances == 1,
            show_worlds_tab_in_instances: res.show_worlds_tab_in_instances == 1,
            show_screenshots_tab_in_instances: res
                .show_screenshots_tab_in_instances
                == 1,
            show_skin_selector_in_sidebar: res.show_skin_selector_in_sidebar
                == 1,
            version: res.version as usize,
        })
    }

    pub async fn update(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let max_concurrent_writes = self.max_concurrent_writes as i32;
        let max_concurrent_downloads = self.max_concurrent_downloads as i32;
        let theme = self.theme.as_str();
        let default_page = self.default_page.as_str();
        let extra_launch_args = serde_json::to_string(&self.extra_launch_args)?;
        let custom_env_vars = serde_json::to_string(&self.custom_env_vars)?;
        let feature_flags = serde_json::to_string(&self.feature_flags)?;
        let version = self.version as i64;

        sqlx::query!(
            "
            UPDATE settings
            SET
                max_concurrent_writes = $1,
                max_concurrent_downloads = $2,

                theme = $3,
                locale = $4,
                default_page = $5,
                collapsed_navigation = $6,
                advanced_rendering = $7,
                native_decorations = $8,

                discord_rpc = $9,
                developer_mode = $10,
                telemetry = $11,
                personalized_ads = $12,

                extra_launch_args = jsonb($13),
                custom_env_vars = jsonb($14),
                mc_memory_max = $15,
                mc_force_fullscreen = $16,
                mc_game_resolution_x = $17,
                mc_game_resolution_y = $18,
                hide_on_process_start = $19,

                hook_pre_launch = $20,
                hook_wrapper = $21,
                hook_post_exit = $22,

                custom_dir = $23,
                prev_custom_dir = $24,
                migrated = $25,

                toggle_sidebar = $26,
                feature_flags = $27,
                hide_nametag_skins_page = $28,

                skipped_update = $29,
                pending_update_toast_for_version = $30,
                auto_download_updates = $31,

                sync_theme_across_devices = $32,
                sync_behavior_across_devices = $33,
				sync_features_across_devices = $34,
				show_files_tab_in_instances = $35,
				show_worlds_tab_in_instances = $36,
				show_screenshots_tab_in_instances = $37,
				show_skin_selector_in_sidebar = $38,

				version = $39,
				refocus_on_game_close = $40,
				compact_instance_cards = $41,
				show_play_time = $42,
				warn_on_unknown_modpacks = $43,
				skip_non_essential_warnings = $44,
				show_jump_in = $45,
				always_show_copy_details = $46,
				hide_installed_modpacks = $47,
				advanced_filters_collapsed = $48,
				dismissed_photosensitivity_filter_warning = $49,
				friends_active_collapsed = $50,
				friends_online_collapsed = $51,
				friends_offline_collapsed = $52,
				friends_pending_collapsed = $53
            ",
            max_concurrent_writes,
            max_concurrent_downloads,
            theme,
            self.locale,
            default_page,
            self.collapsed_navigation,
            self.advanced_rendering,
            self.native_decorations,
            self.discord_rpc,
            self.developer_mode,
            self.telemetry,
            self.personalized_ads,
            extra_launch_args,
            custom_env_vars,
            self.memory.maximum,
            self.force_fullscreen,
            self.game_resolution.0,
            self.game_resolution.1,
            self.hide_on_process_start,
            self.hooks.pre_launch,
            self.hooks.wrapper,
            self.hooks.post_exit,
            self.custom_dir,
            self.prev_custom_dir,
            self.migrated,
            self.toggle_sidebar,
            feature_flags,
            self.hide_nametag_skins_page,
            self.skipped_update,
            self.pending_update_toast_for_version,
            self.auto_download_updates,
            self.sync_theme_across_devices,
            self.sync_behavior_across_devices,
            self.sync_features_across_devices,
            self.show_files_tab_in_instances,
            self.show_worlds_tab_in_instances,
            self.show_screenshots_tab_in_instances,
            self.show_skin_selector_in_sidebar,
            version,
            self.refocus_on_game_close,
            self.compact_instance_cards,
            self.show_play_time,
            self.warn_on_unknown_modpacks,
            self.skip_non_essential_warnings,
            self.show_jump_in,
            self.always_show_copy_details,
            self.hide_installed_modpacks,
            self.advanced_filters_collapsed,
            self.dismissed_photosensitivity_filter_warning,
            self.friends_active_collapsed,
            self.friends_online_collapsed,
            self.friends_offline_collapsed,
            self.friends_pending_collapsed,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn migrate(exec: &Pool<Sqlite>) -> crate::Result<()> {
        let mut settings = Self::get(exec).await?;

        if settings.version < Settings::CURRENT_VERSION {
            tracing::info!(
                "Migrating settings version {} to {:?}",
                settings.version,
                Settings::CURRENT_VERSION
            );
        }
        while settings.version < Settings::CURRENT_VERSION {
            if let Err(err) = settings.perform_migration() {
                tracing::error!(
                    "Failed to migrate settings from version {}: {}",
                    settings.version,
                    err
                );
                return Err(err);
            }
        }

        settings.update(exec).await?;

        Ok(())
    }

    pub fn perform_migration(&mut self) -> crate::Result<()> {
        match self.version {
            1 => {
                let quoter = shlex::Quoter::new().allow_nul(true);

                // Previously split by spaces
                if let Some(pre_launch) = self.hooks.pre_launch.as_ref() {
                    self.hooks.pre_launch =
                        Some(quoter.join(pre_launch.split(' ')).unwrap())
                }

                // Previously treated as complete path to command
                if let Some(wrapper) = self.hooks.wrapper.as_ref() {
                    self.hooks.wrapper =
                        Some(quoter.quote(wrapper).unwrap().to_string())
                }

                // Previously split by spaces
                if let Some(post_exit) = self.hooks.post_exit.as_ref() {
                    self.hooks.post_exit =
                        Some(quoter.join(post_exit.split(' ')).unwrap())
                }

                self.version = 2;
            }
            2 => {
                // Update old default memory setting from 2GB to 4GB (depending on system memory)
                const LEGACY_DEFAULT_MEMORY_MB: u32 = 2048;
                if self.memory.maximum == LEGACY_DEFAULT_MEMORY_MB {
                    self.memory.maximum =
                        crate::api::jre::default_memory_max_mb();
                }

                self.version = 3;
            }
            version => {
                return Err(crate::ErrorKind::OtherError(format!(
                    "Invalid settings version: {version}"
                ))
                .into());
            }
        }

        Ok(())
    }
}

/// Theseus theme
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Dark,
    Light,
    Oled,
    Retro,
    System,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Oled => "oled",
            Theme::Retro => "retro",
            Theme::System => "system",
        }
    }

    pub fn from_string(string: &str) -> Theme {
        match string {
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            "oled" => Theme::Oled,
            "retro" => Theme::Retro,
            "system" => Theme::System,
            _ => Theme::Dark,
        }
    }
}

/// Minecraft memory settings
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct MemorySettings {
    pub maximum: u32,
}

/// Game window size
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct WindowSize(pub u16, pub u16);

/// Game initialization hooks
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde_with::serde_as]
pub struct Hooks {
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub pre_launch: Option<String>,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub wrapper: Option<String>,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub post_exit: Option<String>,
}

/// Opening window to start with
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum DefaultPage {
    Home,
    Library,
}

impl DefaultPage {
    pub fn as_str(&self) -> &'static str {
        match self {
            DefaultPage::Home => "home",
            DefaultPage::Library => "library",
        }
    }

    pub fn from_string(string: &str) -> Self {
        match string {
            "home" => Self::Home,
            "library" => Self::Library,
            _ => Self::Home,
        }
    }
}
