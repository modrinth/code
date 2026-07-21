use serde::Deserialize;

pub const REQUEST_CONTEXT_HEADER: &str = "Modrinth-App-Request-Context";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InvocationContext {
    pub cause: String,
}

impl InvocationContext {
    pub fn new(cause: impl Into<String>) -> Self {
        Self {
            cause: cause.into(),
        }
    }

    pub fn cause(&self) -> &str {
        &self.cause
    }

    pub fn request_context_header(&self) -> &str {
        &self.cause
    }

    pub fn referer(&self) -> String {
        format!("https://tauri.modrinth.app/_rc/{}", self.cause)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_frontend_defined_causes() {
        let context: InvocationContext =
            serde_json::from_str(r#"{"cause":"new/frontend/cause"}"#).unwrap();

        assert_eq!(context.cause(), "new/frontend/cause");
    }

    #[test]
    fn invocation_context_contains_only_a_cause() {
        assert!(
            serde_json::from_str::<InvocationContext>(
                r#"{"cause":"navigation/home","id":"frontend-id"}"#
            )
            .is_err()
        );
    }

    #[test]
    fn headers_use_the_frontend_cause_unchanged() {
        let context = InvocationContext::new("navigation/instance/content");

        assert_eq!(
            context.referer(),
            "https://tauri.modrinth.app/_rc/navigation/instance/content"
        );
        assert_eq!(
            context.request_context_header(),
            "navigation/instance/content"
        );
    }
}
