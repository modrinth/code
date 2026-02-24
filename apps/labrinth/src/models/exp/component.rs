macro_rules! define {
    () => {};
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                #[base(
                    $($field_base_meta:meta),*
                )]
                #[edit(
                    $($field_edit_meta:meta),*
                )]
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident: $field_ty:ty
            ),* $(,)?
        }

        $($rest:tt)*
    ) => { paste::paste! {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $(#[$field_base_meta])*
                $field_vis $field: $field_ty,
            )*
        }

        $(#[$meta])*
        $vis struct [< $name Edit >] {
            $(
                $(#[$field_meta])*
                $(#[$field_edit_meta])*
                $field_vis $field: Option<$field_ty>,
            )*
        }

        impl $crate::models::exp::component::Component for $name {
            type EntityId = $crate::models::ids::ProjectId;
            type Query = $name;
            type Edit = [< $name Edit >];
        }

        impl $crate::models::exp::component::ComponentQuery for $name {
            type Component = $name;
            type Context = $crate::models::exp::project::ProjectQueryContext;
            type Requirements = $crate::models::exp::project::ProjectQueryRequirements;

            fn collect_requirements(
                _serial: &Self::Component,
                _entity_id: <Self::Component as Component>::EntityId,
                _requirements: &mut Self::Requirements,
            ) {}

            fn populate(
                serial: Self::Component,
                _entity_id: <Self::Component as Component>::EntityId,
                _context: &Self::Context,
            ) -> Result<Self> {
                Ok(serial)
            }
        }

        impl $crate::models::exp::component::ComponentEdit for [< $name Edit >] {
            type Component = $name;

            fn create(self) -> eyre::Result<Self::Component> {
                Ok($name {
                    $(
                        $field: eyre::OptionExt::ok_or_eyre(
                            self.$field,
                            concat!("missing field `", stringify!($field), "`")
                        )?,
                    )*
                })
            }

            async fn apply_to(
                self,
                #[allow(unused_variables)]
                component: &mut Self::Component,
            ) -> eyre::Result<()> {
                $(
                    if let Some(f) = self.$field {
                        component.$field = f;
                    }
                )*
                Ok(())
            }
        }

        $crate::models::exp::component::define!($($rest)*);
    }};
}

macro_rules! relations {
    ($vis:vis static $name:ident: $component_kind:ty = $expr:block) => {
        $vis static $name: std::sync::LazyLock<Vec<$crate::models::exp::component::ComponentRelation<$component_kind>>> = std::sync::LazyLock::new(|| {
            #[allow(unused_imports)]
            use $crate::models::exp::component::{ComponentKindExt, ComponentKindArrayExt};

            Vec::<$crate::models::exp::component::ComponentRelation<$component_kind>>::from($expr)
        });
    };
}

pub(crate) use define;
use eyre::Result;
pub(crate) use relations;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{collections::HashSet, hash::Hash};
use thiserror::Error;

pub trait ComponentKind:
    Clone + Send + Sync + PartialEq + Eq + Hash + 'static
{
}

/// Data attached to an entity (like a project or a version), comparable to a
/// component in the [ECS paradigm](https://en.wikipedia.org/wiki/Entity_component_system).
///
/// The struct that implements this trait is the *serial form* of the
/// component, as stored in the database. When it is queried or edited, the
/// schema may take a different form - see [`Component::Query`],
/// [`Component::Edit`].
pub trait Component: Sized + Serialize + DeserializeOwned {
    /// Type of ID that entities which have this type of component use to
    /// identify themselves.
    ///
    /// - For project components, this is [`ProjectId`].
    ///
    /// [`ProjectId`]: crate::models::ids::ProjectId
    type EntityId: Clone + Copy + Eq + Hash + Send + Sync;

    /// Schema of the data returned when querying a component of this type from
    /// the backend.
    ///
    /// See [`ComponentQuery`].
    type Query: ComponentQuery<Component = Self>;

    /// Schema of a modification that can be applied to an existing component of
    /// this type.
    ///
    /// See [`ComponentEdit`].
    type Edit: ComponentEdit<Component = Self>;
}

/// Schema of the data returned when querying a component of type
/// [`Self::Component`] from the backend.
///
/// The [`Component`] stores persistent, serialized data; but when we
/// request a project, we also request its components, and we may want to
/// request extra data alongside the serialized form. For example, if our
/// component stores a project ID to another project, we may want to return
/// that project's name, icon, etc. alongside the ID. [`Component::Query`]
/// provides a way to populate this extra data.
pub trait ComponentQuery: Sized {
    /// Type of serial component this [`ComponentQuery`] is queried from.
    type Component: Component<Query = Self>;

    /// Type of the whole set of information that a query requests from the
    /// database.
    ///
    /// - For project components, this is [`ProjectQueryRequirements`].
    ///
    /// [`ProjectQueryRequirements`]: crate::models::exp::project::ProjectQueryRequirements
    type Requirements;

    /// Type of context provided during [`ComponentQuery::populate`].
    ///
    /// - For project components, this is [`ProjectQueryContext`].
    ///
    /// [`ProjectQueryContext`]: crate::models::exp::project::ProjectQueryContext
    type Context;

    /// What information does this query type require from the database to
    /// populate itself (excluding the [`ComponentQuery::Component`])?
    ///
    /// For example, if the [`ComponentQuery::Component`] has a projecet ID,
    /// this will add the project ID to `requirements`. This will require the
    /// fetcher to also fetch this project ID, which will be available in the
    /// [`ComponentQuery::Context`] during [`ComponentQuery::populate`].
    fn collect_requirements(
        serial: &Self::Component,
        entity_id: <Self::Component as Component>::EntityId,
        requirements: &mut Self::Requirements,
    );

    /// Creates the final component with all queried data, using the serialized
    /// form of the component ([`ComponentQuery::Component`]) and any additional
    /// info requested in [`ComponentQuery::collect_requirements`]
    ///
    /// # Errors
    ///
    /// Errors if some required data in the `context` is missing, indicating a
    /// logic bug.
    fn populate(
        serial: Self::Component,
        entity_id: <Self::Component as Component>::EntityId,
        context: &Self::Context,
    ) -> Result<Self>;
}

/// Schema of a modification to an existing component, or potentially creation
/// of a component.
///
/// The [`Component`] stores persistent, serialized data; but when we want to
/// edit only specific fields of an existing component, we have to be able to
/// exclude fields which are not edited by wrapping the field in an [`Option`].
/// This trait provides a schema for doing this.
pub trait ComponentEdit: Sized {
    /// Type of serial component this [`ComponentQuery`] is a modification for.
    type Component: Component<Edit = Self>;

    /// Attempts to create a [`ComponentEdit::Component`] if this edit has all
    /// of the appropriate fields set.
    ///
    /// # Errors
    ///
    /// Errors if a required field is missing.
    fn create(self) -> Result<Self::Component>;

    /// Applies this edit to an existing component.
    ///
    /// Errors if an edit could not be applied.
    // note: this is `async` because in the future this might issue db/sqlx queries
    #[expect(async_fn_in_trait, reason = "internal trait")]
    async fn apply_to(self, component: &mut Self::Component) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum ComponentRelation<K> {
    /// If one of these components is present, then it can only be present with
    /// other components from this set.
    Only(HashSet<K>),
    /// If component `0` is present, then `1` must also be present.
    Requires(K, K),
}

pub trait ComponentKindExt<K> {
    fn requires(self, other: K) -> ComponentRelation<K>;
}

impl<K> ComponentKindExt<K> for K {
    fn requires(self, other: K) -> ComponentRelation<K> {
        ComponentRelation::Requires(self, other)
    }
}

pub trait ComponentKindArrayExt<K> {
    fn only(self) -> ComponentRelation<K>;
}

impl<K: ComponentKind, const N: usize> ComponentKindArrayExt<K> for [K; N] {
    fn only(self) -> ComponentRelation<K> {
        ComponentRelation::Only(self.iter().cloned().collect())
    }
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ComponentRelationError<K: ComponentKind> {
    #[error("no components")]
    NoComponents,
    #[error("component `{target:?}` is missing")]
    Missing { target: K },
    #[error(
        "only components {only:?} can be together, found extra components {extra:?}"
    )]
    Only { only: HashSet<K>, extra: HashSet<K> },
    #[error("component `{target:?}` requires `{requires:?}`")]
    Requires { target: K, requires: K },
}

pub fn kinds_valid<K: ComponentKind>(
    kinds: &HashSet<K>,
    relations: &[ComponentRelation<K>],
) -> Result<(), ComponentRelationError<K>> {
    for relation in relations {
        match relation {
            ComponentRelation::Only(set) => {
                if kinds.iter().any(|k| set.contains(k)) {
                    let extra: HashSet<_> =
                        kinds.difference(set).cloned().collect();
                    if !extra.is_empty() {
                        return Err(ComponentRelationError::Only {
                            only: set.clone(),
                            extra,
                        });
                    }
                }
            }
            ComponentRelation::Requires(a, b) => {
                if kinds.contains(a) && !kinds.contains(b) {
                    return Err(ComponentRelationError::Requires {
                        target: a.clone(),
                        requires: b.clone(),
                    });
                }
            }
        }
    }

    Ok(())
}
