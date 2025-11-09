//! Re-exports all [`labrinth::test`] items for compatibility.
//!
//! Previously, tests used `mod common` and `common::item` imports for testing.
//! This has been moved into [`labrinth::test`] under a feature flag, and this
//! module remains for backwards compatibility with tests which expect the
//! `common` module.

pub use labrinth::test::*;

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
