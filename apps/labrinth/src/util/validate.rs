use std::sync::LazyLock;

use itertools::Itertools;
use regex::Regex;
use validator::{ValidationErrors, ValidationErrorsKind};

use crate::models::pats::Scopes;

pub static RE_URL_SAFE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^[a-zA-Z0-9!@$()`.+,_"-]*$"#).unwrap());
pub static RE_USERNAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^[a-zA-Z0-9_-]*$"#).unwrap());

//TODO: In order to ensure readability, only the first error is printed, this may need to be expanded on in the future!
pub fn validation_errors_to_string(
    errors: ValidationErrors,
    adder: Option<String>,
) -> String {
    let mut output = String::new();

    let map = errors.into_errors();

    let key_option = map.keys().next();

    if let Some(field) = key_option {
        if let Some(error) = map.get(field) {
            return match error {
                ValidationErrorsKind::Struct(errors) => {
                    validation_errors_to_string(
                        *errors.clone(),
                        Some(format!("of item {field}")),
                    )
                }
                ValidationErrorsKind::List(list) => {
                    if let Some((index, errors)) = list.iter().next() {
                        output.push_str(&validation_errors_to_string(
                            *errors.clone(),
                            Some(format!("of list {field} with index {index}")),
                        ));
                    }

                    output
                }
                ValidationErrorsKind::Field(errors) => {
                    if let Some(error) = errors.first() {
                        if let Some(adder) = adder {
                            output.push_str(&format!(
                                "Field {} {} failed validation with error: {}",
                                field, adder, error.code
                            ));
                        } else {
                            output.push_str(&format!(
                                "Field {} failed validation with error: {}",
                                field, error.code
                            ));
                        }
                    }

                    output
                }
            };
        }
    }

    String::new()
}

pub fn validate_deps(
    values: &[crate::models::projects::Dependency],
) -> Result<(), validator::ValidationError> {
    if values
        .iter()
        .duplicates_by(|x| {
            format!(
                "{}-{}-{}",
                x.version_id.unwrap_or(crate::models::ids::VersionId(0)),
                x.project_id.unwrap_or(crate::models::ids::ProjectId(0)),
                x.file_name.as_deref().unwrap_or_default()
            )
        })
        .next()
        .is_some()
    {
        return Err(validator::ValidationError::new("duplicate dependency"));
    }

    Ok(())
}

pub fn validate_url(value: &str) -> Result<(), validator::ValidationError> {
    let url = url::Url::parse(value)
        .ok()
        .ok_or_else(|| validator::ValidationError::new("invalid URL"))?;

    if url.scheme() != "https" {
        return Err(validator::ValidationError::new("URL must be https"));
    }

    Ok(())
}

pub fn validate_url_hashmap_optional_values(
    values: &std::collections::HashMap<String, Option<String>>,
) -> Result<(), validator::ValidationError> {
    for value in values.values().flatten() {
        validate_url(value)?;
    }

    Ok(())
}

pub fn validate_url_hashmap_values(
    values: &std::collections::HashMap<String, String>,
) -> Result<(), validator::ValidationError> {
    for value in values.values() {
        validate_url(value)?;
    }

    Ok(())
}

pub fn validate_no_restricted_scopes(
    value: &Scopes,
) -> Result<(), validator::ValidationError> {
    if value.is_restricted() {
        return Err(validator::ValidationError::new(
            "Restricted scopes not allowed",
        ));
    }

    Ok(())
}

pub fn validate_name(value: &str) -> Result<(), validator::ValidationError> {
    if value.trim().is_empty() {
        return Err(validator::ValidationError::new(
            "Name cannot contain only whitespace.",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_name_with_valid_input() {
        let result = validate_name("My Test mod");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_name_with_invalid_input_returns_error() {
        let result = validate_name("  ");
        assert!(result.is_err());
    }
}
