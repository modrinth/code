use actix_web::{
    HttpMessage, HttpResponse, error::ParseError, http::header, post, web,
};
use serde::Deserialize;
use tracing::trace;

use crate::routes::ApiError;
use crate::util::gotenberg::{
    GeneratedPdfType, MODRINTH_GENERATED_PDF_TYPE, MODRINTH_PAYMENT_ID,
};
use crate::util::guards::internal_network_guard;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(success).service(error);
}

#[post("/gotenberg/success", guard = "internal_network_guard")]
pub async fn success(
    web::Header(header::ContentDisposition {
        disposition,
        parameters: disposition_parameters,
    }): web::Header<header::ContentDisposition>,
    web::Header(GotenbergTrace(trace)): web::Header<GotenbergTrace>,
    web::Header(ModrinthGeneratedPdfType(r#type)): web::Header<
        ModrinthGeneratedPdfType,
    >,
    maybe_payment_id: Option<web::Header<ModrinthPaymentId>>,
    body: web::Bytes,
) -> Result<HttpResponse, ApiError> {
    trace!(
        %trace,
        %disposition,
        ?disposition_parameters,
        r#type = r#type.as_str(),
        ?maybe_payment_id,
        body.len = body.len(),
        "Received Gotenberg generated PDF"
    );

    Ok(HttpResponse::Ok().finish())
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ErrorBody {
    status: Option<String>,
    message: Option<String>,
}

#[post("/gotenberg/error", guard = "internal_network_guard")]
pub async fn error(
    web::Header(GotenbergTrace(trace)): web::Header<GotenbergTrace>,
    web::Header(ModrinthGeneratedPdfType(r#type)): web::Header<
        ModrinthGeneratedPdfType,
    >,
    maybe_payment_id: Option<web::Header<ModrinthPaymentId>>,
    web::Json(error_body): web::Json<ErrorBody>,
) -> Result<HttpResponse, ApiError> {
    trace!(
        %trace,
        r#type = r#type.as_str(),
        ?maybe_payment_id,
        ?error_body,
        "Received Gotenberg error webhook"
    );

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug)]
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

#[derive(Debug)]
struct ModrinthGeneratedPdfType(GeneratedPdfType);

impl header::TryIntoHeaderValue for ModrinthGeneratedPdfType {
    type Error = header::InvalidHeaderValue;

    fn try_into_value(self) -> Result<header::HeaderValue, Self::Error> {
        header::HeaderValue::from_str(self.0.as_str())
    }
}

impl header::Header for ModrinthGeneratedPdfType {
    fn name() -> header::HeaderName {
        MODRINTH_GENERATED_PDF_TYPE
    }

    fn parse<M: HttpMessage>(m: &M) -> Result<Self, ParseError> {
        m.headers()
            .get(Self::name())
            .ok_or(ParseError::Header)?
            .to_str()
            .map_err(|_| ParseError::Header)?
            .parse()
            .map_err(|_| ParseError::Header)
            .map(ModrinthGeneratedPdfType)
    }
}

#[derive(Debug)]
struct ModrinthPaymentId(String);

impl header::TryIntoHeaderValue for ModrinthPaymentId {
    type Error = header::InvalidHeaderValue;

    fn try_into_value(self) -> Result<header::HeaderValue, Self::Error> {
        header::HeaderValue::from_str(&self.0)
    }
}

impl header::Header for ModrinthPaymentId {
    fn name() -> header::HeaderName {
        MODRINTH_PAYMENT_ID
    }

    fn parse<M: HttpMessage>(m: &M) -> Result<Self, ParseError> {
        m.headers()
            .get(Self::name())
            .ok_or(ParseError::Header)?
            .to_str()
            .map_err(|_| ParseError::Header)
            .map(ToOwned::to_owned)
            .map(ModrinthPaymentId)
    }
}
