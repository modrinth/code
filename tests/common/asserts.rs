#![allow(dead_code)]

use crate::common::get_json_val_str;
use itertools::Itertools;
use labrinth::models::v2::projects::LegacyVersion;

pub fn assert_status(response: &actix_web::dev::ServiceResponse, status: actix_http::StatusCode) {
    assert_eq!(response.status(), status, "{:#?}", response.response());
}

pub fn assert_version_ids(versions: &[LegacyVersion], expected_ids: Vec<String>) {
    let version_ids = versions
        .iter()
        .map(|v| get_json_val_str(v.id))
        .collect_vec();
    assert_eq!(version_ids, expected_ids);
}

pub fn assert_any_status_except(
    response: &actix_web::dev::ServiceResponse,
    status: actix_http::StatusCode,
) {
    assert_ne!(response.status(), status, "{:#?}", response.response());
}
