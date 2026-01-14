//! Highly experimental and unstable API endpoints.
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

use std::{collections::HashSet, sync::LazyLock};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::Validate;

pub mod base;
pub mod minecraft;

macro_rules! define_project_components {
    (
        $(($field_name:ident, $variant_name:ident): $ty:ty),* $(,)?
    ) => {
        #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
        pub struct ProjectCreate {
            pub base: base::Create,
            $(pub $field_name: Option<$ty>,)*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum ProjectComponentKind {
            $($variant_name,)*
        }

        #[expect(dead_code, reason = "static check so $ty implements `ProjectComponent`")]
        const _: () = {
            fn assert_implements_project_component<T: ProjectComponent>() {}

            fn assert_components_implement_trait() {
                $(assert_implements_project_component::<$ty>();)*
            }
        };

        impl ProjectCreate {
            #[must_use]
            pub fn component_kinds(&self) -> HashSet<ProjectComponentKind> {
                let mut kinds = HashSet::new();
                $(if self.$field_name.is_some() {
                    kinds.insert(ProjectComponentKind::$variant_name);
                })*
                kinds
            }
        }
    };
}

define_project_components! [
    (minecraft_mod, MinecraftMod): minecraft::ModCreate,
    (minecraft_server, MinecraftServer): minecraft::ServerCreate,
    (minecraft_java_server, MinecraftJavaServer): minecraft::JavaServerCreate,
    (minecraft_bedrock_server, MinecraftBedrockServer): minecraft::BedrockServerCreate,
];

pub trait ProjectComponent {
    fn kind() -> ProjectComponentKind;
}

#[derive(Debug, Clone)]
pub enum ComponentRelation {
    /// If one of these components, then it can only be present with other
    /// components from this set.
    Only(HashSet<ProjectComponentKind>),
    /// If component `0` is present, then `1` must also be present.
    Requires(ProjectComponentKind, ProjectComponentKind),
}

trait ComponentKindExt {
    fn requires(self, other: ProjectComponentKind) -> ComponentRelation;
}

impl ComponentKindExt for ProjectComponentKind {
    fn requires(self, other: ProjectComponentKind) -> ComponentRelation {
        ComponentRelation::Requires(self, other)
    }
}

trait ComponentKindArrayExt {
    fn only(self) -> ComponentRelation;
}

impl<const N: usize> ComponentKindArrayExt for [ProjectComponentKind; N] {
    fn only(self) -> ComponentRelation {
        ComponentRelation::Only(self.iter().copied().collect())
    }
}

#[derive(Debug, Clone, Error)]
pub enum ComponentsIncompatibleError {
    #[error(
        "only components {only:?} can be together, found extra components {extra:?}"
    )]
    Only {
        only: HashSet<ProjectComponentKind>,
        extra: HashSet<ProjectComponentKind>,
    },
    #[error("component `{target:?}` requires `{requires:?}`")]
    Requires {
        target: ProjectComponentKind,
        requires: ProjectComponentKind,
    },
}

pub fn component_kinds_compatible(
    kinds: &HashSet<ProjectComponentKind>,
) -> Result<(), ComponentsIncompatibleError> {
    static RELATIONS: LazyLock<Vec<ComponentRelation>> = LazyLock::new(|| {
        let mut relations = Vec::new();
        relations.extend_from_slice(minecraft::RELATIONS.as_slice());
        relations
    });

    for relation in RELATIONS.iter() {
        match relation {
            ComponentRelation::Only(set) => {
                if kinds.iter().any(|k| set.contains(k)) {
                    let extra: HashSet<_> =
                        kinds.difference(set).cloned().collect();
                    if !extra.is_empty() {
                        return Err(ComponentsIncompatibleError::Only {
                            only: set.clone(),
                            extra,
                        });
                    }
                }
            }
            ComponentRelation::Requires(a, b) => {
                if kinds.contains(a) && !kinds.contains(b) {
                    return Err(ComponentsIncompatibleError::Requires {
                        target: *a,
                        requires: *b,
                    });
                }
            }
        }
    }

    Ok(())
}
