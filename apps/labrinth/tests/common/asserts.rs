#![allow(dead_code)]

use crate::common::get_json_val_str;
use itertools::Itertools;
use labrinth::models::v3::projects::Version;

use super::api_common::models::CommonVersion;

#[macro_export]
macro_rules! assert_status {
    ($response:expr, $status:expr) => {
        assert_eq!(
            $response.status(),
            $status,
            "{:#?}",
            $response.response().body()
        );
    };
}

#[macro_export]
macro_rules! assert_any_status_except {
    ($response:expr, $status:expr) => {
        assert_ne!(
            $response.status(),
            $status,
            "{:#?}",
            $response.response().body()
        );
    };
}

pub fn assert_version_ids(versions: &[Version], expected_ids: Vec<String>) {
    let version_ids = versions
        .iter()
        .map(|v| get_json_val_str(v.id))
        .collect_vec();
    assert_eq!(version_ids, expected_ids);
}

pub fn assert_common_version_ids(
    versions: &[CommonVersion],
    expected_ids: Vec<String>,
) {
    let version_ids = versions
        .iter()
        .map(|v| get_json_val_str(v.id))
        .collect_vec();
    assert_eq!(version_ids, expected_ids);
}
