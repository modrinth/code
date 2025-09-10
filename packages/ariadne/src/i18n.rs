use serde::Serialize;
use std::collections::HashMap;

pub trait I18nEnum {
    const ROOT_TRANSLATION_ID: &'static str;

    fn translation_id(&self) -> &'static str;

    fn full_translation_id(&self) -> &'static str;

    fn translation_data(&self) -> TranslationData;
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum TranslationData {
    Literal(String),
    Translatable {
        key: &'static str,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        values: HashMap<&'static str, TranslationData>,
    },
}

#[macro_export]
macro_rules! i18n_enum {
    (transparent $for_enum:ty[$field:ident: $field_type:ty]) => {
        impl $crate::i18n::I18nEnum for $for_enum {
            const ROOT_TRANSLATION_ID: &'static str = <$field_type as $crate::i18n::I18nEnum>::ROOT_TRANSLATION_ID;

            fn translation_id(&self) -> &'static str {
                $crate::i18n::I18nEnum::translation_id(&*self.$field)
            }

            fn full_translation_id(&self) -> &'static str {
                $crate::i18n::I18nEnum::full_translation_id(&*self.$field)
            }

            fn translation_data(&self) -> $crate::i18n::TranslationData {
                $crate::i18n::I18nEnum::translation_data(&*self.$field)
            }
        }
    };

    (
        $for_enum:ty,
        root_key: $root_key:literal,
        _ => $key:literal,
    ) => {
        impl $crate::i18n::I18nEnum for $for_enum {
            const ROOT_TRANSLATION_ID: &'static str = $root_key;

            fn translation_id(&self) -> &'static str {
                $key
            }

            fn full_translation_id(&self) -> &'static str {
                ::core::concat!($root_key, ".", $key)
            }

            fn translation_data(&self) -> $crate::i18n::TranslationData {
                $crate::__i18n_enum_variant_values!($root_key, $key, !)
            }
        }
    };

    (
        $for_enum:ty,
        root_key: $root_key:literal,
        $($variant_name:ident$variant_pat:tt => $key:literal,)*
    ) => {
        impl $crate::i18n::I18nEnum for $for_enum {
            const ROOT_TRANSLATION_ID: &'static str = $root_key;

            fn translation_id(&self) -> &'static str {
                use $for_enum::*;
                match self {
                    $($crate::__i18n_enum_variant_parameters_no_store!($variant_name, $variant_pat) => $key,)*
                }
            }

            fn full_translation_id(&self) -> &'static str {
                use $for_enum::*;
                match self {
                    $($crate::__i18n_enum_variant_parameters_no_store!($variant_name, $variant_pat) => ::core::concat!($root_key, ".", $key),)*
                }
            }

            fn translation_data(&self) -> $crate::i18n::TranslationData {
                trait __TranslatableEnum {
                    fn __maybe_translate(&self) -> $crate::i18n::TranslationData;
                }
                impl<T: $crate::i18n::I18nEnum> __TranslatableEnum for T {
                    fn __maybe_translate(&self) -> $crate::i18n::TranslationData {
                        $crate::i18n::I18nEnum::translation_data(self)
                    }
                }
                trait __NonTranslatableValue {
                    fn __maybe_translate(&self) -> $crate::i18n::TranslationData;
                }
                impl<T: ::std::fmt::Display> __NonTranslatableValue for &T {
                    fn __maybe_translate(&self) -> $crate::i18n::TranslationData {
                        $crate::i18n::TranslationData::Literal(::std::string::ToString::to_string(self))
                    }
                }
                use $for_enum::*;
                match self {
                    $(
                        $crate::__i18n_enum_variant_parameters!($variant_name, $variant_pat) =>
                            $crate::__i18n_enum_variant_values!($root_key, $key, $variant_pat),
                    )*
                }
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __i18n_enum_variant_parameters_no_store {
    ($variant_name:ident, !) => {
        $variant_name
    };
    ($variant_name:ident, (transparent $_:ident)) => {
        $variant_name(_)
    };
    ($variant_name:ident, ($($_:tt)+)) => {
        $variant_name(..)
    };
    ($variant_name:ident, {$($_:tt)+}) => {
        $variant_name { .. }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __i18n_enum_variant_parameters {
    ($variant_name:ident, !) => {
        $variant_name
    };
    ($variant_name:ident, (transparent $field:ident)) => {
        $variant_name($field)
    };
    ($variant_name:ident, ($($field:tt)+)) => {
        $variant_name($($field)+)
    };
    ($variant_name:ident, {$($field:tt)+}) => {
        $variant_name { $($field)+ }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __i18n_enum_variant_values {
    ($root_key:literal, $key:literal, !) => {
        $crate::i18n::TranslationData::Translatable {
            key: ::core::concat!($root_key, ".", $key),
            values: ::std::collections::HashMap::new(),
        }
    };
    ($root_key:literal, $key:literal, (..)) => {
        $crate::__i18n_enum_variant_values!($root_key, $key, !)
    };
    ($root_key:literal, $key:literal, {..}) => {
        $crate::__i18n_enum_variant_values!($root_key, $key, !)
    };
    ($root_key:literal, $key:literal, (transparent $field:ident)) => {
        $field.__maybe_translate()
    };
    ($root_key:literal, $key:literal, ($($field:ident),*)) => {
        $crate::i18n::TranslationData::Translatable {
            key: ::core::concat!($root_key, ".", $key),
            values: ::std::collections::HashMap::from([
                $((::core::stringify!($field), $field.__maybe_translate()),)*
            ]),
        }
    };
    ($root_key:literal, $key:literal, {$($field:ident),*}) => {
        $crate::__i18n_enum_variant_values!($root_key, $key, ($($field),*))
    };
}

#[cfg(test)]
#[doc(hidden)]
pub mod test {
    use super::*;
    use serde_json::json;

    struct UnitTranslatable;

    i18n_enum!(
        UnitTranslatable,
        root_key: "unit_translatable",
        _ => "unit",
    );

    enum TestEnum {
        Unit,
        Tuple(&'static str),
        TranslatableTuple(UnitTranslatable),
        Named { subfield: &'static str },
        DirectUnit(UnitTranslatable),
    }

    i18n_enum!(
        TestEnum,
        root_key: "base",
        Unit! => "unit",
        Tuple(value) => "tuple",
        TranslatableTuple(unit) => "translatable_tuple",
        Named { subfield } => "named",
        DirectUnit(transparent unit) => "direct_unit",
    );

    fn assert_i18n_eq(x: impl I18nEnum, should_be: serde_json::Value) {
        assert_eq!(
            serde_json::to_value(x.translation_data()).unwrap(),
            should_be
        );
    }

    #[test]
    fn test() {
        assert_i18n_eq(
            UnitTranslatable,
            json!({
                "key": "unit_translatable.unit",
            }),
        );
        assert_i18n_eq(
            TestEnum::Unit,
            json!({
                "key": "base.unit",
            }),
        );
        assert_i18n_eq(
            TestEnum::Tuple("hello"),
            json!({
                "key": "base.tuple",
                "values": {
                    "value": "hello",
                },
            }),
        );
        assert_i18n_eq(
            TestEnum::TranslatableTuple(UnitTranslatable),
            json!({
                "key": "base.translatable_tuple",
                "values": {
                    "unit": {
                        "key": "unit_translatable.unit",
                    },
                },
            }),
        );
        assert_i18n_eq(
            TestEnum::Named {
                subfield: "Subfield",
            },
            json!({
                "key": "base.named",
                "values": {
                    "subfield": "Subfield",
                }
            }),
        );
        assert_i18n_eq(
            TestEnum::DirectUnit(UnitTranslatable),
            json!({
                "key": "unit_translatable.unit",
            }),
        )
    }
}
