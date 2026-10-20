// In V3, we switched to dynamic loader fields for a better support for more loaders, games, and potential metadata.
// This file contains the legacy loader fields, which are still used by V2 projects.
// They are still useful to have in several places where minecraft-java functionality is hardcoded- for example,
// for fetching data from forge, maven, etc.
// These fields only apply to minecraft-java, and are hardcoded to the minecraft-java game.

use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use xredis::RedisPool;

use super::{
    LoaderFieldEnumId, LoaderFieldEnumValueId,
    loader_fields::{
        LoaderFieldEnum, LoaderFieldEnumValue, VersionField, VersionFieldValue,
    },
};

/// Mojang version names that require aliases for historical Labrinth compatibility.
// A list of version names that contains spaces.
// Generated using the command
// ```sh
// curl https://launchermeta.mojang.com/mc/game/version_manifest.json \
//      | jq '[.versions[].id | select(contains(" "))]'
// ```
pub(crate) const HALL_OF_SHAME: [(&str, &str); 12] = [
    ("1.14.2 Pre-Release 4", "1.14.2-pre4"),
    ("1.14.2 Pre-Release 3", "1.14.2-pre3"),
    ("1.14.2 Pre-Release 2", "1.14.2-pre2"),
    ("1.14.2 Pre-Release 1", "1.14.2-pre1"),
    ("1.14.1 Pre-Release 2", "1.14.1-pre2"),
    ("1.14.1 Pre-Release 1", "1.14.1-pre1"),
    ("1.14 Pre-Release 5", "1.14-pre5"),
    ("1.14 Pre-Release 4", "1.14-pre4"),
    ("1.14 Pre-Release 3", "1.14-pre3"),
    ("1.14 Pre-Release 2", "1.14-pre2"),
    ("1.14 Pre-Release 1", "1.14-pre1"),
    ("3D Shareware v1.34", "3D-Shareware-v1.34"),
];

/// Preserves historical Labrinth aliases while leaving other catalog names unchanged.
pub(crate) fn legacy_game_version_name(name: &str) -> &str {
    HALL_OF_SHAME
        .iter()
        .find_map(|(canonical, legacy)| (*canonical == name).then_some(*legacy))
        .unwrap_or(name)
}

/// Mojank for some reason has versions released at the same DateTime. This hardcodes them to fix this,
/// as most of our ordering logic is with DateTime
pub(crate) static HALL_OF_SHAME_2: LazyLock<[(&str, DateTime<Utc>); 4]> =
    LazyLock::new(|| {
        [
            ("1.4.5", "2012-12-19T22:00:00+00:00"),
            ("1.4.6", "2012-12-19T22:00:01+00:00"),
            ("1.6.3", "2013-09-13T10:54:41+00:00"),
            ("13w37b", "2013-09-13T10:54:42+00:00"),
        ]
        .map(|(version, date)| {
            (
                version,
                DateTime::parse_from_rfc3339(date)
                    .unwrap()
                    .with_timezone(&Utc),
            )
        })
    });

/// Preserves the historical ordering of releases with identical upstream timestamps.
pub(crate) fn legacy_game_version_date(
    name: &str,
    date: DateTime<Utc>,
) -> DateTime<Utc> {
    HALL_OF_SHAME_2
        .iter()
        .find_map(|(version, corrected)| {
            (*version == name).then_some(*corrected)
        })
        .unwrap_or(date)
}

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
    ) -> Result<Vec<MinecraftGameVersion>>
    where
        E: crate::database::Acquire<'a, Database = sqlx::Postgres>,
    {
        let mut exec = exec
            .acquire()
            .await
            .wrap_err("acquiring database connection")?;
        let game_version_enum =
            LoaderFieldEnum::get(Self::FIELD_NAME, &mut exec, redis)
                .await
                .wrap_err("fetching game version enum")?
                .ok_or_else(|| eyre::eyre!("Could not find game version enum."))
                .wrap_err("finding game version enum")?;
        let game_version_enum_values =
            LoaderFieldEnumValue::list(game_version_enum.id, &mut exec, redis)
                .await
                .wrap_err("fetching game version enum values")?;

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
    ) -> Result<Vec<Self>> {
        if version_field.field_name != Self::FIELD_NAME {
            return Err(eyre::eyre!(
                "Field name {} is not {}",
                version_field.field_name,
                Self::FIELD_NAME
            ));
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
                return Err(eyre::eyre!(
                    "Game version requires field value to be an enum: {version_field:?}"
                ));
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
            type_: loader_field_enum_value.ty.unwrap_or_default(),
            major: loader_field_enum_value.major.unwrap_or_default(),
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
    ) -> Result<MinecraftGameVersionBuilder<'a>> {
        Ok(Self {
            version: Some(version),
            ..self
        })
    }

    pub fn version_type(
        self,
        version_type: &'a str,
    ) -> Result<MinecraftGameVersionBuilder<'a>> {
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

	/// Verifies an entry owned by the metadata generator; never creates incomplete catalog rows.
	pub async fn insert<'b, E>(
		self,
		exec: E,
		redis: &RedisPool,
	) -> Result<LoaderFieldEnumValueId>
	where
		E: crate::database::Executor<'b, Database = sqlx::Postgres> + Copy,
	{
		let version = self
			.version
			.ok_or_else(|| eyre::eyre!("missing Minecraft version name"))?;
		let values =
			LoaderFieldEnumValue::list(LoaderFieldEnumId(1), exec, redis)
				.await?;
		let existing = values.into_iter().find(|value| value.value == version)
			.ok_or_else(|| eyre::eyre!("Minecraft version `{version}` is absent from the metadata catalog; ingest it with the metadata generator"))?;
		if let Some(ty) = self.version_type {
			eyre::ensure!(
				existing.ty.as_deref() == Some(ty),
				"Minecraft version `{version}` type differs from the metadata catalog; update it with the metadata generator"
			);
		}
		if let Some(date) = self.date {
			eyre::ensure!(
				existing.created == *date,
				"Minecraft version `{version}` release date differs from the metadata catalog; update it with the metadata generator"
			);
		}
		Ok(existing.id)
	}
}
