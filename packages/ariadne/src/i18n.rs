use std::borrow::Cow;

pub use ariadne_macros::*;

pub trait I18nEnum {
    fn translation_id(&self) -> &'static str;

    fn translated_message(&self, locale: &str) -> Cow<'_, str>;
}
