#![doc = include_str!("../README.md")]

mod error;

#[cfg(feature = "decimal")]
pub mod decimal;

pub use error::*;
pub use modrinth_log as log;

use anyhow::{Result, anyhow};

/// Fetches an environment variable, possibly loading it using [`dotenvy`].
///
/// # Errors
///
/// Errors if the environment variable is missing or empty, providing a
/// pretty-printed error including the environment variable name.
#[track_caller]
pub fn env_var(key: &str) -> Result<String> {
    let value = dotenvy::var(key)
        .with_context(|| anyhow!("missing environment variable `{key}`"))?;
    if value.is_empty() {
        Err(anyhow!("environment variable `{key}` is empty"))
    } else {
        Ok(value)
    }
}
