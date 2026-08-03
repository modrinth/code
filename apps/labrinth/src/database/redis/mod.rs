use std::sync::Arc;

use crate::env::ENV;
use eyre::{Result, WrapErr};

struct RedisConfig {
    inner: xredis::RedisConfig,
    cache_settings: xredis::CacheSettings,
}

impl RedisConfig {
    fn from_env() -> Result<Self> {
        let inner = xredis::RedisConfig::new(
            ENV.REDIS_TOPOLOGY,
            ENV.REDIS_CONNECTION_TYPE,
            &ENV.REDIS_URL,
            ENV.REDIS_WAIT_TIMEOUT_MS,
            (
                ENV.REDIS_MAX_CONNECTIONS as usize,
                ENV.REDIS_MIN_CONNECTIONS,
            ),
            (
                ENV.REDIS_CLUSTER_MAX_CONNECTIONS as usize,
                ENV.REDIS_CLUSTER_MIN_CONNECTIONS,
            ),
            (ENV.REDIS_BLOCKING_MAX_CONNECTIONS as usize, 0),
            ENV.REDIS_CACHE_LOCKING_STRATEGY,
            ENV.REDIS_READ_REPLICA_STRATEGY,
        )
        .wrap_err("loading Redis configuration from environment")?;
        let cache_settings = xredis::CacheSettings {
            default_expiry: ENV.REDIS_DEFAULT_EXPIRY,
            actual_expiry: ENV.REDIS_ACTUAL_EXPIRY,
            version_default_expiry: ENV.REDIS_VERSION_DEFAULT_EXPIRY,
            version_actual_expiry: ENV.REDIS_VERSION_ACTUAL_EXPIRY,
            encoding_format: ENV.REDIS_ENCODING_FORMAT,
            compression_algorithm: ENV.REDIS_COMPRESSION_ALGORITHM,
            compression_level: ENV.REDIS_COMPRESSION_LEVEL,
            compression_threshold_bytes: ENV.REDIS_COMPRESSION_THRESHOLD_BYTES,
            compression_min_savings_ratio: ENV
                .REDIS_COMPRESSION_MIN_SAVINGS_RATIO,
        };

        Ok(Self {
            inner,
            cache_settings,
        })
    }
}

pub async fn from_env(
    meta_namespace: impl Into<Arc<str>>,
) -> xredis::RedisPool {
    let config = RedisConfig::from_env().expect("invalid Redis configuration");
    xredis::RedisPool::new(meta_namespace, config.inner, config.cache_settings)
        .await
        .expect("failed to initialize Redis connections")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ::serde::{Serialize, de::DeserializeOwned};
    use chrono::Utc;
    use url::Url;
    use uuid::Uuid;
    use webauthn_rs::WebauthnBuilder;

    use crate::database::models::flow_item::DBFlow;
    use crate::database::models::ids::{
        DBNotificationId, DBProjectId, DBTeamId, DBThreadId, DBUserId,
        DBVersionId, LoaderFieldEnumId, LoaderFieldEnumValueId, LoaderFieldId,
        LoaderId,
    };
    use crate::database::models::loader_fields::{
        Loader, LoaderFieldEnumValue, LoaderMetadata, VersionField,
        VersionFieldValue,
    };
    use crate::database::models::notification_item::DBNotification;
    use crate::database::models::project_item::{
        DBProject, ProjectQueryResult,
    };
    use crate::database::models::version_item::{
        DBVersion, VersionQueryResult,
    };
    use crate::models::billing::{Price, ProductMetadata};
    use crate::models::exp::{self, minecraft};
    use crate::models::notifications::NotificationBody;
    use crate::models::projects::{
        MonetizationStatus, ProjectStatus, SideTypesMigrationReviewStatus,
        VersionStatus,
    };

    fn postcard_round_trip<T>(value: &T) -> T
    where
        T: Serialize + DeserializeOwned,
    {
        let serialized =
            postcard::to_allocvec(value).expect("serializing with postcard");
        postcard::from_bytes(&serialized).expect("deserializing with postcard")
    }

    fn loader_field_enum_value(ty: &str, major: bool) -> LoaderFieldEnumValue {
        LoaderFieldEnumValue {
            id: LoaderFieldEnumValueId(1),
            enum_id: LoaderFieldEnumId(2),
            value: "1.21.8".to_string(),
            ordering: None,
            created: Utc::now(),
            ty: Some(ty.to_string()),
            major: Some(major),
        }
    }

    fn db_project() -> DBProject {
        let now = Utc::now();

        DBProject {
            id: DBProjectId(1),
            team_id: DBTeamId(2),
            organization_id: None,
            name: "project".to_string(),
            summary: "summary".to_string(),
            description: "description".to_string(),
            published: now,
            updated: now,
            approved: Some(now),
            queued: None,
            status: ProjectStatus::Approved,
            requested_status: None,
            downloads: 3,
            follows: 4,
            icon_url: None,
            raw_icon_url: None,
            license_url: None,
            license: "MIT".to_string(),
            slug: Some("project".to_string()),
            moderation_message: None,
            moderation_message_body: None,
            webhook_sent: false,
            color: None,
            monetization_status: MonetizationStatus::Monetized,
            side_types_migration_review_status:
                SideTypesMigrationReviewStatus::Reviewed,
            loaders: vec!["fabric".to_string()],
            components: exp::ProjectSerial::default(),
        }
    }

    fn db_version() -> DBVersion {
        DBVersion {
            id: DBVersionId(1),
            project_id: DBProjectId(2),
            author_id: DBUserId(3),
            name: "version".to_string(),
            version_number: "1.0.0".to_string(),
            changelog: "changelog".to_string(),
            date_published: Utc::now(),
            downloads: 4,
            version_type: "release".to_string(),
            featured: true,
            status: VersionStatus::Listed,
            requested_status: None,
            ordering: None,
            components: exp::VersionSerial::default(),
        }
    }

    #[test]
    fn loader_metadata_round_trips_with_postcard() {
        for platform in [None, Some(false), Some(true)] {
            let metadata = LoaderMetadata { platform };
            let round_tripped = postcard_round_trip(&metadata);

            assert_eq!(round_tripped.platform, platform);
        }
    }

    #[test]
    fn loader_round_trips_with_postcard() {
        for platform in [None, Some(false), Some(true)] {
            let loader = Loader {
                id: LoaderId(1),
                loader: "paper".to_string(),
                icon: "icon".to_string(),
                supported_project_types: vec!["plugin".to_string()],
                supported_games: vec!["minecraft-java".to_string()],
                metadata: LoaderMetadata { platform },
            };
            let round_tripped = postcard_round_trip(&loader);

            assert_eq!(round_tripped.id, loader.id);
            assert_eq!(round_tripped.loader, loader.loader);
            assert_eq!(round_tripped.icon, loader.icon);
            assert_eq!(
                round_tripped.supported_project_types,
                loader.supported_project_types
            );
            assert_eq!(round_tripped.supported_games, loader.supported_games);
            assert_eq!(round_tripped.metadata.platform, platform);
        }
    }

    #[test]
    fn loader_field_enum_values_round_trip_with_postcard() {
        let metadata_values = [
            (None, None),
            (Some("snapshot"), Some(false)),
            (Some("alpha"), Some(false)),
            (Some("beta"), Some(true)),
            (Some("release"), Some(true)),
            (Some("beta"), Some(false)),
            (Some("release"), Some(false)),
        ];

        for (ty, major) in metadata_values {
            let enum_value = LoaderFieldEnumValue {
                id: LoaderFieldEnumValueId(1),
                enum_id: LoaderFieldEnumId(2),
                value: "1.21.8".to_string(),
                ordering: None,
                created: Utc::now(),
                ty: ty.map(str::to_string),
                major,
            };

            assert_eq!(postcard_round_trip(&enum_value), enum_value);
        }
    }

    #[test]
    fn loader_field_enum_value_keeps_flattened_json_layout() {
        let enum_value = loader_field_enum_value("release", true);
        let json = serde_json::to_value(&enum_value)
            .expect("serializing loader field enum value as JSON");

        assert_eq!(json.get("type"), Some(&serde_json::json!("release")));
        assert_eq!(json.get("major"), Some(&serde_json::json!(true)));
        assert!(json.get("metadata").is_none());
        assert_eq!(
            serde_json::from_value::<LoaderFieldEnumValue>(json)
                .expect("deserializing loader field enum value from JSON"),
            enum_value
        );
    }

    #[test]
    fn non_enum_version_fields_round_trip_with_postcard() {
        let values = [
            VersionFieldValue::Integer(5),
            VersionFieldValue::Text("value".to_string()),
            VersionFieldValue::Boolean(true),
            VersionFieldValue::ArrayInteger(vec![1, 2]),
            VersionFieldValue::ArrayText(vec!["one".to_string()]),
            VersionFieldValue::ArrayBoolean(vec![true, false]),
        ];

        for value in values {
            let field = VersionField {
                version_id: DBVersionId(1),
                field_id: LoaderFieldId(2),
                field_name: "field".to_string(),
                value,
            };

            assert_eq!(postcard_round_trip(&field), field);
        }
    }

    #[test]
    fn flattened_cache_values_round_trip_with_postcard() {
        let enum_value = loader_field_enum_value("release", true);
        postcard_round_trip(&enum_value);

        let version = VersionQueryResult {
            inner: db_version(),
            files: Vec::new(),
            version_fields: vec![VersionField {
                version_id: DBVersionId(1),
                field_id: LoaderFieldId(2),
                field_name: "game_versions".to_string(),
                value: VersionFieldValue::Enum(
                    LoaderFieldEnumId(2),
                    enum_value,
                ),
            }],
            loaders: vec!["fabric".to_string()],
            project_types: vec!["mod".to_string()],
            games: vec!["minecraft-java".to_string()],
            dependencies: Vec::new(),
            components: exp::VersionQuery::default(),
        };
        postcard_round_trip(&version);

        let project = ProjectQueryResult {
            inner: db_project(),
            categories: Vec::new(),
            additional_categories: Vec::new(),
            versions: vec![DBVersionId(1)],
            project_types: vec!["mod".to_string()],
            games: vec!["minecraft-java".to_string()],
            urls: Vec::new(),
            gallery_items: Vec::new(),
            thread_id: DBThreadId(1),
            aggregate_version_fields: Vec::new(),
            components: exp::ProjectQuery::default(),
        };
        postcard_round_trip(&project);
    }

    #[test]
    fn skipped_cache_fields_round_trip_with_postcard() {
        postcard_round_trip(&exp::ProjectSerial::default());
        postcard_round_trip(&exp::ProjectQuery::default());
        postcard_round_trip(&db_project());
    }

    #[test]
    fn internally_tagged_cache_values_round_trip_with_postcard() {
        for metadata in [
            ProductMetadata::Midas,
            ProductMetadata::Pyro {
                cpu: 1,
                ram: 2,
                swap: 3,
                storage: 4,
            },
            ProductMetadata::Medal {
                cpu: 1,
                ram: 2,
                swap: 3,
                storage: 4,
                region: "us-east".to_string(),
            },
        ] {
            postcard_round_trip(&metadata);
        }

        for price in [
            Price::OneTime { price: 500 },
            Price::Recurring {
                intervals: HashMap::new(),
            },
        ] {
            postcard_round_trip(&price);
        }

        postcard_round_trip(&NotificationBody::TwoFactorEnabled);
        postcard_round_trip(&DBNotification {
            id: DBNotificationId(1),
            user_id: DBUserId(2),
            body: NotificationBody::TwoFactorEnabled,
            read: false,
            created: Utc::now(),
        });
        postcard_round_trip(&minecraft::ServerContent::Vanilla {
            supported_game_versions: vec!["1.21.8".to_string()],
            recommended_game_version: Some("1.21.8".to_string()),
        });
        postcard_round_trip(&minecraft::ServerContentQuery::Vanilla {
            supported_game_versions: vec!["1.21.8".to_string()],
            recommended_game_version: Some("1.21.8".to_string()),
        });
    }

    #[test]
    fn simple_flow_variants_round_trip_with_postcard() {
        assert!(matches!(
            postcard_round_trip(&DBFlow::MinecraftAuth),
            DBFlow::MinecraftAuth
        ));

        let flow = postcard_round_trip(&DBFlow::Login2FA {
            user_id: DBUserId(1),
        });
        assert!(matches!(
            flow,
            DBFlow::Login2FA {
                user_id: DBUserId(1)
            }
        ));
    }

    #[test]
    fn passkey_flow_variants_round_trip_with_postcard() {
        let origin = Url::parse("https://example.com").unwrap();
        let webauthn = WebauthnBuilder::new("example.com", &origin)
            .unwrap()
            .build()
            .unwrap();
        let (_, registration) = webauthn
            .start_passkey_registration(
                Uuid::from_u128(1),
                "user@example.com",
                "user",
                None,
            )
            .unwrap();
        postcard_round_trip(&DBFlow::RegisterPasskey {
            user_id: DBUserId(1),
            state: registration,
        });

        let (_, authentication) =
            webauthn.start_discoverable_authentication().unwrap();
        postcard_round_trip(&DBFlow::AuthenticatePasskey {
            state: authentication,
        });
    }
}
