//! Minecraft compatibility adapters for legacy API consumers.

use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::LazyLock;

use crate::database::{
    PgTransaction,
    models::{
        ids::{DBVersionId, LoaderId},
        loader_item::Loader,
    },
};
use crate::models::exp;
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

/// Synthetic Minecraft compatibility identifier, not a database row ID.
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    utoipa::ToSchema,
)]
pub struct LoaderFieldId(pub i32);

/// Synthetic Minecraft compatibility identifier, not a database row ID.
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    utoipa::ToSchema,
)]
pub struct LoaderFieldEnumId(pub i32);

/// Synthetic Minecraft compatibility identifier, not a database row ID.
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    utoipa::ToSchema,
)]
pub struct LoaderFieldEnumValueId(pub i32);

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LoaderField {
    pub id: LoaderFieldId,
    pub field: String,
    pub field_type: LoaderFieldType,
    pub optional: bool,
    pub min_val: Option<i32>,
    pub max_val: Option<i32>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LoaderFieldType {
    Integer,
    Text,
    Enum(LoaderFieldEnumId),
    Boolean,
    ArrayInteger,
    ArrayText,
    ArrayEnum(LoaderFieldEnumId),
    ArrayBoolean,
}
impl LoaderFieldType {
    pub fn build(
        field_type_name: &str,
        loader_field_enum: Option<i32>,
    ) -> Option<LoaderFieldType> {
        Some(match (field_type_name, loader_field_enum) {
            ("integer", _) => LoaderFieldType::Integer,
            ("text", _) => LoaderFieldType::Text,
            ("boolean", _) => LoaderFieldType::Boolean,
            ("array_integer", _) => LoaderFieldType::ArrayInteger,
            ("array_text", _) => LoaderFieldType::ArrayText,
            ("array_boolean", _) => LoaderFieldType::ArrayBoolean,
            ("enum", Some(id)) => LoaderFieldType::Enum(LoaderFieldEnumId(id)),
            ("array_enum", Some(id)) => {
                LoaderFieldType::ArrayEnum(LoaderFieldEnumId(id))
            }
            _ => return None,
        })
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            LoaderFieldType::Integer => "integer",
            LoaderFieldType::Text => "text",
            LoaderFieldType::Boolean => "boolean",
            LoaderFieldType::ArrayInteger => "array_integer",
            LoaderFieldType::ArrayText => "array_text",
            LoaderFieldType::ArrayBoolean => "array_boolean",
            LoaderFieldType::Enum(_) => "enum",
            LoaderFieldType::ArrayEnum(_) => "array_enum",
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            LoaderFieldType::ArrayInteger => true,
            LoaderFieldType::ArrayText => true,
            LoaderFieldType::ArrayBoolean => true,
            LoaderFieldType::ArrayEnum(_) => true,

            LoaderFieldType::Integer => false,
            LoaderFieldType::Text => false,
            LoaderFieldType::Boolean => false,
            LoaderFieldType::Enum(_) => false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LoaderFieldEnum {
    pub id: LoaderFieldEnumId,
    pub enum_name: String,
    pub ordering: Option<i32>,
    pub hidable: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LoaderFieldEnumValue {
    pub id: LoaderFieldEnumValueId,
    pub enum_id: LoaderFieldEnumId,
    pub value: String,
    pub ordering: Option<i32>,
    pub created: DateTime<Utc>,
    #[serde(rename = "type")]
    pub ty: Option<String>,
    pub major: Option<bool>,
}

impl std::hash::Hash for LoaderFieldEnumValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.enum_id.hash(state);
        self.value.hash(state);
        self.ordering.hash(state);
        self.created.hash(state);
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct VersionField {
    pub version_id: DBVersionId,
    pub field_id: LoaderFieldId,
    pub field_name: String,
    pub value: VersionFieldValue,
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum VersionFieldValue {
    Integer(i32),
    Text(String),
    Enum(LoaderFieldEnumId, LoaderFieldEnumValue),
    Boolean(bool),
    ArrayInteger(Vec<i32>),
    ArrayText(Vec<String>),
    ArrayEnum(LoaderFieldEnumId, Vec<LoaderFieldEnumValue>),
    ArrayBoolean(Vec<bool>),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QueryVersionField {
    pub version_id: DBVersionId,
    pub field_id: LoaderFieldId,
    pub int_value: Option<i32>,
    pub enum_value: Option<LoaderFieldEnumValueId>,
    pub string_value: Option<String>,
}

impl QueryVersionField {
    pub fn with_int_value(mut self, int_value: i32) -> Self {
        self.int_value = Some(int_value);
        self
    }

    pub fn with_enum_value(
        mut self,
        enum_value: LoaderFieldEnumValueId,
    ) -> Self {
        self.enum_value = Some(enum_value);
        self
    }

    pub fn with_string_value(mut self, string_value: String) -> Self {
        self.string_value = Some(string_value);
        self
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QueryLoaderField {
    pub id: LoaderFieldId,
    pub field: String,
    pub field_type: String,
    pub enum_type: Option<LoaderFieldEnumId>,
    pub min_val: Option<i32>,
    pub max_val: Option<i32>,
    pub optional: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QueryLoaderFieldEnumValue {
    pub id: LoaderFieldEnumValueId,
    pub enum_id: LoaderFieldEnumId,
    pub value: String,
    pub ordering: Option<i32>,
    pub created: DateTime<Utc>,
    pub ty: Option<String>,
    pub major: Option<bool>,
}

/// Compatibility IDs are schema identifiers, not IDs from the retired tables.
fn minecraft_fields() -> Vec<LoaderField> {
    [
        (1, "game_versions", false, true),
        (2, "environment", true, false),
        (3, "mrpack_loaders", true, true),
    ]
    .into_iter()
    .map(|(id, field, optional, array)| LoaderField {
        id: LoaderFieldId(id),
        field: field.into(),
        optional,
        field_type: if array {
            LoaderFieldType::ArrayEnum(LoaderFieldEnumId(id))
        } else {
            LoaderFieldType::Enum(LoaderFieldEnumId(id))
        },
        min_val: if id == 1 { Some(1) } else { None },
        max_val: None,
    })
    .collect()
}

static MINECRAFT_METADATA: std::sync::LazyLock<
    tokio::sync::OnceCell<std::sync::Arc<crate::database::MinecraftMetaDb>>,
> = std::sync::LazyLock::new(tokio::sync::OnceCell::new);

/// Register the application's existing metadata service before compatibility lookups.
/// Call once during setup with `minecraft_meta_db.clone()`.
pub fn register_minecraft_metadata(
    metadata: actix_web::web::Data<crate::database::MinecraftMetaDb>,
) -> Result<()> {
    MINECRAFT_METADATA.set(metadata.into_inner()).map_err(|_| {
        eyre::eyre!("Minecraft metadata service is already initialized")
    })
}

pub(crate) async fn minecraft_metadata()
-> Result<&'static crate::database::MinecraftMetaDb> {
    Ok(MINECRAFT_METADATA
        .get_or_try_init(|| async {
            crate::database::MinecraftMetaDb::connect()
                .await
                .map(std::sync::Arc::new)
        })
        .await?
        .as_ref())
}

pub(crate) fn compatibility_value(
    enum_id: i32,
    value: String,
) -> LoaderFieldEnumValue {
    // FNV-1a is deterministic across processes and catalog ordering changes.
    let hash = value.bytes().fold(2166136261u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16777619)
    });
    LoaderFieldEnumValue {
        id: LoaderFieldEnumValueId((hash & 0x7fff_ffff) as i32),
        enum_id: LoaderFieldEnumId(enum_id),
        value,
        ordering: None,
        created: DateTime::<Utc>::UNIX_EPOCH,
        ty: None,
        major: None,
    }
}

impl LoaderField {
    pub async fn get_field<'a, E>(
        field: &str,
        loader_ids: &[LoaderId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<Self>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Self::get_fields(loader_ids, exec, redis)
            .await?
            .into_iter()
            .find(|item| item.field == field))
    }
    pub async fn get_fields<'a, E>(
        loader_ids: &[LoaderId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Self>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Self::get_fields_per_loader(loader_ids, exec, redis)
            .await?
            .into_values()
            .flatten()
            .unique_by(|field| field.id)
            .collect())
    }
    pub async fn get_fields_per_loader<'a, E>(
        loader_ids: &[LoaderId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<HashMap<LoaderId, Vec<Self>>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Loader::list(exec, redis)
            .await?
            .into_iter()
            .filter(|loader| loader_ids.contains(&loader.id))
            .map(|loader| {
                let fields = minecraft_fields()
                    .into_iter()
                    .filter(|field| {
                        loader
                            .supported_games
                            .iter()
                            .any(|game| game == "minecraft-java")
                            && (field.field != "mrpack_loaders"
                                || loader.loader == "mrpack")
                    })
                    .collect();
                (loader.id, fields)
            })
            .collect())
    }
    pub async fn get_fields_all<'a, E>(
        _exec: E,
        _redis: &RedisPool,
    ) -> Result<Vec<Self>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(minecraft_fields())
    }
}

impl LoaderFieldEnum {
    pub async fn get<'a, E>(
        enum_name: &str,
        _exec: E,
        _redis: &RedisPool,
    ) -> Result<Option<Self>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(minecraft_fields()
            .into_iter()
            .find(|field| field.field == enum_name)
            .map(|field| Self {
                id: LoaderFieldEnumId(field.id.0),
                enum_name: field.field,
                ordering: Some(field.id.0),
                hidable: field.id.0 == 1,
            }))
    }
}

impl LoaderFieldEnumValue {
    /// Fetches authoritative game-version metadata using an explicitly supplied service.
    pub async fn minecraft_catalog(
        db: &crate::database::MinecraftMetaDb,
    ) -> Result<Vec<Self>> {
        let values = {
            let mut conn =
                db.0.connection()
                    .await
                    .wrap_err("acquiring Minecraft metadata connection")?;
            minecraft_meta_model::model::MinecraftVersion::all()
                .exec(&mut conn)
                .await
                .wrap_err("fetching Minecraft version catalog")?
                .into_iter()
                .map(|row| {
                    let name = row.name.to_string();
                    let mut value = compatibility_value(
                        1,
                        legacy_game_version_name(&name).to_owned(),
                    );
                    value.created = DateTime::<Utc>::from_timestamp(
                        row.catalog_entry.0.release_time.as_second(),
                        row.catalog_entry.0.release_time.subsec_nanosecond()
                            as u32,
                    )
                    .expect("Minecraft timestamp is representable by chrono");
                    value.created =
                        legacy_game_version_date(&name, value.created);
                    use minecraft_meta_model::upstream::mojang::VersionType;
                    value.ty = Some(
                        match row.catalog_entry.0.ty {
                            VersionType::Release => "release",
                            VersionType::Snapshot => "snapshot",
                            VersionType::OldAlpha => "alpha",
                            VersionType::OldBeta => "beta",
                        }
                        .to_owned(),
                    );
                    value.major = Some(value.ty.as_deref() == Some("release"));
                    value
                })
                .collect::<Vec<_>>()
        };
        let mut seen = std::collections::HashSet::new();
        eyre::ensure!(
            values.iter().all(|value| seen.insert(value.id)),
            "compatibility enum value ID collision"
        );
        Ok(values)
    }

    pub async fn list<'a, E>(
        id: LoaderFieldEnumId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Self>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Self::list_many(&[id], exec, redis)
            .await?
            .remove(&id)
            .unwrap_or_default())
    }
    pub async fn list_many_loader_fields<'a, E>(
        fields: &[LoaderField],
        exec: E,
        redis: &RedisPool,
    ) -> Result<HashMap<LoaderFieldId, Vec<Self>>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let ids = fields
            .iter()
            .filter_map(|field| match field.field_type {
                LoaderFieldType::Enum(id) | LoaderFieldType::ArrayEnum(id) => {
                    Some(id)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let values = Self::list_many(&ids, exec, redis).await?;
        Ok(fields
            .iter()
            .filter_map(|field| match field.field_type {
                LoaderFieldType::Enum(id) | LoaderFieldType::ArrayEnum(id) => {
                    Some((field.id, values[&id].clone()))
                }
                _ => None,
            })
            .collect())
    }
    pub async fn list_many<'a, E>(
        ids: &[LoaderFieldEnumId],
        _exec: E,
        _redis: &RedisPool,
    ) -> Result<HashMap<LoaderFieldEnumId, Vec<Self>>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        use crate::models::exp::minecraft::{Environment, MrpackLoader};
        use strum::IntoEnumIterator;
        let mut result = HashMap::new();
        for &id in ids {
            let mut values = match id.0 {
                1 => {
                    Self::minecraft_catalog(minecraft_metadata().await?).await?
                }
                2 => Environment::iter()
                    .map(|value| {
                        compatibility_value(2, value.as_str().to_owned())
                    })
                    .collect(),
                3 => MrpackLoader::iter()
                    .map(|value| {
                        compatibility_value(3, value.as_str().to_owned())
                    })
                    .collect(),
                _ => eyre::bail!("unknown compatibility enum ID {}", id.0),
            };
            values.sort_by(|a, b| {
                b.created
                    .cmp(&a.created)
                    .then_with(|| a.value.cmp(&b.value))
            });
            let mut seen = std::collections::HashSet::new();
            eyre::ensure!(
                values.iter().all(|value| seen.insert(value.id)),
                "compatibility enum value ID collision"
            );
            result.insert(id, values);
        }
        Ok(result)
    }
    pub async fn list_filter<'a, E>(
        id: LoaderFieldEnumId,
        filter: HashMap<String, serde_json::Value>,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Self>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Self::list(id, exec, redis)
            .await?
            .into_iter()
            .filter(|value| {
                filter.iter().all(|(key, expected)| match key.as_str() {
                    "type" => value
                        .ty
                        .as_deref()
                        .is_some_and(|ty| expected.as_str() == Some(ty)),
                    "major" => value
                        .major
                        .is_some_and(|major| expected.as_bool() == Some(major)),
                    _ => false,
                })
            })
            .collect())
    }
}

impl VersionField {
    /// Converts compatibility field names and enum values directly into typed metadata.
    pub fn to_minecraft_component(
        fields: &[VersionField],
    ) -> Result<exp::minecraft::ModVersion> {
        use minecraft_meta_model::model::MinecraftVersionName;

        let mut component = exp::minecraft::ModVersion {
            game_versions: Vec::new(),
            environment: None,
            mrpack_loaders: Vec::new(),
        };
        let mut seen = std::collections::HashSet::new();
        for field in fields {
            eyre::ensure!(
                seen.insert(field.field_name.as_str()),
                "duplicate Minecraft compatibility field `{}`",
                field.field_name
            );
            match (field.field_name.as_str(), &field.value) {
                ("game_versions", VersionFieldValue::ArrayEnum(_, values)) => {
                    component.game_versions = values
                        .iter()
                        .map(|value| MinecraftVersionName(value.value.clone()))
                        .collect();
                }
                ("environment", VersionFieldValue::Enum(_, value)) => {
                    component.environment = Some(
                        value
                            .value
                            .parse()
                            .wrap_err("parsing Minecraft environment")?,
                    );
                }
                ("mrpack_loaders", VersionFieldValue::ArrayEnum(_, values)) => {
                    component.mrpack_loaders = values
                        .iter()
                        .map(|value| {
                            value
                                .value
                                .parse()
                                .wrap_err("parsing Minecraft mrpack loader")
                        })
                        .collect::<Result<Vec<_>>>()?;
                }
                ("game_versions" | "environment" | "mrpack_loaders", _) => {
                    eyre::bail!(
                        "invalid value type for Minecraft field `{}`",
                        field.field_name
                    );
                }
                _ => eyre::bail!(
                    "unknown Minecraft compatibility field `{}`",
                    field.field_name
                ),
            }
        }
        Ok(component)
    }

    /// Hydrates game versions before exposing legacy dates to validation or webhooks.
    pub async fn from_minecraft_component(
        version_id: DBVersionId,
        component: &crate::models::exp::minecraft::ModVersion,
    ) -> Result<Vec<VersionField>> {
        let catalog = LoaderFieldEnumValue::minecraft_catalog(
            minecraft_metadata().await?,
        )
        .await?;
        Self::from_minecraft_component_with_catalog(
            version_id, component, &catalog,
        )
    }

    /// Reuses one catalog across a batch of versions without additional database reads.
    pub fn from_minecraft_component_with_catalog(
        version_id: DBVersionId,
        component: &crate::models::exp::minecraft::ModVersion,
        catalog: &[LoaderFieldEnumValue],
    ) -> Result<Vec<VersionField>> {
        let mut fields = vec![
            Self {
                version_id,
                field_id: LoaderFieldId(1),
                field_name: "game_versions".into(),
                value: VersionFieldValue::ArrayEnum(
                    LoaderFieldEnumId(1),
                    component
                        .game_versions
                        .iter()
                        .map(|name| {
                            catalog
                                .iter()
                                .find(|value| {
                                    value.enum_id == LoaderFieldEnumId(1)
                                        && value.value == name.0
                                })
                                .cloned()
                                .ok_or_else(|| {
                                    eyre::eyre!(
                                        "Minecraft version `{name}` is absent from the metadata catalog"
                                    )
                                })
                        })
                        .collect::<Result<Vec<_>>>()?,
                ),
            },
            Self {
                version_id,
                field_id: LoaderFieldId(3),
                field_name: "mrpack_loaders".into(),
                value: VersionFieldValue::ArrayEnum(
                    LoaderFieldEnumId(3),
                    component
                        .mrpack_loaders
                        .iter()
                        .map(|value| {
                            compatibility_value(3, value.as_str().to_owned())
                        })
                        .collect(),
                ),
            },
        ];
        if let Some(environment) = component.environment {
            fields.push(Self {
                version_id,
                field_id: LoaderFieldId(2),
                field_name: "environment".into(),
                value: VersionFieldValue::Enum(
                    LoaderFieldEnumId(2),
                    compatibility_value(2, environment.as_str().to_owned()),
                ),
            });
        }
        Ok(fields)
    }

    pub async fn insert_many(
        items: Vec<Self>,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<()> {
        let mut patches = HashMap::<
            DBVersionId,
            serde_json::Map<String, serde_json::Value>,
        >::new();
        for item in items {
            let schema = minecraft_fields()
                .into_iter()
                .find(|field| {
                    field.field == item.field_name && field.id == item.field_id
                })
                .ok_or_else(|| {
                    eyre::eyre!(
                        "unknown Minecraft compatibility field `{}`",
                        item.field_name
                    )
                })?;
            let value = match (&schema.field_type, item.value) {
                (
                    LoaderFieldType::Enum(id),
                    VersionFieldValue::Enum(actual, value),
                ) if *id == actual => {
                    let _: crate::models::exp::minecraft::Environment =
                        value.value.parse()?;
                    serde_json::Value::String(value.value)
                }
                (
                    LoaderFieldType::ArrayEnum(id),
                    VersionFieldValue::ArrayEnum(actual, values),
                ) if *id == actual => {
                    if item.field_name == "mrpack_loaders" {
                        for value in &values {
                            let _: crate::models::exp::minecraft::MrpackLoader =
                                value.value.parse()?;
                        }
                    }
                    serde_json::Value::Array(
                        values
                            .into_iter()
                            .map(|value| serde_json::Value::String(value.value))
                            .collect(),
                    )
                }
                _ => eyre::bail!(
                    "invalid value type for Minecraft field `{}`",
                    item.field_name
                ),
            };

            patches
                .entry(item.version_id)
                .or_default()
                .insert(item.field_name, value);
        }
        for (version_id, patch) in patches {
            let patch = serde_json::Value::Object(patch);
            let result = sqlx::query!(
                r#"
                UPDATE versions SET components = jsonb_set(
                    COALESCE(components, '{}'::jsonb), '{minecraft_mod}',
                    jsonb_build_object('game_versions', '[]'::jsonb, 'environment', NULL, 'mrpack_loaders', '[]'::jsonb)
                        || COALESCE(NULLIF(components->'minecraft_mod', 'null'::jsonb), '{}'::jsonb)
                        || $2::jsonb
                ) WHERE id = $1
            "#,
                version_id.0,
                patch
            )
            .execute(&mut *transaction)
            .await
            .wrap_err("updating Minecraft version component")?;
            eyre::ensure!(
                result.rows_affected() == 1,
                "version {} does not exist",
                version_id.0
            );
        }
        Ok(())
    }

    pub fn check_parse(
        version_id: DBVersionId,
        loader_field: LoaderField,
        value: serde_json::Value,
        enum_variants: Vec<LoaderFieldEnumValue>,
    ) -> Result<VersionField, String> {
        let value =
            VersionFieldValue::parse(&loader_field, value, enum_variants)?;

        // Ensure, if applicable, that the value is within the min/max bounds
        let countable = match &value {
            VersionFieldValue::Integer(i) => Some(*i),
            VersionFieldValue::ArrayInteger(v) => Some(v.len() as i32),
            VersionFieldValue::Text(_) => None,
            VersionFieldValue::ArrayText(v) => Some(v.len() as i32),
            VersionFieldValue::Boolean(_) => None,
            VersionFieldValue::ArrayBoolean(v) => Some(v.len() as i32),
            VersionFieldValue::Enum(_, _) => None,
            VersionFieldValue::ArrayEnum(_, v) => Some(v.len() as i32),
        };

        if let Some(count) = countable {
            if let Some(min) = loader_field.min_val
                && count < min
            {
                return Err(format!(
                    "Provided value '{v}' for {field_name} is less than the minimum of {min}",
                    v = serde_json::to_string(&value).unwrap_or_default(),
                    field_name = loader_field.field,
                ));
            }

            if let Some(max) = loader_field.max_val
                && count > max
            {
                return Err(format!(
                    "Provided value '{v}' for {field_name} is greater than the maximum of {max}",
                    v = serde_json::to_string(&value).unwrap_or_default(),
                    field_name = loader_field.field,
                ));
            }
        }

        Ok(VersionField {
            version_id,
            field_id: loader_field.id,
            field_name: loader_field.field,
            value,
        })
    }

    pub fn from_query_json(
        // A list of all version fields to extract data from
        query_version_field_combined: Vec<QueryVersionField>,
        // A list of all loader fields to reference when extracting data
        // Note: any loader field in here that is not in query_version_field_combined will be still considered
        // (For example, game_versions in query_loader_fields but not in query_version_field_combined would produce game_versions: [])
        query_loader_fields: &[&QueryLoaderField],
        // enum values to reference when parsing enum values
        query_loader_field_enum_values: &[QueryLoaderFieldEnumValue],
        // If true, will allow multiple values for a single singleton field, returning them as separate VersionFields
        // allow_many = true, multiple Bools => two VersionFields of Bool
        // allow_many = false, multiple Bools => error
        // multiple Arraybools => 1 VersionField of ArrayBool
        allow_many: bool,
    ) -> Vec<VersionField> {
        query_loader_fields
            .iter()
            .flat_map(|q| {
                let Some(loader_field_type) = LoaderFieldType::build(
                    &q.field_type,
                    q.enum_type.map(|l| l.0),
                ) else {
                    return vec![];
                };
                let loader_field = LoaderField {
                    id: q.id,
                    field: q.field.clone(),
                    field_type: loader_field_type,
                    optional: q.optional,
                    min_val: q.min_val,
                    max_val: q.max_val,
                };

                // todo: avoid clone here?
                let version_fields = query_version_field_combined
                    .iter()
                    .filter(|qvf| qvf.field_id == q.id)
                    .cloned()
                    .collect::<Vec<_>>();
                if allow_many {
                    VersionField::build_many(
                        loader_field,
                        version_fields,
                        query_loader_field_enum_values,
                    )
                    .unwrap_or_default()
                    .into_iter()
                    .unique()
                    .collect_vec()
                } else {
                    match VersionField::build(
                        loader_field,
                        version_fields,
                        query_loader_field_enum_values,
                    ) {
                        Ok(vf) => vec![vf],
                        Err(_) => vec![],
                    }
                }
            })
            .collect()
    }

    pub fn build(
        loader_field: LoaderField,
        query_version_fields: Vec<QueryVersionField>,
        query_loader_field_enum_values: &[QueryLoaderFieldEnumValue],
    ) -> Result<VersionField> {
        let (version_id, value) = VersionFieldValue::build(
            &loader_field.field_type,
            query_version_fields,
            query_loader_field_enum_values,
        )
        .wrap_err("building version field value")?;
        Ok(VersionField {
            version_id,
            field_id: loader_field.id,
            field_name: loader_field.field,
            value,
        })
    }

    pub fn build_many(
        loader_field: LoaderField,
        query_version_fields: Vec<QueryVersionField>,
        query_loader_field_enum_values: &[QueryLoaderFieldEnumValue],
    ) -> Result<Vec<VersionField>> {
        let values = VersionFieldValue::build_many(
            &loader_field.field_type,
            query_version_fields,
            query_loader_field_enum_values,
        )
        .wrap_err("building version field values")?;
        Ok(values
            .into_iter()
            .map(|(version_id, value)| VersionField {
                version_id,
                field_id: loader_field.id,
                field_name: loader_field.field.clone(),
                value,
            })
            .collect())
    }
}

impl VersionFieldValue {
    // Build from user-submitted JSON data
    // value is the attempted value of the field, which will be tried to parse to the correct type
    // enum_array is the list of valid enum variants for the field, if it is an enum (see LoaderFieldEnumValue::list_many_loader_fields)
    pub fn parse(
        loader_field: &LoaderField,
        value: serde_json::Value,
        enum_array: Vec<LoaderFieldEnumValue>,
    ) -> Result<VersionFieldValue, String> {
        let field_name = &loader_field.field;
        let field_type = &loader_field.field_type;

        let error_value = value.clone();
        let incorrect_type_error = |field_type: &str| {
            format!(
                "Provided value '{v}' for {field_name} could not be parsed to {field_type} ",
                v = serde_json::to_string(&error_value).unwrap_or_default()
            )
        };

        Ok(match field_type {
            LoaderFieldType::Integer => VersionFieldValue::Integer(
                serde_json::from_value(value)
                    .map_err(|_| incorrect_type_error("integer"))?,
            ),
            LoaderFieldType::Text => VersionFieldValue::Text(
                value
                    .as_str()
                    .ok_or_else(|| incorrect_type_error("string"))?
                    .to_string(),
            ),
            LoaderFieldType::Boolean => VersionFieldValue::Boolean(
                value
                    .as_bool()
                    .ok_or_else(|| incorrect_type_error("boolean"))?,
            ),
            LoaderFieldType::ArrayInteger => VersionFieldValue::ArrayInteger({
                let array_values: Vec<i32> = serde_json::from_value(value)
                    .map_err(|_| incorrect_type_error("array of integers"))?;
                array_values.into_iter().collect()
            }),
            LoaderFieldType::ArrayText => VersionFieldValue::ArrayText({
                let array_values: Vec<String> = serde_json::from_value(value)
                    .map_err(|_| {
                    incorrect_type_error("array of strings")
                })?;
                array_values.into_iter().collect()
            }),
            LoaderFieldType::ArrayBoolean => VersionFieldValue::ArrayBoolean({
                let array_values: Vec<i64> = serde_json::from_value(value)
                    .map_err(|_| incorrect_type_error("array of booleans"))?;
                array_values.into_iter().map(|v| v != 0).collect()
            }),
            LoaderFieldType::Enum(id) => VersionFieldValue::Enum(*id, {
                let enum_value = value
                    .as_str()
                    .ok_or_else(|| incorrect_type_error("enum"))?;
                if let Some(ev) =
                    enum_array.into_iter().find(|v| v.value == enum_value)
                {
                    ev
                } else {
                    return Err(format!(
                        "Provided value '{enum_value}' is not a valid variant for {field_name}"
                    ));
                }
            }),
            LoaderFieldType::ArrayEnum(id) => {
                VersionFieldValue::ArrayEnum(*id, {
                    let array_values: Vec<String> =
                        serde_json::from_value(value).map_err(|_| {
                            incorrect_type_error("array of enums")
                        })?;
                    let mut enum_values = vec![];
                    for av in array_values {
                        if let Some(ev) =
                            enum_array.iter().find(|v| v.value == av)
                        {
                            enum_values.push(ev.clone());
                        } else {
                            return Err(format!(
                                "Provided value '{av}' is not a valid variant for {field_name}"
                            ));
                        }
                    }
                    enum_values
                })
            }
        })
    }

    // This will ensure that if multiple QueryVersionFields are provided, they can be combined into a single VersionFieldValue
    // of the appropriate type (ie: false, false, true -> ArrayBoolean([false, false, true])) (and not just Boolean)
    pub fn build(
        field_type: &LoaderFieldType,
        qvfs: Vec<QueryVersionField>,
        qlfev: &[QueryLoaderFieldEnumValue],
    ) -> Result<(DBVersionId, VersionFieldValue)> {
        match field_type {
            LoaderFieldType::Integer
            | LoaderFieldType::Text
            | LoaderFieldType::Boolean
            | LoaderFieldType::Enum(_) => {
                let mut fields = Self::build_many(field_type, qvfs, qlfev)
                    .wrap_err("building singleton version field values")?;
                if fields.len() > 1 {
                    return Err(eyre::eyre!(
                        "Multiple fields for field {}",
                        field_type.to_str()
                    ));
                }
                fields.pop().ok_or_else(|| {
                    eyre::eyre!(
                        "No version fields for field {}",
                        field_type.to_str()
                    )
                })
            }
            LoaderFieldType::ArrayInteger
            | LoaderFieldType::ArrayText
            | LoaderFieldType::ArrayBoolean
            | LoaderFieldType::ArrayEnum(_) => {
                let fields = Self::build_many(field_type, qvfs, qlfev)
                    .wrap_err("building array version field values")?;
                fields.into_iter().next().ok_or_else(|| {
                    eyre::eyre!(
                        "No version fields for field {}",
                        field_type.to_str()
                    )
                })
            }
        }
    }

    // Build from internal query data
    // This encapsulates redundant behavior in db query -> object conversions
    // This allows for multiple fields to be built at once. If there are multiple fields,
    // but the type only allows for a single field, then multiple VersionFieldValues will be returned
    // If there are multiple fields, and the type allows for multiple fields, then a single VersionFieldValue will be returned (array.len == 1)
    pub fn build_many(
        field_type: &LoaderFieldType,
        qvfs: Vec<QueryVersionField>,
        qlfev: &[QueryLoaderFieldEnumValue],
    ) -> Result<Vec<(DBVersionId, VersionFieldValue)>> {
        let field_name = field_type.to_str();
        let did_not_exist_error = |field_name: &str, desired_field: &str| {
            eyre::eyre!(
                "Field name {desired_field} for field {field_name} in does not exist"
            )
        };

        // Check errors- version_id must all be the same
        // If the field type is a non-array, then the reason for multiple version ids is that there are multiple versions being aggregated, and those version ids are contained within.
        // If the field type is an array, then the reason for multiple version ids is that there are multiple values for a single version
        // (or a greater aggregation between multiple arrays, in which case the per-field version is lost, so we just take the first one and use it for that)
        let version_id = qvfs
            .iter()
            .map(|qvf| qvf.version_id)
            .unique()
            .next()
            .unwrap_or(DBVersionId(0));

        if qvfs.iter().map(|qvf| qvf.field_id).unique().count() > 1 {
            return Err(eyre::eyre!(
                "Multiple field ids for field {field_name}"
            ));
        }

        let mut value = match field_type {
            // Singleton fields
            // If there are multiple, we assume multiple versions are being concatenated
            LoaderFieldType::Integer => qvfs
                .into_iter()
                .map(|qvf| {
                    eyre::Ok((
                        qvf.version_id,
                        VersionFieldValue::Integer(
                            qvf.int_value
                                .ok_or_else(|| {
                                    did_not_exist_error(field_name, "int_value")
                                })
                                .wrap_err(
                                    "reading integer version field value",
                                )?,
                        ),
                    ))
                })
                .collect::<Result<Vec<_>>>()
                .wrap_err("building integer version field values")?,
            LoaderFieldType::Text => qvfs
                .into_iter()
                .map(|qvf| {
                    eyre::Ok((
                        qvf.version_id,
                        VersionFieldValue::Text(
                            qvf.string_value
                                .ok_or_else(|| {
                                    did_not_exist_error(
                                        field_name,
                                        "string_value",
                                    )
                                })
                                .wrap_err("reading text version field value")?,
                        ),
                    ))
                })
                .collect::<Result<Vec<_>>>()
                .wrap_err("building text version field values")?,
            LoaderFieldType::Boolean => qvfs
                .into_iter()
                .map(|qvf| {
                    eyre::Ok((
                        qvf.version_id,
                        VersionFieldValue::Boolean(
                            qvf.int_value
                                .ok_or_else(|| {
                                    did_not_exist_error(field_name, "int_value")
                                })
                                .wrap_err(
                                    "reading boolean version field value",
                                )?
                                != 0,
                        ),
                    ))
                })
                .collect::<Result<Vec<_>>>()
                .wrap_err("building boolean version field values")?,
            LoaderFieldType::Enum(id) => qvfs
                .into_iter()
                .map(|qvf| {
                    eyre::Ok((
                        qvf.version_id,
                        VersionFieldValue::Enum(*id, {
                            let enum_id = qvf
                                .enum_value
                                .ok_or_else(|| {
                                    did_not_exist_error(
                                        field_name,
                                        "enum_value",
                                    )
                                })
                                .wrap_err("reading enum version field value")?;
                            let lfev = qlfev
                                .iter()
                                .find(|x| x.id == enum_id)
                                .ok_or_else(|| {
                                    did_not_exist_error(
                                        field_name,
                                        "enum_value",
                                    )
                                })
                                .wrap_err("finding loader field enum value")?;
                            LoaderFieldEnumValue {
                                id: lfev.id,
                                enum_id: lfev.enum_id,
                                value: lfev.value.clone(),
                                ordering: lfev.ordering,
                                created: lfev.created,
                                ty: lfev.ty.clone(),
                                major: lfev.major,
                            }
                        }),
                    ))
                })
                .collect::<Result<Vec<_>>>()
                .wrap_err("building enum version field values")?,

            // Array fields
            // We concatenate into one array
            LoaderFieldType::ArrayInteger => vec![(
                version_id,
                VersionFieldValue::ArrayInteger(
                    qvfs.into_iter()
                        .map(|qvf| {
                            qvf.int_value.ok_or_else(|| {
                                did_not_exist_error(field_name, "int_value")
                            })
                        })
                        .collect::<Result<_>>()
                        .wrap_err("building integer array version field value")?,
                ),
            )],
            LoaderFieldType::ArrayText => vec![(
                version_id,
                VersionFieldValue::ArrayText(
                    qvfs.into_iter()
                        .map(|qvf| {
                            qvf.string_value.ok_or_else(|| {
                                did_not_exist_error(field_name, "string_value")
                            })
                        })
                        .collect::<Result<_>>()
                        .wrap_err("building text array version field value")?,
                ),
            )],
            LoaderFieldType::ArrayBoolean => vec![(
                version_id,
                VersionFieldValue::ArrayBoolean(
                    qvfs.into_iter()
                        .map(|qvf| {
                            eyre::Ok(
                                qvf.int_value
                                    .ok_or_else(|| {
                                        did_not_exist_error(
                                            field_name,
                                            "int_value",
                                        )
                                    })
                                    .wrap_err(
                                        "reading boolean array version field value",
                                    )?
                                    != 0,
                            )
                        })
                        .collect::<Result<_>>()
                        .wrap_err("building boolean array version field value")?,
                ),
            )],
            LoaderFieldType::ArrayEnum(id) => vec![(
                version_id,
                VersionFieldValue::ArrayEnum(
                    *id,
                    qvfs.into_iter()
                        .map(|qvf| {
                            let enum_id = qvf
                                .enum_value
                                .ok_or_else(|| {
                                    did_not_exist_error(field_name, "enum_value")
                                })
                                .wrap_err(
                                    "reading enum array version field value",
                                )?;
                            let lfev = qlfev
                                .iter()
                                .find(|x| x.id == enum_id)
                                .ok_or_else(|| {
                                    did_not_exist_error(field_name, "enum_value")
                                })
                                .wrap_err(
                                    "finding loader field enum array value",
                                )?;
                            eyre::Ok(LoaderFieldEnumValue {
                                id: lfev.id,
                                enum_id: lfev.enum_id,
                                value: lfev.value.clone(),
                                ordering: lfev.ordering,
                                created: lfev.created,
                                ty: lfev.ty.clone(),
                                major: lfev.major,
                            })
                        })
                        .collect::<Result<_>>()
                        .wrap_err("building enum array version field value")?,
                ),
            )],
        };

        // Sort arrayenums by ordering, then by created
        for (_, v) in &mut value {
            if let VersionFieldValue::ArrayEnum(_, v) = v {
                v.sort_by(|a, b| {
                    a.ordering.cmp(&b.ordering).then(a.created.cmp(&b.created))
                });
            }
        }

        Ok(value)
    }

    // Serialize to internal value, such as for converting to user-facing JSON
    pub fn serialize_internal(&self) -> serde_json::Value {
        match self {
            VersionFieldValue::Integer(i) => {
                serde_json::Value::Number((*i).into())
            }
            VersionFieldValue::Text(s) => serde_json::Value::String(s.clone()),
            VersionFieldValue::Boolean(b) => serde_json::Value::Bool(*b),
            VersionFieldValue::ArrayInteger(v) => serde_json::Value::Array(
                v.iter()
                    .map(|i| serde_json::Value::Number((*i).into()))
                    .collect(),
            ),
            VersionFieldValue::ArrayText(v) => serde_json::Value::Array(
                v.iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            ),
            VersionFieldValue::ArrayBoolean(v) => serde_json::Value::Array(
                v.iter().map(|b| serde_json::Value::Bool(*b)).collect(),
            ),
            VersionFieldValue::Enum(_, v) => {
                serde_json::Value::String(v.value.clone())
            }
            VersionFieldValue::ArrayEnum(_, v) => serde_json::Value::Array(
                v.iter()
                    .map(|v| serde_json::Value::String(v.value.clone()))
                    .collect(),
            ),
        }
    }

    // For conversion to an internal string(s), such as for search facets, filtering, or direct hardcoding
    // No matter the type, it will be converted to a Vec<String>, where the non-array types will have a single element
    pub fn as_strings(&self) -> Vec<String> {
        match self {
            VersionFieldValue::Integer(i) => vec![i.to_string()],
            VersionFieldValue::Text(s) => vec![s.clone()],
            VersionFieldValue::Boolean(b) => vec![b.to_string()],
            VersionFieldValue::ArrayInteger(v) => {
                v.iter().map(|i| i.to_string()).collect()
            }
            VersionFieldValue::ArrayText(v) => v.clone(),
            VersionFieldValue::ArrayBoolean(v) => {
                v.iter().map(|b| b.to_string()).collect()
            }
            VersionFieldValue::Enum(_, v) => vec![v.value.clone()],
            VersionFieldValue::ArrayEnum(_, v) => {
                v.iter().map(|v| v.value.clone()).collect()
            }
        }
    }

    pub fn contains_json_value(&self, value: &serde_json::Value) -> bool {
        match self {
            VersionFieldValue::Integer(i) => value.as_i64() == Some(*i as i64),
            VersionFieldValue::Text(s) => value.as_str() == Some(s),
            VersionFieldValue::Boolean(b) => value.as_bool() == Some(*b),
            VersionFieldValue::ArrayInteger(v) => {
                value.as_i64().is_some_and(|i| v.contains(&(i as i32)))
            }
            VersionFieldValue::ArrayText(v) => {
                value.as_str().is_some_and(|s| v.contains(&s.to_string()))
            }
            VersionFieldValue::ArrayBoolean(v) => {
                value.as_bool().is_some_and(|b| v.contains(&b))
            }
            VersionFieldValue::Enum(_, v) => value.as_str() == Some(&v.value),
            VersionFieldValue::ArrayEnum(_, v) => value
                .as_str()
                .is_some_and(|s| v.iter().any(|v| v.value == s)),
        }
    }
}

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
