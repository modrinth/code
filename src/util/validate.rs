use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use validator::{ValidationErrors, ValidationErrorsKind};

lazy_static! {
    pub static ref RE_URL_SAFE: Regex =
        Regex::new(r#"^[a-zA-Z0-9!@$()`.+,_"-]*$"#).unwrap();
}

//TODO: In order to ensure readability, only the first error is printed, this may need to be expanded on in the future!
pub fn validation_errors_to_string(
    errors: ValidationErrors,
    adder: Option<String>,
) -> String {
    let mut output = String::new();

    let map = errors.into_errors();

    let key_option = map.keys().next().copied();

    if let Some(field) = key_option {
        if let Some(error) = map.get(field) {
            return match error {
                ValidationErrorsKind::Struct(errors) => {
                    validation_errors_to_string(
                        *errors.clone(),
                        Some(format!("of item {}", field)),
                    )
                }
                ValidationErrorsKind::List(list) => {
                    if let Some((index, errors)) = list.iter().next() {
                        output.push_str(&*validation_errors_to_string(
                            *errors.clone(),
                            Some(format!(
                                "of list {} with index {}",
                                index, field
                            )),
                        ));
                    }

                    output
                }
                ValidationErrorsKind::Field(errors) => {
                    if let Some(error) = errors.get(0) {
                        if let Some(adder) = adder {
                            output.push_str(&*format!(
                                "Field {} {} failed validation with error {}",
                                field, adder, error.code
                            ));
                        } else {
                            output.push_str(&*format!(
                                "Field {} failed validation with error {}",
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
                "{}-{}",
                x.version_id
                    .unwrap_or(crate::models::projects::VersionId(0)),
                x.project_id
                    .unwrap_or(crate::models::projects::ProjectId(0))
            )
        })
        .next()
        .is_some()
    {
        return Err(validator::ValidationError::new("duplicate dependency"));
    }

    Ok(())
}
