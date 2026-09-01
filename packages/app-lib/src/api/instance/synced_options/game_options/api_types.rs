use crate::state::{CanonicalValue, GameOptionKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameOptionsSourceIssue {
    InstallingOrUpdating,
    Running,
    UnsupportedVersion,
    MissingOptionsFile,
    NoSyncableSettings,
    UnreadableOptionsFile,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptionsSourceCandidate {
    pub source_id: String,
    pub instance_id: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub game_version: Option<String>,
    pub eligible: bool,
    pub disabled_reason: Option<GameOptionsSourceIssue>,
    pub recognized_setting_count: u32,
    pub custom_setting_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSettingCategory {
    pub id: String,
    pub is_custom: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptionEditorDefinition {
    #[serde(rename = "type")]
    pub type_: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub unit: Option<String>,
    pub choices: Vec<GameOptionEditorChoice>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptionEditorChoice {
    pub value: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameOptionCompatibilityStatus {
    Ready,
    Deferred,
    WaitingForFile,
    WaitingForBase,
    NotAvailable,
    UnsupportedValue,
    Unmappable,
    CatalogUncovered,
    Controlled,
    Degraded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameOptionMappingKind {
    Direct,
    Legacy,
    Migrated,
    Lossless,
    Lossy,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameOptionCompatibilityReason {
    LauncherControlled,
    CatalogUncovered,
    InspectionFailed,
    WaitingForOptionsFile,
    UnsupportedValue,
    MigratesOnWrite,
    WaitingForCompatibleBase,
    MissingSetting,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptionCompatibilityBucket {
    pub instance_count: u32,
    pub write_keys: Vec<String>,
    pub eventual_keys: Vec<String>,
    pub game_versions: Vec<String>,
    pub status: GameOptionCompatibilityStatus,
    pub mapping: Option<GameOptionMappingKind>,
    pub reason: Option<GameOptionCompatibilityReason>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptionCompatibility {
    pub total_participating: u32,
    pub will_receive: u32,
    pub write_now: u32,
    pub left_local: u32,
    pub buckets: Vec<GameOptionCompatibilityBucket>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameOptionValueState {
    Canonical,
    UniformLocal,
    Mixed,
    Unset,
    Invalid,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameOptionValidationIssue {
    LocalValueNeedsSaving,
    MissingValue,
    NoCompatibleInstances,
    InvalidValue,
    ChangedSinceOpened,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditableGameSetting {
    pub option_id: String,
    pub category_id: String,
    pub kind: GameOptionKind,
    pub raw_key: Option<String>,
    pub sync_enabled: bool,
    pub canonical_value: Option<CanonicalValue>,
    pub value_state: GameOptionValueState,
    pub option_revision: u64,
    pub editor: GameOptionEditorDefinition,
    pub compatibility: GameOptionCompatibility,
    pub validation_error: Option<GameOptionValidationIssue>,
    pub controlled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSettingsEditorState {
    pub summary_revision: String,
    pub canonical_revision: u64,
    pub catalog_revision: u32,
    pub total_participating: u32,
    pub categories: Vec<GameSettingCategory>,
    pub settings: Vec<EditableGameSetting>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateGameSettingsRequest {
    pub expected_summary_revision: String,
    pub expected_canonical_revision: u64,
    pub expected_catalog_revision: u32,
    pub changes: Vec<GameSettingChange>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSettingChange {
    pub option_id: String,
    pub base_option_revision: u64,
    pub sync_enabled: Option<bool>,
    #[serde(
        default,
        with = "serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub canonical_value: Option<Option<CanonicalValue>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SaveGameSettingsResult {
    pub state: Option<GameSettingsEditorState>,
    pub applied: u32,
    pub migrated: u32,
    pub deferred: u32,
    pub unsupported: u32,
    pub failed: u32,
    pub conflicts: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::api::instance) enum SyncReason {
    Normal,
    BeforePackUpdate,
    PackExtracted,
    BeforeLaunch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SyncOutcome {
    Applied,
    Migrated,
    Unchanged,
    Deferred,
    WaitingForFile,
}

pub(super) fn canonical_values_equal(
    left: &CanonicalValue,
    right: &CanonicalValue,
) -> bool {
    match (left, right) {
        (CanonicalValue::Decimal(left), CanonicalValue::Decimal(right)) => {
            left.parse::<f64>().ok() == right.parse::<f64>().ok()
        }
        _ => left == right,
    }
}
