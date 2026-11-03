use crate::models::exp::{
    base,
    component::{self, Component, ComponentKind},
    minecraft,
};

use crate::models::exp::component::ComponentQuery;
use crate::models::ids::VersionId;
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use validator::Validate;

pub trait VersionComponent:
    Component<EntityId = crate::models::ids::VersionId>
{
    fn kind() -> VersionComponentKind;
}

macro_rules! define_version_components {
    (
        $(($field_name:ident, $variant_name:ident): $ty:ty),* $(,)?
    ) => {
        // kinds

		#[expect(dead_code, reason = "static check so type implements `VersionComponent`")]
        const _: () = {
			fn assert_implements_component<T: VersionComponent>() {}

            fn assert_components_implement_trait() {
                $(assert_implements_component::<$ty>();)*
            }
        };

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum VersionComponentKind {
            $($variant_name,)*
        }

        impl ComponentKind for VersionComponentKind {}

        // structs

        #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
        pub struct Version {
            #[validate(nested)]
            pub base: base::Version,
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                #[validate(nested)]
                pub $field_name: Option<$ty>,
            )*
        }

        #[derive(Debug, Clone, Default, Serialize, Deserialize)]
        pub struct VersionSerial {
            $(
                pub $field_name: Option<$ty>,
            )*
        }

        impl VersionSerial {
            pub fn collect_query_requirements(
                &self,
                version_id: VersionId,
                requirements: &mut VersionQueryRequirements,
            ) {
                $(
                    if let Some(component) = &self.$field_name {
                        <<$ty as Component>::Query as ComponentQuery>::collect_requirements(
                            component,
                            version_id,
                            requirements,
                        );
                    }
                )*
            }

            pub fn into_query(
                self,
                version_id: VersionId,
                context: &VersionQueryContext,
            ) -> Result<VersionQuery> {
                Ok(VersionQuery {
                    $(
                        $field_name: match self.$field_name {
                            Some(serial) => <$ty as Component>::Query::populate(
                                serial,
                                version_id,
                                context,
                            )
                            .map(Some)
                            .wrap_err(concat!("failed to populate `", stringify!($ty), "`"))?,
                            None => None,
                        },
                    )*
                })
            }
        }

        #[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, utoipa::ToSchema)]
        pub struct VersionCreate {
            #[validate(nested)]
            pub base: Option<base::Project>,
            $(
                #[validate(nested)]
                pub $field_name: Option<$ty>,
            )*
        }

        impl VersionCreate {
            #[must_use]
            pub fn component_kinds(&self) -> HashSet<VersionComponentKind> {
                let mut kinds = HashSet::new();
                $(if self.$field_name.is_some() {
                    kinds.insert(VersionComponentKind::$variant_name);
                })*
                kinds
            }
        }

        #[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
        pub struct VersionQuery {
            $(
                pub $field_name: Option<Query<$ty>>,
            )*
        }

        #[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, utoipa::ToSchema)]
        pub struct VersionEdit {
            $(
                #[validate(nested)]
                pub $field_name: Option<Edit<$ty>>,
            )*
        }
    };
}

// needed because the `utoipa::ToSchema` macro is broken
// when you have a `::` in the type path
type Edit<T> = <T as Component>::Edit;
type Query<T> = <T as Component>::Query;

define_version_components![
    (minecraft_mod, MinecraftMod): minecraft::ModVersion,
    (minecraft_java_server, MinecraftJavaServer): minecraft::JavaServerVersion,
];

#[derive(Default)]
pub struct VersionQueryRequirements {}

pub struct VersionQueryContext {}

pub async fn fetch_query_context(
    versions: &[(VersionId, &VersionSerial)],
    _db: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    _redis: &xredis::RedisPool,
) -> Result<VersionQueryContext> {
    let mut requirements = VersionQueryRequirements::default();
    for (version_id, version) in versions {
        version.collect_query_requirements(*version_id, &mut requirements);
    }
    Ok(VersionQueryContext {})
}

component::relations! {
    pub static VERSION_COMPONENT_RELATIONS: VersionComponentKind = {
        use VersionComponentKind::*;

        [
            [MinecraftMod].only(),
            [MinecraftJavaServer].only(),
        ]
    }
}
