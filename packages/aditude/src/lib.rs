#![doc = include_str!("../README.md")]
#![allow(missing_docs, reason = "these are Aditude types")]

#[cfg(feature = "mock")]
pub mod mock;
pub mod v1;
pub mod v2;

use std::borrow::Cow;

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};
use secrecy::SecretString;

const PHOENIX_UTC_OFFSET_SECONDS: i32 = 7 * 60 * 60;

fn phoenix_offset() -> FixedOffset {
    FixedOffset::west_opt(PHOENIX_UTC_OFFSET_SECONDS)
        .expect("Phoenix UTC offset should be valid")
}

/// Returns the UTC instant at which a calendar day starts in Phoenix.
///
/// Aditude buckets metrics in Phoenix time, which is MST (UTC-7) year-round.
#[must_use]
pub fn phoenix_midnight(date: NaiveDate) -> DateTime<Utc> {
    date.and_hms_opt(0, 0, 0)
        .expect("midnight should be valid")
        .and_local_timezone(phoenix_offset())
        .single()
        .expect("a fixed offset should have one local midnight")
        .with_timezone(&Utc)
}

/// Returns the Phoenix calendar date containing a UTC instant.
#[must_use]
pub fn phoenix_date(time: DateTime<Utc>) -> NaiveDate {
    time.with_timezone(&phoenix_offset()).date_naive()
}

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
    let start = phoenix_midnight(phoenix_date(now - Duration::days(1)));
    let end = start + Duration::days(1);
    (start, end)
}
