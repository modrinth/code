#![doc = include_str!("../README.md")]
#![allow(missing_docs, reason = "these are Aditude types")]

#[cfg(feature = "mock")]
pub mod mock;
pub mod v1;
pub mod v2;

use std::borrow::Cow;

use chrono::{DateTime, Duration, Utc};
use secrecy::SecretString;

/// [Aditude](https://www.aditude.com/) client.
#[derive(Debug)]
pub struct Client {
    http: reqwest::Client,
    pub api_url: Cow<'static, str>,
    pub api_key: SecretString,
    #[cfg(feature = "mock")]
    pub mock: arc_swap::ArcSwapOption<mock::AditudeMock>,
}

impl Client {
    /// Creates a new Aditude client with a [`reqwest::Client`].
    #[must_use]
    pub fn from_client(
        http: reqwest::Client,
        api_url: impl Into<Cow<'static, str>>,
        api_key: impl Into<SecretString>,
    ) -> Self {
        Self {
            http,
            api_url: api_url.into(),
            api_key: api_key.into(),
            #[cfg(feature = "mock")]
            mock: arc_swap::ArcSwapOption::empty(),
        }
    }

    /// Creates a new Aditude client.
    #[must_use]
    pub fn new(
        api_url: impl Into<Cow<'static, str>>,
        api_key: impl Into<SecretString>,
    ) -> Self {
        Self::from_client(reqwest::Client::new(), api_url, api_key)
    }

    /// Creates an Aditude client which mocks responses.
    #[cfg(feature = "mock")]
    #[must_use]
    pub fn from_mock(mock: mock::AditudeMock) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_url: "".into(),
            api_key: SecretString::from(String::new()),
            mock: arc_swap::ArcSwapOption::from_pointee(mock),
        }
    }

    /// Sets the mock responses that this client will output.
    #[cfg(feature = "mock")]
    pub fn set_mock(&self, mock: mock::AditudeMock) {
        self.mock.store(Some(std::sync::Arc::new(mock)));
    }
}

/// Returns the start and end of what Aditude considers the "Yesterday" time
/// range.
#[must_use]
pub fn yesterday(now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let start = DateTime::<Utc>::from_naive_utc_and_offset(
        (now - Duration::days(1))
            .date_naive()
            .and_hms_nano_opt(0, 0, 0, 0)
            .unwrap_or_default(),
        Utc,
    );
    let end = start + Duration::days(1);
    (start, end)
}
