use std::borrow::Cow;

#[cfg(feature = "labrinth")]
pub use ariadne_macros::localized_labrinth_error;

pub trait I18nEnum {
    const ROOT_TRANSLATION_ID: &'static str;

    fn translation_id(&self) -> &'static str;

    fn full_translation_id(&self) -> &'static str;

    fn translated_message<'a>(&self, locale: &str) -> Cow<'a, str>;
}

#[macro_export]
macro_rules! i18n_enum {
    (
        $for_enum:ty,
        root_key: $root_key:literal,
        _ => $key:literal,
    ) => {
        #[allow(unused_variables)] // Rust doesn't see the variables from $variant get used for some rason
        impl $crate::i18n::I18nEnum for $for_enum {
            const ROOT_TRANSLATION_ID: &'static str = $root_key;

            fn translation_id(&self) -> &'static str {
                $key
            }

            fn full_translation_id(&self) -> &'static str {
                concat!($root_key, ".", $key)
            }

            fn translated_message<'a>(&self, locale: &str) -> ::std::borrow::Cow<'a, str> {
                ::rust_i18n::t!(concat!($root_key, ".", $key), locale = locale)
            }
        }

        impl ::std::fmt::Display for $for_enum {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(&self.translated_message("en"))
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
                    $($crate::__i18n_enum_variant_parameters_no_store!($variant_name, $variant_pat) => concat!($root_key, ".", $key),)*
                }
            }

            fn translated_message<'a>(&self, locale: &str) -> ::std::borrow::Cow<'a, str> {
                trait __TranslatableEnum {
                    fn __maybe_translate<'a>(&self, locale: &str) -> ::std::borrow::Cow<'a, str>;
                }
                impl<T: $crate::i18n::I18nEnum> __TranslatableEnum for T {
                    fn __maybe_translate<'a>(&self, locale: &str) -> ::std::borrow::Cow<'a, str> {
                        $crate::i18n::I18nEnum::translated_message(self, locale)
                    }
                }
                trait __NonTranslatableValue {
                    fn __maybe_translate<'a>(&self, _locale: &str) -> ::std::borrow::Cow<'a, str>;
                }
                impl<T: ::std::fmt::Display> __NonTranslatableValue for &T {
                    fn __maybe_translate<'a>(&self, _locale: &str) -> ::std::borrow::Cow<'a, str> {
                        ::std::borrow::Cow::Owned(::std::string::ToString::to_string(self))
                    }
                }
                use $for_enum::*;
                match self {
                    $(
                        $crate::__i18n_enum_variant_parameters!($variant_name, $variant_pat) =>
                            $crate::__i18n_enum_variant_values!($root_key, $key, locale, $variant_pat),
                    )*
                }
            }
        }

        impl ::std::fmt::Display for $for_enum {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(&$crate::i18n::I18nEnum::translated_message(self, "en"))
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
    ($root_key:literal, $key:literal, $locale:ident, !) => {
        ::rust_i18n::t!(concat!($root_key, ".", $key), locale = $locale)
    };
    ($root_key:literal, $key:literal, $locale:ident, (..)) => {
        ::rust_i18n::t!(concat!($root_key, ".", $key), locale = $locale)
    };
    ($root_key:literal, $key:literal, $locale:ident, {..}) => {
        ::rust_i18n::t!(concat!($root_key, ".", $key), locale = $locale)
    };
    ($root_key:literal, $key:literal, $locale:ident, ($($field:ident),*)) => {
        ::rust_i18n::t!(concat!($root_key, ".", $key), locale = $locale $(, $field = $field.__maybe_translate($locale))*)
    };
    ($root_key:literal, $key:literal, $locale:ident, {$($field:ident),*}) => {
        ::rust_i18n::t!(concat!($root_key, ".", $key), locale = $locale $(, $field = $field.__maybe_translate($locale))*)
    };
}

#[cfg(test)]
#[doc(hidden)]
pub mod test {
    use super::*;

    pub struct TestingBackend;

    impl rust_i18n::Backend for TestingBackend {
        fn available_locales(&self) -> Vec<&str> {
            vec!["en", "ja"]
        }

        fn translate(&self, locale: &str, key: &str) -> Option<&str> {
            let value = match locale {
                "en" => match key {
                    "unit_translatable.unit" => "Unit Translatable",
                    "base.unit" => "Unit",
                    "base.tuple" => "Tuple: %{value}",
                    "base.translatable_tuple" => "Translatable Tuple: %{unit}",
                    "base.named" => "Named: %{subfield}",
                    _ => panic!("No translation for {key}"),
                },
                "ja" => match key {
                    "unit_translatable.unit" => "この単位は翻訳できる",
                    "base.unit" => "単位",
                    "base.tuple" => "組：　%{value}",
                    "base.translatable_tuple" => {
                        "この組は翻訳できる：　%{unit}"
                    }
                    _ => self.translate("en", key)?,
                },
                _ => panic!("No translations for {locale}"),
            };
            Some(value)
        }
    }

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
    }

    i18n_enum!(
        TestEnum,
        root_key: "base",
        Unit! => "unit",
        Tuple(value) => "tuple",
        TranslatableTuple(unit) => "translatable_tuple",
        Named { subfield } => "named",
    );

    fn assert_i18n_eq(x: impl I18nEnum, lang: &str, should_be: &str) {
        assert_eq!(x.translated_message(lang), should_be);
    }

    #[test]
    fn test_en() {
        assert_i18n_eq(UnitTranslatable, "en", "Unit Translatable");
        assert_i18n_eq(TestEnum::Unit, "en", "Unit");
        assert_i18n_eq(TestEnum::Tuple("hello"), "en", "Tuple: hello");
        assert_i18n_eq(
            TestEnum::TranslatableTuple(UnitTranslatable),
            "en",
            "Translatable Tuple: Unit Translatable",
        );
        assert_i18n_eq(
            TestEnum::Named {
                subfield: "Subfield",
            },
            "en",
            "Named: Subfield",
        );
    }

    #[test]
    fn test_ja() {
        assert_i18n_eq(UnitTranslatable, "ja", "この単位は翻訳できる");
        assert_i18n_eq(TestEnum::Unit, "ja", "単位");
        assert_i18n_eq(TestEnum::Tuple("こんにちは"), "ja", "組：　こんにちは");
        assert_i18n_eq(
            TestEnum::TranslatableTuple(UnitTranslatable),
            "ja",
            "この組は翻訳できる：　この単位は翻訳できる",
        );
    }
}
