use crate::models::exp::{
    base,
    component::{Component, ComponentKind},
    minecraft,
};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use validator::Validate;

macro_rules! define_version_components {
    (
        $(($field_name:ident, $variant_name:ident): $ty:ty),* $(,)?
    ) => {
        // kinds

        #[expect(dead_code, reason = "static check so $ty implements `Component`")]
        const _: () = {
            fn assert_implements_component<T: Component>() {}

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
    (minecraft_java_server, MinecraftJavaServer): minecraft::JavaServerVersion,
];
