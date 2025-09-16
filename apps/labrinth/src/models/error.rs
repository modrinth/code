use ariadne::i18n::{I18nEnum, TranslationData};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// An error returned by the API
#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub error: &'static str,
    pub description: String,
    pub translatable_error: TranslationData,
}

pub trait AsApiError {
    fn as_api_error(&self) -> ApiError;
}

impl<T> AsApiError for T
where
    T: I18nEnum + Display,
{
    fn as_api_error(&self) -> ApiError {
        let translation_id = self.translation_id();
        ApiError {
            error: translation_id
                .split_once('.')
                .map_or(translation_id, |(base, _)| base),
            description: self.to_string(),
            translatable_error: self.translation_data(),
        }
    }
}
