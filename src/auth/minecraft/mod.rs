mod auth;
mod login;
mod refresh;
mod socket;
mod stages;

use actix_web::http::StatusCode;
use actix_web::web::{scope, ServiceConfig};
use actix_web::HttpResponse;
use serde_json::json;
use std::fmt::{Display, Formatter};

/// Error message
#[derive(Debug)]
pub struct Error {
    pub code: StatusCode,
    pub reason: String,
}

impl Error {
    pub fn render_string(reason: &str) -> String {
        json!({ "error": reason }).to_string()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            json!({
                "error": self.reason
            })
        )?;

        Ok(())
    }
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        self.code
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.code).json(json!({
            "error": self.reason
        }))
    }
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("minecraft")
            .service(auth::route)
            .service(login::route)
            .service(refresh::route)
            .service(socket::route),
    );
}
