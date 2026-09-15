use serde::{Deserialize, Serialize};

/// The value shared between instances for one Minecraft setting.
///
/// Minecraft versions sometimes store the same setting in different ways. We
/// convert those formats into one of these values before syncing them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum CanonicalValue {
    Bool(bool),
    Integer(i64),
    Decimal(String),
    Enum(String),
    Text(String),
    StringList(Vec<String>),
    KeyBinding(String),
    ExternalRaw(String),
}

impl CanonicalValue {
    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Decimal(_) => "decimal",
            Self::Enum(_) => "enum",
            Self::Text(_) => "text",
            Self::StringList(_) => "string_list",
            Self::KeyBinding(_) => "key_binding",
            Self::ExternalRaw(_) => "external_raw",
        }
    }
}

/// Whether a setting comes from Minecraft itself or was added by a mod.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameOptionKind {
    Vanilla,
    External,
}

/// A shared game setting loaded from the database.
#[derive(Clone)]
pub(crate) struct StoredOption {
    pub(crate) option_id: String,
    pub(crate) kind: GameOptionKind,
    pub(crate) raw_key: Option<String>,
    pub(crate) value: Option<CanonicalValue>,
    pub(crate) seeded: bool,
    pub(crate) revision: u64,
}

/// Whether the user chose to sync this setting.
///
/// This is stored separately so turning sync off does not forget its value.
#[derive(Clone)]
pub(crate) struct StoredPreference {
    pub(crate) enabled: bool,
    pub(crate) revision: u64,
}
