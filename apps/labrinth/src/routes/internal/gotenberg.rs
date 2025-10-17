use actix_web::{
    HttpMessage, HttpResponse, error::ParseError, http::header, post, web,
};
use serde::Deserialize;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(success).service(error);
}

struct GotenbergTrace(String);

impl header::TryIntoHeaderValue for GotenbergTrace {
    type Error = header::InvalidHeaderValue;

    fn try_into_value(self) -> Result<header::HeaderValue, Self::Error> {
        header::HeaderValue::from_str(&self.0)
    }
}

impl header::Header for GotenbergTrace {
    fn name() -> header::HeaderName {
        header::HeaderName::from_static("gotenberg-trace")
    }

    fn parse<M: HttpMessage>(m: &M) -> Result<Self, ParseError> {
        m.headers()
            .get(Self::name())
            .ok_or(ParseError::Header)?
            .to_str()
            .map_err(|_| ParseError::Header)
            .map(ToOwned::to_owned)
            .map(GotenbergTrace)
    }
}

#[post("/gotenberg/success")]
pub async fn success(
    _content_disposition: web::Header<header::ContentDisposition>,
    _trace: web::Header<GotenbergTrace>,
    _body: web::Bytes,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ErrorBody {
    status: Option<String>,
    message: Option<String>,
}

#[post("/gotenberg/error")]
pub async fn error(
    _trace: web::Header<GotenbergTrace>,
    _body: web::Json<ErrorBody>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}
