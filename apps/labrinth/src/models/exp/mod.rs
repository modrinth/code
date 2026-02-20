//! Highly experimental and unstable API endpoint models.
//!
//! These are used for testing new API patterns and exploring future endpoints,
//! which may or may not make it into an official release.
//!
//! # Projects and versions
//!
//! Projects and versions work in an ECS-like architecture, where each project
//! is an entity (project ID), and components can be attached to that project to
//! determine the project's type, like a Minecraft mod, data pack, etc. Project
//! components *may* store extra data (like a server listing which stores the
//! server address), but typically, the version will store this data in *version
//! components*.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use validator::Validate;

pub mod base;
pub mod compat;
pub mod component;
pub mod minecraft;

macro_rules! define_project_components {
    (
        $(($field_name:ident, $variant_name:ident): $ty:ty),* $(,)?
    ) => {
        // kinds

        #[expect(dead_code, reason = "static check so $ty implements `Component`")]
        const _: () = {
            fn assert_implements_component<T>()
            where
                T: component::Component<Kind = ProjectComponentKind>,
            {}

            fn assert_components_implement_trait() {
                $(assert_implements_component::<$ty>();)*
            }
        };

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum ProjectComponentKind {
            $($variant_name,)*
        }

        impl component::ComponentKind for ProjectComponentKind {}

        // structs

        #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
        pub struct Project {
            #[validate(nested)]
            pub base: base::Project,
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                #[validate(nested)]
                pub $field_name: Option<$ty>,
            )*
        }

        #[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
        pub struct ProjectSerial {
            $(
                #[validate(nested)]
                pub $field_name: Option<<$ty as $crate::models::exp::component::Component>::Serial>,
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
        }

        #[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, utoipa::ToSchema)]
        pub struct ProjectCreate {
            #[validate(nested)]
            pub base: Option<base::Project>,
            $(
                #[validate(nested)]
                pub $field_name: Option<$ty>,
            )*
        }

        impl ProjectCreate {
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
        }

        #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
        // #[derive(utoipa::ToSchema)]
        pub struct ProjectEdit {
            $(
                #[validate(nested)]
                pub $field_name: Option<<$ty as $crate::models::exp::component::Component>::Edit>,
            )*
        }

        // logic

        impl ProjectCreate {
            pub fn into_db(self) -> ProjectSerial {
                ProjectSerial {
                    $(
                        $field_name: self.$field_name.map(component::Component::into_db),
                    )*
                }
            }
        }
    };
}

macro_rules! define_version_components {
    (
        $(($field_name:ident, $variant_name:ident): $ty:ty),* $(,)?
    ) => {
        // kinds

        #[expect(dead_code, reason = "static check so $ty implements `Component`")]
        const _: () = {
            fn assert_implements_component<T>()
            where
                T: component::Component<Kind = VersionComponentKind>,
            {}

            fn assert_components_implement_trait() {
                $(assert_implements_component::<$ty>();)*
            }
        };

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum VersionComponentKind {
            $($variant_name,)*
        }

        impl component::ComponentKind for VersionComponentKind {}

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
                pub $field_name: Option<<$ty as $crate::models::exp::component::Component>::Serial>,
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

        #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
        // #[derive(utoipa::ToSchema)]
        pub struct VersionEdit {
            $(
                #[validate(nested)]
                pub $field_name: Option<<$ty as $crate::models::exp::component::Component>::Edit>,
            )*
        }

        // logic

        impl VersionCreate {
            pub fn into_db(self) -> VersionSerial {
                VersionSerial {
                    $(
                        $field_name: self.$field_name.map(component::Component::into_db),
                    )*
                }
            }
        }
    };
}

define_project_components![
    (minecraft_mod, MinecraftMod): minecraft::ModProject,
    (minecraft_server, MinecraftServer): minecraft::ServerProject,
    (minecraft_java_server, MinecraftJavaServer): minecraft::JavaServerProject,
    (minecraft_bedrock_server, MinecraftBedrockServer): minecraft::BedrockServerProject,
];

define_version_components![
    (minecraft_java_server, MinecraftJavaServer): minecraft::JavaServerVersion,
];

component::relations! {
    pub static PROJECT_COMPONENT_RELATIONS: ProjectComponentKind = {
        minecraft::PROJECT_COMPONENT_RELATIONS.clone()
    }
}
