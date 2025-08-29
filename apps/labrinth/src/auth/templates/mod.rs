use crate::auth::AuthenticationError;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, ResponseError};
use std::fmt::{Debug, Display, Formatter};
use actix_web::http::header::{AcceptLanguage, ContentLanguage, LanguageTag, Header, QualityItem};
use ariadne::i18n::I18nEnum;

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
    pub language: LanguageTag,
}

impl Display for ErrorPage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let html = include_str!("error.html")
            .replace("{{ code }}", &self.code.to_string())
            .replace("{{ message }}", &self.message);
        write!(f, "{html}")?;

        Ok(())
    }
}

impl ErrorPage {
    pub fn new(error: AuthenticationError, req: &HttpRequest) -> Self {
        let language = AcceptLanguage::parse(req)
            .ok()
            .and_then(|x| x.preference().into_item())
            .unwrap_or_else(|| LanguageTag::parse("en").unwrap());
        let message = error.translated_message(language.as_str());
        Self {
            code: error.status_code(),
            message: message.into_owned(),
            language
        }
    }

    pub fn render(&self) -> HttpResponse {
        HttpResponse::Ok()
            .append_header(("Content-Type", "text/html; charset=utf-8"))
            .append_header(ContentLanguage(vec![QualityItem::max(self.language.to_owned())]))
            .body(self.to_string())
    }
}

impl ResponseError for ErrorPage {
    fn status_code(&self) -> StatusCode {
        self.code
    }

    fn error_response(&self) -> HttpResponse {
        self.render()
    }
}
