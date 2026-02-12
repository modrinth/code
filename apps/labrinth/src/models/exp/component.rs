macro_rules! define {
    () => {};
    (
        #[component($component_kind:ident :: $component_kind_variant:ident)]
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
            type Serial = Self;
            type Edit = [< $name Edit >];
            type Kind = $component_kind;

            fn kind() -> Self::Kind {
                $component_kind::$component_kind_variant
            }

            fn into_db(self) -> Self::Serial {
                self
            }

            fn from_db(serial: Self::Serial) -> Self {
                serial
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

pub trait Component: Sized {
    type Serial: Serialize + DeserializeOwned;

    type Edit: ComponentEdit<Component = Self>;

    type Kind;

    fn kind() -> Self::Kind;

    fn into_db(self) -> Self::Serial;

    fn from_db(serial: Self::Serial) -> Self;
}

pub trait ComponentEdit: Sized {
    type Component: Component<Edit = Self>;

    fn create(self) -> Result<Self::Component>;

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
    if kinds.is_empty() {
        return Err(ComponentRelationError::NoComponents);
    }

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
