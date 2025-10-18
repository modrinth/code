#![doc = include_str!("../README.md")]

mod error;
pub use error::*;

use eyre::{Result, eyre};

/// Fetches an environment variable, possibly loading it using [`dotenvy`].
///
/// # Errors
///
/// Errors if the environment variable is missing or empty, providing a
/// pretty-printed error including the environment variable name.
#[track_caller]
pub fn env_var(key: &str) -> Result<String> {
    let value = dotenvy::var(key)
        .wrap_err_with(|| eyre!("missing environment variable `{key}`"))?;
    if value.is_empty() {
        Err(eyre!("environment variable `{key}` is empty"))
    } else {
        Ok(value)
    }
}
