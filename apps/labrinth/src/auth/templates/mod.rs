use crate::auth::AuthenticationError;
use crate::env::ENV;
use actix_web::http::StatusCode;
use actix_web::http::header::LOCATION;
use actix_web::{HttpResponse, ResponseError};
use std::fmt::{Debug, Display, Formatter};

pub struct Success<'a> {
    pub icon: &'a str,
    pub name: &'a str,
}

impl Success<'_> {
    pub fn render(self) -> HttpResponse {
        let html = include_str!("success.html");

        HttpResponse::Ok()
            .append_header(("Content-Type", "text/html; charset=utf-8"))
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
    pub redirect_url: Option<String>,
}

impl Display for ErrorPage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ErrorPage {
    pub fn from_auth_error(
        error: AuthenticationError,
        redirect_url: Option<String>,
    ) -> Self {
        ErrorPage {
            code: error.status_code(),
            message: error.to_string(),
            redirect_url,
        }
    }

    pub fn render(&self) -> HttpResponse {
        let mut redirect_url = format!(
            "{}/auth/error?statusCode={}&message={}",
            ENV.SITE_URL.trim_end_matches('/'),
            self.code.as_u16(),
            urlencoding::encode(&self.message),
        );

        if let Some(url) = &self.redirect_url {
            redirect_url.push_str("&redirect=");
            redirect_url.push_str(&urlencoding::encode(url));
        }

        HttpResponse::TemporaryRedirect()
            .append_header((LOCATION, redirect_url))
            .finish()
    }
}

impl actix_web::ResponseError for ErrorPage {
    fn status_code(&self) -> StatusCode {
        self.code
    }

    fn error_response(&self) -> HttpResponse {
        self.render()
    }
}

impl From<AuthenticationError> for ErrorPage {
    fn from(item: AuthenticationError) -> Self {
        ErrorPage {
            code: item.status_code(),
            message: item.to_string(),
            redirect_url: None,
        }
    }
}
