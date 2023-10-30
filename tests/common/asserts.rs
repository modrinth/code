#![allow(dead_code)]

pub fn assert_status(response: &actix_web::dev::ServiceResponse, status: actix_http::StatusCode) {
    assert_eq!(response.status(), status, "{:#?}", response.response());
}

pub fn assert_any_status_except(
    response: &actix_web::dev::ServiceResponse,
    status: actix_http::StatusCode,
) {
    assert_ne!(response.status(), status, "{:#?}", response.response());
}
