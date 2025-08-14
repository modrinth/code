// In V3, we switched to dynamic loader fields for a better support for more loaders, games, and potential metadata.
// This file contains the legacy loader fields, which are still used by V2 projects.
// They are still useful to have in several places where minecraft-java functionality is hardcoded- for example,
// for fetching data from forge, maven, etc.
// These fields only apply to minecraft-java, and are hardcoded to the minecraft-java game.

use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::database::redis::RedisPool;

use super::{
    DatabaseError, LoaderFieldEnumValueId,
    loader_fields::{
        LoaderFieldEnum, LoaderFieldEnumValue, VersionField, VersionFieldValue,
    },
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MinecraftGameVersion {
    pub id: LoaderFieldEnumValueId,
    pub version: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub created: DateTime<Utc>,
    pub major: bool,
}

impl MinecraftGameVersion {
    // The name under which this legacy field is stored as a LoaderField
    pub const FIELD_NAME: &'static str = "game_versions";

    pub fn builder() -> MinecraftGameVersionBuilder<'static> {
        MinecraftGameVersionBuilder::default()
    }

    pub async fn list<'a, E>(
        version_type_option: Option<&str>,
        major_option: Option<bool>,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<MinecraftGameVersion>, DatabaseError>
    where
        E: sqlx::Acquire<'a, Database = sqlx::Postgres>,
    {
        let mut exec = exec.acquire().await?;
        let game_version_enum =
            LoaderFieldEnum::get(Self::FIELD_NAME, &mut *exec, redis)
                .await?
                .ok_or_else(|| {
                    DatabaseError::SchemaError(
                        "Could not find game version enum.".to_string(),
                    )
                })?;
        let game_version_enum_values =
            LoaderFieldEnumValue::list(game_version_enum.id, &mut *exec, redis)
                .await?;

        let game_versions = game_version_enum_values
            .into_iter()
            .map(MinecraftGameVersion::from_enum_value)
            .filter(|x| {
                let mut bool = true;

                if let Some(version_type) = version_type_option {
                    bool &= &*x.type_ == version_type;
                }
                if let Some(major) = major_option {
                    bool &= x.major == major;
                }

                bool
            })
            .collect_vec();

        Ok(game_versions)
    }

    // Tries to create a MinecraftGameVersion from a VersionField
    // Clones on success
    pub fn try_from_version_field(
        version_field: &VersionField,
    ) -> Result<Vec<Self>, DatabaseError> {
        if version_field.field_name != Self::FIELD_NAME {
            return Err(DatabaseError::SchemaError(format!(
                "Field name {} is not {}",
                version_field.field_name,
                Self::FIELD_NAME
            )));
        }
        let game_versions = match version_field.clone() {
            VersionField {
                value: VersionFieldValue::ArrayEnum(_, values),
                ..
            } => values.into_iter().map(Self::from_enum_value).collect(),
            VersionField {
                value: VersionFieldValue::Enum(_, value),
                ..
            } => {
                vec![Self::from_enum_value(value)]
            }
            _ => {
                return Err(DatabaseError::SchemaError(format!(
                    "Game version requires field value to be an enum: {version_field:?}"
                )));
            }
        };
        Ok(game_versions)
    }

    pub fn from_enum_value(
        loader_field_enum_value: LoaderFieldEnumValue,
    ) -> MinecraftGameVersion {
        MinecraftGameVersion {
            id: loader_field_enum_value.id,
            version: loader_field_enum_value.value,
            created: loader_field_enum_value.created,
            type_: loader_field_enum_value
                .metadata
                .get("type")
                .and_then(|x| x.as_str())
                .map(|x| x.to_string())
                .unwrap_or_default(),
            major: loader_field_enum_value
                .metadata
                .get("major")
                .and_then(|x| x.as_bool())
                .unwrap_or_default(),
        }
    }
}

#[derive(Default)]
pub struct MinecraftGameVersionBuilder<'a> {
    pub version: Option<&'a str>,
    pub version_type: Option<&'a str>,
    pub date: Option<&'a DateTime<Utc>>,
}

impl<'a> MinecraftGameVersionBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    /// The game version.  Spaces must be replaced with '_' for it to be valid
    pub fn version(
        self,
        version: &'a str,
    ) -> Result<MinecraftGameVersionBuilder<'a>, DatabaseError> {
        Ok(Self {
            version: Some(version),
            ..self
        })
    }

    pub fn version_type(
        self,
        version_type: &'a str,
    ) -> Result<MinecraftGameVersionBuilder<'a>, DatabaseError> {
        Ok(Self {
            version_type: Some(version_type),
            ..self
        })
    }

    pub fn created(
        self,
        created: &'a DateTime<Utc>,
    ) -> MinecraftGameVersionBuilder<'a> {
        Self {
            date: Some(created),
            ..self
        }
    }

    pub async fn insert<'b, E>(
        self,
        exec: E,
        redis: &RedisPool,
    ) -> Result<LoaderFieldEnumValueId, DatabaseError>
    where
        E: sqlx::Executor<'b, Database = sqlx::Postgres> + Copy,
    {
        let game_versions_enum =
            LoaderFieldEnum::get("game_versions", exec, redis)
                .await?
                .ok_or(DatabaseError::SchemaError(
                    "Missing loaders field: 'game_versions'".to_string(),
                ))?;

        // Get enum id for game versions
        let metadata = json!({
            "type": self.version_type,
            "major": false
        });

        // This looks like a mess, but it *should* work
        // This allows game versions to be partially updated without
        // replacing the unspecified fields with defaults.
        let result = sqlx::query!(
            "
                INSERT INTO loader_field_enum_values (enum_id, value, created, metadata)
                VALUES ($1, $2, COALESCE($3, timezone('utc', now())), $4)
                ON CONFLICT (enum_id, value) DO UPDATE
                    SET metadata = jsonb_set(
                        COALESCE(loader_field_enum_values.metadata, $4),
                        '{type}', 
                        COALESCE($4->'type', loader_field_enum_values.metadata->'type')
                    ),
                    created = COALESCE($3, loader_field_enum_values.created)
                RETURNING id
                ",
            game_versions_enum.id.0,
            self.version,
            self.date.map(chrono::DateTime::naive_utc),
            metadata
        )
        .fetch_one(exec)
        .await?;

        let mut conn = redis.connect().await?;
        conn.delete(
            crate::database::models::loader_fields::LOADER_FIELD_ENUM_VALUES_NAMESPACE,
            game_versions_enum.id.0,
        )
        .await?;

        Ok(LoaderFieldEnumValueId(result.id))
    }
}
