use super::errors::OAuthError;
use crate::auth::oauth::OAuthErrorType;
use crate::database::models::DBOAuthClientId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OAuthRedirectUris {
    pub original: Option<String>,
    pub validated: ValidatedRedirectUri,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatedRedirectUri(pub String);

impl ValidatedRedirectUri {
    pub fn validate<'a>(
        to_validate: &Option<String>,
        validate_against: impl IntoIterator<Item = &'a str> + Clone,
        client_id: DBOAuthClientId,
    ) -> Result<Self, OAuthError> {
        if let Some(first_client_redirect_uri) =
            validate_against.clone().into_iter().next()
        {
            if let Some(to_validate) = to_validate {
                if validate_against.into_iter().any(|uri| {
                    same_uri_except_query_components(uri, to_validate)
                }) {
                    Ok(ValidatedRedirectUri(to_validate.clone()))
                } else {
                    Err(OAuthError::error(
                        OAuthErrorType::RedirectUriNotConfigured(
                            to_validate.clone(),
                        ),
                    ))
                }
            } else {
                Ok(ValidatedRedirectUri(first_client_redirect_uri.to_string()))
            }
        } else {
            Err(OAuthError::error(
                OAuthErrorType::ClientMissingRedirectURI { client_id },
            ))
        }
    }
}

fn same_uri_except_query_components(a: &str, b: &str) -> bool {
    let mut a_components = a.split('?');
    let mut b_components = b.split('?');
    a_components.next() == b_components.next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_for_none_returns_first_valid_uri() {
        let validate_against = vec!["https://modrinth.com/a"];

        let validated = ValidatedRedirectUri::validate(
            &None,
            validate_against.clone(),
            DBOAuthClientId(0),
        )
        .unwrap();

        assert_eq!(validate_against[0], validated.0);
    }

    #[test]
    fn validate_for_valid_uri_returns_first_matching_uri_ignoring_query_params()
    {
        let validate_against = vec![
            "https://modrinth.com/a?q3=p3&q4=p4",
            "https://modrinth.com/a/b/c?q1=p1&q2=p2",
        ];
        let to_validate =
            "https://modrinth.com/a/b/c?query0=param0&query1=param1"
                .to_string();

        let validated = ValidatedRedirectUri::validate(
            &Some(to_validate.clone()),
            validate_against,
            DBOAuthClientId(0),
        )
        .unwrap();

        assert_eq!(to_validate, validated.0);
    }

    #[test]
    fn validate_for_invalid_uri_returns_err() {
        let validate_against = vec!["https://modrinth.com/a"];
        let to_validate = "https://modrinth.com/a/b".to_string();

        let validated = ValidatedRedirectUri::validate(
            &Some(to_validate),
            validate_against,
            DBOAuthClientId(0),
        );

        assert!(validated.is_err_and(|e| matches!(
            e.error_type,
            OAuthErrorType::RedirectUriNotConfigured(_)
        )));
    }
}
