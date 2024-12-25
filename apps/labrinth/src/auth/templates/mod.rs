use crate::auth::AuthenticationError;
use ntex::http::StatusCode;
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use std::fmt::{Debug, Display, Formatter};

pub struct Success<'a> {
    pub icon: &'a str,
    pub name: &'a str,
}

impl Success<'_> {
    pub fn render(self) -> HttpResponse {
        let html = include_str!("success.html");

        HttpResponse::Ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(
                html.replace("{{ icon }}", self.icon)
                    .replace("{{ name }}", self.name),
            )
    }
}

#[derive(Debug)]
pub struct ErrorPage {
    pub code: StatusCode,
    pub message: String,
}

impl Display for ErrorPage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let html = include_str!("error.html")
            .replace("{{ code }}", &self.code.to_string())
            .replace("{{ message }}", &self.message);
        write!(f, "{}", html)?;

        Ok(())
    }
}

impl ErrorPage {
    pub fn render(&self) -> HttpResponse {
        HttpResponse::Ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(self.to_string())
    }
}

impl WebResponseError for ErrorPage {
    fn status_code(&self) -> StatusCode {
        self.code
    }

    fn error_response(&self, _req: &HttpRequest) -> HttpResponse {
        self.render()
    }
}

impl From<AuthenticationError> for ErrorPage {
    fn from(item: AuthenticationError) -> Self {
        ErrorPage {
            code: item.status_code(),
            message: item.to_string(),
        }
    }
}
