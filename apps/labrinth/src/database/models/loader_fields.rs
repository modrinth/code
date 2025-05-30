use std::collections::HashMap;
use std::hash::Hasher;

use super::DatabaseError;
use super::ids::*;
use crate::database::redis::RedisPool;
use chrono::DateTime;
use chrono::Utc;
use dashmap::DashMap;
use futures::TryStreamExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

const GAMES_LIST_NAMESPACE: &str = "games";
const LOADER_ID: &str = "loader_id";
const LOADERS_LIST_NAMESPACE: &str = "loaders";
const LOADER_FIELDS_NAMESPACE: &str = "loader_fields";
const LOADER_FIELDS_NAMESPACE_ALL: &str = "loader_fields_all";
const LOADER_FIELD_ENUMS_ID_NAMESPACE: &str = "loader_field_enums";
pub const LOADER_FIELD_ENUM_VALUES_NAMESPACE: &str = "loader_field_enum_values";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Game {
    pub id: GameId,
    pub slug: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
}

impl Game {
    pub async fn get_slug<'a, E>(
        slug: &str,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<Game>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Self::list(exec, redis)
            .await?
            .into_iter()
            .find(|x| x.slug == slug))
    }

    pub async fn list<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Game>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;
        let cached_games: Option<Vec<Game>> = redis
            .get_deserialized_from_json(GAMES_LIST_NAMESPACE, "games")
            .await?;
        if let Some(cached_games) = cached_games {
            return Ok(cached_games);
        }

        let result = sqlx::query!(
            "
            SELECT id, slug, name, icon_url, banner_url FROM games
            ",
        )
        .fetch(exec)
        .map_ok(|x| Game {
            id: GameId(x.id),
            slug: x.slug,
            name: x.name,
            icon_url: x.icon_url,
            banner_url: x.banner_url,
        })
        .try_collect::<Vec<Game>>()
        .await?;

        redis
            .set_serialized_to_json(
                GAMES_LIST_NAMESPACE,
                "games",
                &result,
                None,
            )
            .await?;

        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Loader {
    pub id: LoaderId,
    pub loader: String,
    pub icon: String,
    pub supported_project_types: Vec<String>,
    pub supported_games: Vec<String>, // slugs
    pub metadata: serde_json::Value,
}

impl Loader {
    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<LoaderId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;
        let cached_id: Option<i32> =
            redis.get_deserialized_from_json(LOADER_ID, name).await?;
        if let Some(cached_id) = cached_id {
            return Ok(Some(LoaderId(cached_id)));
        }

        let result = sqlx::query!(
            "
            SELECT id FROM loaders
            WHERE loader = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await?
        .map(|r| LoaderId(r.id));

        if let Some(result) = result {
            redis
                .set_serialized_to_json(LOADER_ID, name, &result.0, None)
                .await?;
        }

        Ok(result)
    }

    pub async fn list<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<Loader>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;
        let cached_loaders: Option<Vec<Loader>> = redis
            .get_deserialized_from_json(LOADERS_LIST_NAMESPACE, "all")
            .await?;
        if let Some(cached_loaders) = cached_loaders {
            return Ok(cached_loaders);
        }

        let result = sqlx::query!(
            "
            SELECT l.id id, l.loader loader, l.icon icon, l.metadata metadata,
            ARRAY_AGG(DISTINCT pt.name) filter (where pt.name is not null) project_types,
            ARRAY_AGG(DISTINCT g.slug) filter (where g.slug is not null) games
            FROM loaders l
            LEFT OUTER JOIN loaders_project_types lpt ON joining_loader_id = l.id
            LEFT OUTER JOIN project_types pt ON lpt.joining_project_type_id = pt.id
            LEFT OUTER JOIN loaders_project_types_games lptg ON lptg.loader_id = lpt.joining_loader_id AND lptg.project_type_id = lpt.joining_project_type_id
            LEFT OUTER JOIN games g ON lptg.game_id = g.id
            GROUP BY l.id;
            ",
        )
        .fetch(exec)
        .map_ok(|x| Loader {
            id: LoaderId(x.id),
            loader: x.loader,
            icon: x.icon,
            supported_project_types: x
                .project_types
                .unwrap_or_default()
                .iter()
                .map(|x| x.to_string())
                .collect(),
            supported_games: x
                .games
                .unwrap_or_default(),
            metadata: x.metadata
        })
        .try_collect::<Vec<_>>()
        .await?;

        redis
            .set_serialized_to_json(
                LOADERS_LIST_NAMESPACE,
                "all",
                &result,
                None,
            )
            .await?;

        Ok(result)
    }
}

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
    #[serde(flatten)]
    pub metadata: serde_json::Value,
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
    pub metadata: Option<serde_json::Value>,
}

impl LoaderField {
    pub async fn get_field<'a, E>(
        field: &str,
        loader_ids: &[LoaderId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<LoaderField>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let fields = Self::get_fields(loader_ids, exec, redis).await?;
        Ok(fields.into_iter().find(|f| f.field == field))
    }

    // Gets all fields for a given loader(s)
    // Returns all as this there are probably relatively few fields per loader
    pub async fn get_fields<'a, E>(
        loader_ids: &[LoaderId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<LoaderField>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let found_loader_fields =
            Self::get_fields_per_loader(loader_ids, exec, redis).await?;
        let result = found_loader_fields
            .into_values()
            .flatten()
            .unique_by(|x| x.id)
            .collect();
        Ok(result)
    }

    pub async fn get_fields_per_loader<'a, E>(
        loader_ids: &[LoaderId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<HashMap<LoaderId, Vec<LoaderField>>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let val = redis.get_cached_keys_raw(
            LOADER_FIELDS_NAMESPACE,
            &loader_ids.iter().map(|x| x.0).collect::<Vec<_>>(),
            |loader_ids| async move {
                let result = sqlx::query!(
                    "
                    SELECT DISTINCT lf.id, lf.field, lf.field_type, lf.optional, lf.min_val, lf.max_val, lf.enum_type, lfl.loader_id
                    FROM loader_fields lf
                    LEFT JOIN loader_fields_loaders lfl ON lfl.loader_field_id = lf.id
                    WHERE lfl.loader_id = ANY($1)
                    ",
                    &loader_ids,
                )
                    .fetch(exec)
                    .try_fold(DashMap::new(), |acc: DashMap<i32, Vec<LoaderField>>, r| {
                        if let Some(field_type) = LoaderFieldType::build(&r.field_type, r.enum_type) {
                            let loader_field = LoaderField {
                                id: LoaderFieldId(r.id),
                                field_type,
                                field: r.field,
                                optional: r.optional,
                                min_val: r.min_val,
                                max_val: r.max_val,
                            };

                            acc.entry(r.loader_id)
                                .or_default()
                                .push(loader_field);
                        }

                        async move {
                            Ok(acc)
                        }
                    })
                    .await?;

                Ok(result)
            },
        ).await?;

        Ok(val.into_iter().map(|x| (LoaderId(x.0), x.1)).collect())
    }

    // Gets all fields for a given loader(s)
    // This is for tags, which need all fields for all loaders
    // We want to return them even in testing situations where we dont have loaders or loader_fields_loaders set up
    pub async fn get_fields_all<'a, E>(
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<LoaderField>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;

        let cached_fields: Option<Vec<LoaderField>> = redis
            .get(LOADER_FIELDS_NAMESPACE_ALL, "")
            .await?
            .and_then(|x| serde_json::from_str::<Vec<LoaderField>>(&x).ok());

        if let Some(cached_fields) = cached_fields {
            return Ok(cached_fields);
        }

        let result = sqlx::query!(
            "
            SELECT DISTINCT lf.id, lf.field, lf.field_type, lf.optional, lf.min_val, lf.max_val, lf.enum_type
            FROM loader_fields lf
            ",
        )
        .fetch(exec)
        .map_ok(|r| {
            Some(LoaderField {
                id: LoaderFieldId(r.id),
                field_type: LoaderFieldType::build(&r.field_type, r.enum_type)?,
                field: r.field,
                optional: r.optional,
                min_val: r.min_val,
                max_val: r.max_val,
            })
        })
        .try_collect::<Vec<Option<LoaderField>>>()
        .await?
            .into_iter()
            .flatten()
            .collect();

        redis
            .set_serialized_to_json(
                LOADER_FIELDS_NAMESPACE_ALL,
                "",
                &result,
                None,
            )
            .await?;

        Ok(result)
    }
}
impl LoaderFieldEnum {
    pub async fn get<'a, E>(
        enum_name: &str, // Note: NOT loader field name
        exec: E,
        redis: &RedisPool,
    ) -> Result<Option<LoaderFieldEnum>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let mut redis = redis.connect().await?;

        let cached_enum = redis
            .get_deserialized_from_json(
                LOADER_FIELD_ENUMS_ID_NAMESPACE,
                enum_name,
            )
            .await?;
        if let Some(cached_enum) = cached_enum {
            return Ok(cached_enum);
        }

        let result = sqlx::query!(
            "
            SELECT lfe.id, lfe.enum_name, lfe.ordering, lfe.hidable
            FROM loader_field_enums lfe
            WHERE lfe.enum_name = $1
            ORDER BY lfe.ordering ASC
            ",
            enum_name
        )
        .fetch_optional(exec)
        .await?
        .map(|l| LoaderFieldEnum {
            id: LoaderFieldEnumId(l.id),
            enum_name: l.enum_name,
            ordering: l.ordering,
            hidable: l.hidable,
        });

        redis
            .set_serialized_to_json(
                LOADER_FIELD_ENUMS_ID_NAMESPACE,
                enum_name,
                &result,
                None,
            )
            .await?;

        Ok(result)
    }
}

impl LoaderFieldEnumValue {
    pub async fn list<'a, E>(
        loader_field_enum_id: LoaderFieldEnumId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<LoaderFieldEnumValue>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Self::list_many(&[loader_field_enum_id], exec, redis)
            .await?
            .into_iter()
            .next()
            .map(|x| x.1)
            .unwrap_or_default())
    }

    pub async fn list_many_loader_fields<'a, E>(
        loader_fields: &[LoaderField],
        exec: E,
        redis: &RedisPool,
    ) -> Result<HashMap<LoaderFieldId, Vec<LoaderFieldEnumValue>>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let get_enum_id = |x: &LoaderField| match x.field_type {
            LoaderFieldType::Enum(id) | LoaderFieldType::ArrayEnum(id) => {
                Some(id)
            }
            _ => None,
        };

        let enum_ids = loader_fields
            .iter()
            .filter_map(get_enum_id)
            .collect::<Vec<_>>();
        let values = Self::list_many(&enum_ids, exec, redis)
            .await?
            .into_iter()
            .collect::<HashMap<_, _>>();

        let mut res = HashMap::new();
        for lf in loader_fields {
            if let Some(id) = get_enum_id(lf) {
                res.insert(
                    lf.id,
                    values.get(&id).unwrap_or(&Vec::new()).to_vec(),
                );
            }
        }
        Ok(res)
    }

    pub async fn list_many<'a, E>(
        loader_field_enum_ids: &[LoaderFieldEnumId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<
        HashMap<LoaderFieldEnumId, Vec<LoaderFieldEnumValue>>,
        DatabaseError,
    >
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let val = redis.get_cached_keys_raw(
            LOADER_FIELD_ENUM_VALUES_NAMESPACE,
            &loader_field_enum_ids.iter().map(|x| x.0).collect::<Vec<_>>(),
            |loader_field_enum_ids| async move {
                let values = sqlx::query!(
                    "
                    SELECT id, enum_id, value, ordering, metadata, created FROM loader_field_enum_values
                    WHERE enum_id = ANY($1)
                    ORDER BY enum_id, ordering, created DESC
                    ",
                    &loader_field_enum_ids
                )
                    .fetch(exec)
                    .try_fold(DashMap::new(), |acc: DashMap<i32, Vec<LoaderFieldEnumValue>>, c| {
                        let value = LoaderFieldEnumValue {
                            id: LoaderFieldEnumValueId(c.id),
                            enum_id: LoaderFieldEnumId(c.enum_id),
                            value: c.value,
                            ordering: c.ordering,
                            created: c.created,
                            metadata: c.metadata.unwrap_or_default(),
                        };

                        acc.entry(c.enum_id)
                            .or_default()
                            .push(value);

                        async move {
                            Ok(acc)
                        }
                    })
                    .await?;

                Ok(values)
            },
        ).await?;

        Ok(val
            .into_iter()
            .map(|x| (LoaderFieldEnumId(x.0), x.1))
            .collect())
    }

    // Matches filter against metadata of enum values
    pub async fn list_filter<'a, E>(
        loader_field_enum_id: LoaderFieldEnumId,
        filter: HashMap<String, serde_json::Value>,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<LoaderFieldEnumValue>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = Self::list(loader_field_enum_id, exec, redis)
            .await?
            .into_iter()
            .filter(|x| {
                let mut bool = true;
                for (key, value) in filter.iter() {
                    if let Some(metadata_value) = x.metadata.get(key) {
                        bool &= metadata_value == value;
                    } else {
                        bool = false;
                    }
                }
                bool
            })
            .collect();

        Ok(result)
    }
}

impl VersionField {
    pub async fn insert_many(
        items: Vec<Self>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let mut query_version_fields = vec![];
        for item in items {
            let base = QueryVersionField {
                version_id: item.version_id,
                field_id: item.field_id,
                int_value: None,
                enum_value: None,
                string_value: None,
            };

            match item.value {
                VersionFieldValue::Integer(i) => {
                    query_version_fields.push(base.clone().with_int_value(i))
                }
                VersionFieldValue::Text(s) => {
                    query_version_fields.push(base.clone().with_string_value(s))
                }
                VersionFieldValue::Boolean(b) => query_version_fields
                    .push(base.clone().with_int_value(if b { 1 } else { 0 })),
                VersionFieldValue::ArrayInteger(v) => {
                    for i in v {
                        query_version_fields
                            .push(base.clone().with_int_value(i));
                    }
                }
                VersionFieldValue::ArrayText(v) => {
                    for s in v {
                        query_version_fields
                            .push(base.clone().with_string_value(s));
                    }
                }
                VersionFieldValue::ArrayBoolean(v) => {
                    for b in v {
                        query_version_fields.push(
                            base.clone().with_int_value(if b { 1 } else { 0 }),
                        );
                    }
                }
                VersionFieldValue::Enum(_, v) => query_version_fields
                    .push(base.clone().with_enum_value(v.id)),
                VersionFieldValue::ArrayEnum(_, v) => {
                    for ev in v {
                        query_version_fields
                            .push(base.clone().with_enum_value(ev.id));
                    }
                }
            };
        }

        let (field_ids, version_ids, int_values, enum_values, string_values): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = query_version_fields
            .iter()
            .map(|l| {
                (
                    l.field_id.0,
                    l.version_id.0,
                    l.int_value,
                    l.enum_value.as_ref().map(|e| e.0).unwrap_or(-1),
                    l.string_value.clone(),
                )
            })
            .multiunzip();

        sqlx::query!(
                "
                INSERT INTO version_fields (field_id, version_id, int_value, string_value, enum_value)
                SELECT * FROM UNNEST($1::integer[], $2::bigint[], $3::integer[], $4::text[], $5::integer[])
                ",
                &field_ids[..],
                &version_ids[..],
                &int_values[..] as &[Option<i32>],
                &string_values[..] as &[Option<String>],
                &enum_values[..] as &[i32]
            )
            .execute(&mut **transaction)
            .await?;

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
            if let Some(min) = loader_field.min_val {
                if count < min {
                    return Err(format!(
                        "Provided value '{v}' for {field_name} is less than the minimum of {min}",
                        v = serde_json::to_string(&value).unwrap_or_default(),
                        field_name = loader_field.field,
                    ));
                }
            }

            if let Some(max) = loader_field.max_val {
                if count > max {
                    return Err(format!(
                        "Provided value '{v}' for {field_name} is greater than the maximum of {max}",
                        v = serde_json::to_string(&value).unwrap_or_default(),
                        field_name = loader_field.field,
                    ));
                }
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
                let loader_field_type = match LoaderFieldType::build(
                    &q.field_type,
                    q.enum_type.map(|l| l.0),
                ) {
                    Some(lft) => lft,
                    None => return vec![],
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
    ) -> Result<VersionField, DatabaseError> {
        let (version_id, value) = VersionFieldValue::build(
            &loader_field.field_type,
            query_version_fields,
            query_loader_field_enum_values,
        )?;
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
    ) -> Result<Vec<VersionField>, DatabaseError> {
        let values = VersionFieldValue::build_many(
            &loader_field.field_type,
            query_version_fields,
            query_loader_field_enum_values,
        )?;
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
    ) -> Result<(DBVersionId, VersionFieldValue), DatabaseError> {
        match field_type {
            LoaderFieldType::Integer
            | LoaderFieldType::Text
            | LoaderFieldType::Boolean
            | LoaderFieldType::Enum(_) => {
                let mut fields = Self::build_many(field_type, qvfs, qlfev)?;
                if fields.len() > 1 {
                    return Err(DatabaseError::SchemaError(format!(
                        "Multiple fields for field {}",
                        field_type.to_str()
                    )));
                }
                fields.pop().ok_or_else(|| {
                    DatabaseError::SchemaError(format!(
                        "No version fields for field {}",
                        field_type.to_str()
                    ))
                })
            }
            LoaderFieldType::ArrayInteger
            | LoaderFieldType::ArrayText
            | LoaderFieldType::ArrayBoolean
            | LoaderFieldType::ArrayEnum(_) => {
                let fields = Self::build_many(field_type, qvfs, qlfev)?;
                Ok(fields.into_iter().next().ok_or_else(|| {
                    DatabaseError::SchemaError(format!(
                        "No version fields for field {}",
                        field_type.to_str()
                    ))
                })?)
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
    ) -> Result<Vec<(DBVersionId, VersionFieldValue)>, DatabaseError> {
        let field_name = field_type.to_str();
        let did_not_exist_error = |field_name: &str, desired_field: &str| {
            DatabaseError::SchemaError(format!(
                "Field name {desired_field} for field {field_name} in does not exist"
            ))
        };

        // Check errors- version_id must all be the same
        let version_id = qvfs
            .iter()
            .map(|qvf| qvf.version_id)
            .unique()
            .collect::<Vec<_>>();
        // If the field type is a non-array, then the reason for multiple version ids is that there are multiple versions being aggregated, and those version ids are contained within.
        // If the field type is an array, then the reason for multiple version ids is that there are multiple values for a single version
        // (or a greater aggregation between multiple arrays, in which case the per-field version is lost, so we just take the first one and use it for that)
        let version_id =
            version_id.into_iter().next().unwrap_or(DBVersionId(0));

        let field_id = qvfs
            .iter()
            .map(|qvf| qvf.field_id)
            .unique()
            .collect::<Vec<_>>();
        if field_id.len() > 1 {
            return Err(DatabaseError::SchemaError(format!(
                "Multiple field ids for field {field_name}"
            )));
        }

        let mut value = match field_type {
            // Singleton fields
            // If there are multiple, we assume multiple versions are being concatenated
            LoaderFieldType::Integer => {
                qvfs.into_iter()
                    .map(|qvf| {
                        Ok((
                            qvf.version_id,
                            VersionFieldValue::Integer(qvf.int_value.ok_or(
                                did_not_exist_error(field_name, "int_value"),
                            )?),
                        ))
                    })
                    .collect::<Result<
                        Vec<(DBVersionId, VersionFieldValue)>,
                        DatabaseError,
                    >>()?
            }
            LoaderFieldType::Text => {
                qvfs.into_iter()
                    .map(|qvf| {
                        Ok((
                            qvf.version_id,
                            VersionFieldValue::Text(qvf.string_value.ok_or(
                                did_not_exist_error(field_name, "string_value"),
                            )?),
                        ))
                    })
                    .collect::<Result<
                        Vec<(DBVersionId, VersionFieldValue)>,
                        DatabaseError,
                    >>()?
            }
            LoaderFieldType::Boolean => {
                qvfs.into_iter()
                    .map(|qvf| {
                        Ok((
                            qvf.version_id,
                            VersionFieldValue::Boolean(
                                qvf.int_value.ok_or(did_not_exist_error(
                                    field_name,
                                    "int_value",
                                ))? != 0,
                            ),
                        ))
                    })
                    .collect::<Result<
                        Vec<(DBVersionId, VersionFieldValue)>,
                        DatabaseError,
                    >>()?
            }
            LoaderFieldType::Enum(id) => {
                qvfs.into_iter()
                    .map(|qvf| {
                        Ok((
                            qvf.version_id,
                            VersionFieldValue::Enum(*id, {
                                let enum_id = qvf.enum_value.ok_or(
                                    did_not_exist_error(
                                        field_name,
                                        "enum_value",
                                    ),
                                )?;
                                let lfev = qlfev
                                    .iter()
                                    .find(|x| x.id == enum_id)
                                    .ok_or(did_not_exist_error(
                                        field_name,
                                        "enum_value",
                                    ))?;
                                LoaderFieldEnumValue {
                                    id: lfev.id,
                                    enum_id: lfev.enum_id,
                                    value: lfev.value.clone(),
                                    ordering: lfev.ordering,
                                    created: lfev.created,
                                    metadata: lfev
                                        .metadata
                                        .clone()
                                        .unwrap_or_default(),
                                }
                            }),
                        ))
                    })
                    .collect::<Result<
                        Vec<(DBVersionId, VersionFieldValue)>,
                        DatabaseError,
                    >>()?
            }

            // Array fields
            // We concatenate into one array
            LoaderFieldType::ArrayInteger => vec![(
                version_id,
                VersionFieldValue::ArrayInteger(
                    qvfs.into_iter()
                        .map(|qvf| {
                            qvf.int_value.ok_or(did_not_exist_error(
                                field_name,
                                "int_value",
                            ))
                        })
                        .collect::<Result<_, _>>()?,
                ),
            )],
            LoaderFieldType::ArrayText => vec![(
                version_id,
                VersionFieldValue::ArrayText(
                    qvfs.into_iter()
                        .map(|qvf| {
                            qvf.string_value.ok_or(did_not_exist_error(
                                field_name,
                                "string_value",
                            ))
                        })
                        .collect::<Result<_, _>>()?,
                ),
            )],
            LoaderFieldType::ArrayBoolean => vec![(
                version_id,
                VersionFieldValue::ArrayBoolean(
                    qvfs.into_iter()
                        .map(|qvf| {
                            Ok::<bool, DatabaseError>(
                                qvf.int_value.ok_or(did_not_exist_error(
                                    field_name,
                                    "int_value",
                                ))? != 0,
                            )
                        })
                        .collect::<Result<_, _>>()?,
                ),
            )],
            LoaderFieldType::ArrayEnum(id) => vec![(
                version_id,
                VersionFieldValue::ArrayEnum(
                    *id,
                    qvfs.into_iter()
                        .map(|qvf| {
                            let enum_id = qvf.enum_value.ok_or(
                                did_not_exist_error(field_name, "enum_value"),
                            )?;
                            let lfev = qlfev
                                .iter()
                                .find(|x| x.id == enum_id)
                                .ok_or(did_not_exist_error(
                                field_name,
                                "enum_value",
                            ))?;
                            Ok::<_, DatabaseError>(LoaderFieldEnumValue {
                                id: lfev.id,
                                enum_id: lfev.enum_id,
                                value: lfev.value.clone(),
                                ordering: lfev.ordering,
                                created: lfev.created,
                                metadata: lfev
                                    .metadata
                                    .clone()
                                    .unwrap_or_default(),
                            })
                        })
                        .collect::<Result<_, _>>()?,
                ),
            )],
        };

        // Sort arrayenums by ordering, then by created
        for (_, v) in value.iter_mut() {
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

    // For conversion to an interanl string(s), such as for search facets, filtering, or direct hardcoding
    // No matter the type, it will be converted to a Vec<String>, whre the non-array types will have a single element
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
            VersionFieldValue::ArrayInteger(v) => value
                .as_i64()
                .map(|i| v.contains(&(i as i32)))
                .unwrap_or(false),
            VersionFieldValue::ArrayText(v) => value
                .as_str()
                .map(|s| v.contains(&s.to_string()))
                .unwrap_or(false),
            VersionFieldValue::ArrayBoolean(v) => {
                value.as_bool().map(|b| v.contains(&b)).unwrap_or(false)
            }
            VersionFieldValue::Enum(_, v) => value.as_str() == Some(&v.value),
            VersionFieldValue::ArrayEnum(_, v) => value
                .as_str()
                .map(|s| v.iter().any(|v| v.value == s))
                .unwrap_or(false),
        }
    }
}
