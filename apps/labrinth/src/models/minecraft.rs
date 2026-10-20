//! Minecraft catalog access and compatibility parsing for legacy API payloads.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr, eyre};
use minecraft_meta_model::{
    model::{MinecraftVersion, MinecraftVersionName},
    upstream::mojang::VersionType,
};
use serde_json::Value;
use tokio::sync::OnceCell;

use crate::database::MinecraftMetaDb;
use crate::models::exp::minecraft::{Environment, ModVersionEdit, MrpackLoader};

static MINECRAFT_METADATA: LazyLock<OnceCell<Arc<MinecraftMetaDb>>> =
    LazyLock::new(OnceCell::new);

pub fn register_minecraft_metadata(
    metadata: actix_web::web::Data<MinecraftMetaDb>,
) -> Result<()> {
    MINECRAFT_METADATA.set(metadata.into_inner()).map_err(|_| {
        eyre!("minecraft-meta service is already initialized")
    })
}

async fn minecraft_metadata() -> Result<&'static MinecraftMetaDb> {
    Ok(MINECRAFT_METADATA
        .get_or_try_init(|| async {
            MinecraftMetaDb::connect().await.map(Arc::new)
        })
        .await?
        .as_ref())
}

/// Returns catalog records with the names historically exposed by Labrinth.
pub async fn minecraft_versions() -> Result<Vec<MinecraftVersion>> {
    let mut connection = minecraft_metadata()
        .await?
        .0
        .connection()
        .await
        .wrap_err("acquiring minecraft-meta connection")?;
    let mut versions = MinecraftVersion::all()
        .exec(&mut connection)
        .await
        .wrap_err("fetching Minecraft game versions")?;
    for version in &mut versions {
        version.name = MinecraftVersionName(
            legacy_game_version_name(&version.name.0).to_owned(),
        );
    }
    versions.sort_by(|a, b| {
        minecraft_version_date(b)
            .cmp(&minecraft_version_date(a))
            .then_with(|| a.name.0.cmp(&b.name.0))
    });
    Ok(versions)
}

pub fn minecraft_version_date(version: &MinecraftVersion) -> DateTime<Utc> {
    let timestamp = version.catalog_entry.0.release_time;
    let date = DateTime::from_timestamp(
        timestamp.as_second(),
        timestamp.subsec_nanosecond() as u32,
    )
    .expect("Minecraft release timestamp is representable by chrono");
    legacy_game_version_date(&version.name.0, date)
}

pub fn minecraft_version_type(version: &MinecraftVersion) -> &'static str {
    match version.catalog_entry.0.ty {
        VersionType::Release => "release",
        VersionType::Snapshot => "snapshot",
        VersionType::OldAlpha => "alpha",
        VersionType::OldBeta => "beta",
    }
}

pub async fn validate_minecraft_versions(
    names: &[MinecraftVersionName],
) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }
    let versions = minecraft_versions().await?;
    let known = versions.iter().map(|version| &version.name).collect::<HashSet<_>>();
    for name in names {
        eyre::ensure!(
            known.contains(name),
            "Minecraft version `{name}` is absent from minecraft-meta"
        );
    }
    Ok(())
}

/// Parses only the three flattened Minecraft fields retained by the v3 API.
pub fn parse_minecraft_fields(
    fields: &HashMap<String, Value>,
) -> Result<ModVersionEdit> {
    let mut edit = ModVersionEdit {
        game_versions: None,
        environment: None,
        mrpack_loaders: None,
    };
    for (name, value) in fields {
        match name.as_str() {
            "game_versions" => {
                edit.game_versions = Some(
                    string_array(value, name)?
                        .into_iter()
                        .map(|name| MinecraftVersionName(name.to_owned()))
                        .collect(),
                );
            }
            "environment" => {
                edit.environment = Some(if value.is_null() {
                    None
                } else {
                    Some(value.as_str()
                        .ok_or_else(|| eyre!("`environment` must be a string"))?
                        .parse::<Environment>()?)
                });
            }
            "mrpack_loaders" => {
                edit.mrpack_loaders = Some(
                    string_array(value, name)?
                        .into_iter()
                        .map(str::parse::<MrpackLoader>)
                        .collect::<Result<Vec<_>>>()?,
                );
            }
            _ => return Err(eyre!("unknown Minecraft field `{name}`")),
        }
    }
    Ok(edit)
}

fn string_array<'a>(value: &'a Value, field: &str) -> Result<Vec<&'a str>> {
    value.as_array()
        .ok_or_else(|| eyre!("`{field}` must be an array"))?
        .iter()
        .map(|value| value.as_str()
            .ok_or_else(|| eyre!("`{field}` must contain only strings")))
        .collect()
}

/// Mojang version names that require aliases for historical Labrinth compatibility.
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

pub(crate) fn legacy_game_version_name(name: &str) -> &str {
    HALL_OF_SHAME.iter()
        .find_map(|(canonical, legacy)| (*canonical == name).then_some(*legacy))
        .unwrap_or(name)
}

/// Mojank timestamps corrected to preserve historical release ordering.
pub(crate) static HALL_OF_SHAME_2: LazyLock<[(&str, DateTime<Utc>); 4]> =
    LazyLock::new(|| {
        [
            ("1.4.5", "2012-12-19T22:00:00+00:00"),
            ("1.4.6", "2012-12-19T22:00:01+00:00"),
            ("1.6.3", "2013-09-13T10:54:41+00:00"),
            ("13w37b", "2013-09-13T10:54:42+00:00"),
        ]
        .map(|(version, date)| {
            (version, DateTime::parse_from_rfc3339(date).unwrap().with_timezone(&Utc))
        })
    });

pub(crate) fn legacy_game_version_date(
    name: &str,
    date: DateTime<Utc>,
) -> DateTime<Utc> {
    HALL_OF_SHAME_2.iter()
        .find_map(|(version, corrected)| (*version == name).then_some(*corrected))
        .unwrap_or(date)
}
