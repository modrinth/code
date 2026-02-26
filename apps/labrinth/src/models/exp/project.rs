use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use validator::Validate;

use crate::{
    database::{
        models::{DBProjectId, DBVersionId},
        redis::RedisPool,
    },
    models::{
        exp::{
            component::{
                self, Component, ComponentEdit, ComponentKind, ComponentQuery,
            },
            minecraft,
        },
        ids::{ProjectId, VersionId},
    },
    queue::{
        analytics::cache::{
            MINECRAFT_SERVER_ANALYTICS, MinecraftServerAnalytics,
        },
        server_ping,
    },
    util::error::Context,
};

pub trait ProjectComponent: Component<EntityId = ProjectId> {
    fn kind() -> ProjectComponentKind;
}

macro_rules! define_project_components {
    (
        $(($field_name:ident, $variant_name:ident): $ty:ty),* $(,)?
    ) => {
        // kinds

        #[expect(dead_code, reason = "static check so $ty implements `ProjectComponent`")]
        const _: () = {
            fn assert_implements_component<T: ProjectComponent>() {}

            fn assert_components_implement_trait() {
                $(assert_implements_component::<$ty>();)*
            }
        };

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum ProjectComponentKind {
            $($variant_name,)*
        }

        impl ComponentKind for ProjectComponentKind {}

        #[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
        pub struct ProjectSerial {
            $(
                #[validate(nested)]
                #[serde(default, skip_serializing_if = "Option::is_none")]
                pub $field_name: Option<$ty>,
            )*
        }

        impl ProjectSerial {
            #[must_use]
            pub fn component_kinds(&self) -> HashSet<ProjectComponentKind> {
                let mut kinds = HashSet::new();
                $(
                    if self.$field_name.is_some() {
                        kinds.insert(ProjectComponentKind::$variant_name);
                    }
                )*
                kinds
            }

            pub fn collect_query_requirements(
                &self,
                project_id: ProjectId,
                requirements: &mut ProjectQueryRequirements,
            ) {
                $(
                    if let Some(component) = &self.$field_name {
                        <<$ty as Component>::Query as ComponentQuery>::collect_requirements(
                            component,
                            project_id,
                            requirements
                        );
                    }
                )*
            }

            pub fn into_query(
                self,
                project_id: ProjectId,
                context: &ProjectQueryContext,
            ) -> Result<ProjectQuery> {
                Ok(ProjectQuery {
                    $(
                        $field_name: match self.$field_name {
                            Some(serial) => {
                                <$ty as Component>::Query::populate(
                                    serial,
                                    project_id,
                                    context,
                                )
                                .map(Some)
                                .wrap_err(concat!("failed to populate `", stringify!($ty), "`"))?
                            }
                            None => None,
                        },
                    )*
                })
            }
        }

        #[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
        pub struct ProjectQuery {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $field_name: Option<Query<$ty>>,
            )*
        }

        #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
        pub struct ProjectEdit {
            $(
                #[validate(nested)]
                #[serde(skip_serializing_if = "Option::is_none", default)]
                pub $field_name: Option<Edit<$ty>>,
            )*
        }

        impl ProjectEdit {
            #[must_use]
            pub fn component_kinds(&self) -> HashSet<ProjectComponentKind> {
                let mut kinds = HashSet::new();
                $(
                    if self.$field_name.is_some() {
                        kinds.insert(ProjectComponentKind::$variant_name);
                    }
                )*
                kinds
            }

            pub fn create(self) -> Result<ProjectSerial> {
                Ok(ProjectSerial {
                    $(
                        $field_name: self
                            .$field_name
                            .map(<<$ty as Component>::Edit as ComponentEdit>::create)
                            .transpose()?,
                    )*
                })
            }
        }
    };
}

// needed because the `utoipa::ToSchema` macro is broken
// when you have a `::` in the type path
type Edit<T> = <T as Component>::Edit;
type Query<T> = <T as Component>::Query;

define_project_components![
    (minecraft_mod, MinecraftMod): minecraft::ModProject,
    (minecraft_server, MinecraftServer): minecraft::ServerProject,
    (minecraft_java_server, MinecraftJavaServer): minecraft::JavaServerProject,
    (minecraft_bedrock_server, MinecraftBedrockServer): minecraft::BedrockServerProject,
];

component::relations! {
    pub static PROJECT_COMPONENT_RELATIONS: ProjectComponentKind = {
        minecraft::PROJECT_COMPONENT_RELATIONS.clone()
    }
}

// query logic

#[derive(Default)]
pub struct ProjectQueryRequirements {
    pub partial_versions: HashSet<VersionId>,
    pub minecraft_java_server_pings: HashSet<ProjectId>,
    pub minecraft_server_analytics: HashSet<ProjectId>,
}

pub struct ProjectQueryContext {
    pub partial_versions: HashMap<VersionId, PartialVersion>,
    pub minecraft_java_server_pings:
        HashMap<ProjectId, minecraft::JavaServerPing>,
    pub minecraft_server_analytics:
        HashMap<ProjectId, MinecraftServerAnalytics>,
}

#[derive(Clone, Debug)]
pub struct PartialVersion {
    pub project_id: ProjectId,
    pub project_name: String,
    pub project_icon: String,
}

pub async fn fetch_query_context(
    projects: &[(ProjectId, &ProjectSerial)],
    db: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    redis: &RedisPool,
) -> Result<ProjectQueryContext> {
    let mut requirements = ProjectQueryRequirements::default();
    for (project_id, project) in projects {
        project.collect_query_requirements(*project_id, &mut requirements);
    }
    let ProjectQueryRequirements {
        partial_versions,
        minecraft_java_server_pings,
        minecraft_server_analytics,
    } = requirements;

    let partial_versions = if partial_versions.is_empty() {
        HashMap::new()
    } else {
        sqlx::query!(
            r#"
            SELECT
                v.id AS "version_id: DBVersionId",
                m.id AS "project_id: DBProjectId",
                m.name AS "project_name!",
                COALESCE(m.icon_url, '') AS "project_icon!"
            FROM versions v
            INNER JOIN mods m ON m.id = v.mod_id
            WHERE v.id = ANY($1)
            "#,
            &partial_versions
                .iter()
                .map(|id| DBVersionId::from(*id).0)
                .collect::<Vec<_>>(),
        )
        .fetch_all(db)
        .await
        .wrap_err("failed to fetch partial versions")?
        .into_iter()
        .map(|row| {
            (
                VersionId::from(row.version_id),
                PartialVersion {
                    project_id: ProjectId::from(row.project_id),
                    project_name: row.project_name,
                    project_icon: row.project_icon,
                },
            )
        })
        .collect::<HashMap<_, _>>()
    };

    let mut redis = redis.connect().await?;

    let minecraft_java_server_pings =
        minecraft_java_server_pings.into_iter().collect::<Vec<_>>();
    let minecraft_java_server_pings = if minecraft_java_server_pings.is_empty()
    {
        HashMap::new()
    } else {
        redis
            .get_many_deserialized_from_json::<minecraft::JavaServerPing>(
                server_ping::REDIS_NAMESPACE,
                &minecraft_java_server_pings
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )
            .await?
            .into_iter()
            .enumerate()
            .filter_map(|(idx, ping)| {
                ping.map(|ping| (minecraft_java_server_pings[idx], ping))
            })
            .collect::<HashMap<_, _>>()
    };

    let minecraft_server_analytics =
        minecraft_server_analytics.into_iter().collect::<Vec<_>>();

    let minecraft_server_analytics = if minecraft_server_analytics.is_empty() {
        HashMap::new()
    } else {
        redis
            .get_many_deserialized_from_json::<MinecraftServerAnalytics>(
                MINECRAFT_SERVER_ANALYTICS,
                &minecraft_server_analytics
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )
            .await?
            .into_iter()
            .enumerate()
            .filter_map(|(idx, data)| {
                data.map(|data| (minecraft_server_analytics[idx], data))
            })
            .collect::<HashMap<_, _>>()
    };

    Ok(ProjectQueryContext {
        partial_versions,
        minecraft_java_server_pings,
        minecraft_server_analytics,
    })
}
