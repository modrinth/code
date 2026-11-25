use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;
use serde_json::value::RawValue;

use crate::enums::Country;
use crate::event::{AppEventType, InstanceInfo};
use crate::http::batch::{
    AppEvent, AppEventBatch, SkewCorrectionError, SkewCorrectionParameters,
};
use crate::normal::NormalizedAppEvent;

/// Largest request body accepted, in bytes.
pub const MAX_BODY_BYTES: usize = 512 * 1024;

/// Most events accepted in one batch.
pub const MAX_EVENTS_PER_BATCH: usize = 128;

/// Oldest an event may be when its batch is sent. Clients should drop older
/// events instead of sending them.
pub const MAX_EVENT_AGE: TimeDelta = TimeDelta::days(2);

/// How far in advance an event may be when its batch is sent.
///
/// Note events still mainly use corrected client time, and not the raw event time sent
/// by the client, so this does NOT mean data up to 23 hours in the future is accepted and
/// shown in query, this is purely to account for client clock skew.
pub const MAX_EVENT_IN_ADVANCE: TimeDelta = TimeDelta::hours(23);

/// Longest app version accepted, in bytes.
pub const MAX_APP_VERSION_LEN: usize = 32;

/// Longest event source accepted, in bytes.
pub const MAX_SOURCE_LEN: usize = 23;

/// Longest game or loader version accepted, in bytes.
pub const MAX_INSTANCE_VERSION_FIELD_LEN: usize = 64;

/// Events extracted from a received batch.
#[derive(Debug)]
pub struct ExtractedAppEvents {
    pub events: Vec<NormalizedAppEvent>,
    /// Events that were dropped, such as event types this version does not
    /// know about yet. The rest of the batch is still accepted.
    pub rejected: Vec<SingleEventProcessingError>,
}

/// Error that rejects a whole batch.
#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("request body is larger than {MAX_BODY_BYTES} bytes")]
    BodyTooLarge,
    #[error("failed to deserialize app event batch: {0}")]
    Deserialize(#[source] serde_path_to_error::Error<serde_json::Error>),
    #[error("app event batch contains no events")]
    Empty,
    #[error("app event batch contains more than {MAX_EVENTS_PER_BATCH} events")]
    TooManyEvents,
    #[error("installation id is not a valid UUID v4")]
    InstallationIdIsNotUuidVersion4,
    #[error(
        "app version is not 1 to {MAX_APP_VERSION_LEN} ASCII letters, digits, `.`, `+` or `-`"
    )]
    InvalidAppVersion,
}

#[derive(Debug, thiserror::Error)]
pub enum SingleEventProcessingError {
    #[error("failed to deserialize app event: {0}")]
    Deserialize(#[source] serde_path_to_error::Error<serde_json::Error>),
    #[error("failed to correct timestamp skew: {0}")]
    TimestampSkewCorrection(
        #[source]
        #[from]
        SkewCorrectionError,
    ),
    #[error("id is not a valid UUID v7")]
    IdIsNotUuidVersion7,
    #[error(
        "source is not 1 to {MAX_SOURCE_LEN} ASCII letters or digits starting with an uppercase letter"
    )]
    InvalidSource,
    #[error(
        "app version is not 1 to {MAX_APP_VERSION_LEN} ASCII letters, digits, `.`, `+` or `-`"
    )]
    InvalidAppVersion,
    #[error(
        "game version field is not 1 to {MAX_INSTANCE_VERSION_FIELD_LEN} bytes"
    )]
    InvalidInstanceGameVersion,
    #[error(
        "loader version field is not 1 to {MAX_INSTANCE_VERSION_FIELD_LEN} bytes"
    )]
    InvalidInstanceLoaderVersion,
}

/// Validates a request body sent by the app and extracts its events.
///
/// `headers` are the request headers as name and value pairs, used for the
/// client's country. Header names are matched case-insensitively.
///
/// # Errors
///
/// Errors if the batch as a whole is invalid. Individual invalid events are
/// returned in [`ExtractedAppEvents::rejected`] instead.
pub fn extract_events<K: AsRef<str>, V: AsRef<str>>(
    body: &str,
    headers: impl IntoIterator<Item = (K, V)>,
    arrived_at: DateTime<Utc>,
) -> Result<ExtractedAppEvents, ExtractError> {
    // in the happy path, with no JSON escapes and strings that fit inline in a `SmolStr`,
    // this function only allocates while growing `batch.events` and once for `events`

    if body.len() > MAX_BODY_BYTES {
        return Err(ExtractError::BodyTooLarge);
    }

    let batch: AppEventBatch<&RawValue> =
        from_str_with_error_path(body).map_err(ExtractError::Deserialize)?;

    if batch.events.is_empty() {
        return Err(ExtractError::Empty);
    }

    if batch.events.len() > MAX_EVENTS_PER_BATCH {
        return Err(ExtractError::TooManyEvents);
    }

    if batch.installation_id.get_version_num() != 4 {
        return Err(ExtractError::InstallationIdIsNotUuidVersion4);
    }

    if !is_valid_app_version(&batch.app_version) {
        return Err(ExtractError::InvalidAppVersion);
    }

    let header_data = extract_headers(headers);

    let mut events = Vec::with_capacity(batch.events.len());
    let mut rejected = Vec::new();

    for raw_event in batch.events {
        let event: AppEvent = match from_str_with_error_path(raw_event.get()) {
            Ok(event) => event,
            Err(err) => {
                rejected.push(SingleEventProcessingError::Deserialize(err));
                continue;
            }
        };

        let corrected_result =
            event.corrected_timestamp_skew(&SkewCorrectionParameters {
                client_sent_at: batch.sent_at,
                server_received_at: arrived_at,
            });

        let event_ts = match corrected_result {
            Ok(corrected) => corrected,
            Err(err) => {
                rejected.push(err.into());
                continue;
            }
        };

        if event.id.get_version_num() != 7 {
            rejected.push(SingleEventProcessingError::IdIsNotUuidVersion7);
            continue;
        }

        if !is_valid_source(&event.source) {
            rejected.push(SingleEventProcessingError::InvalidSource);
            continue;
        }

        if let AppEventType::AppStart { app_version, .. } = &event.event
            && !is_valid_app_version(app_version)
        {
            rejected.push(SingleEventProcessingError::InvalidAppVersion);
            continue;
        }

        if let AppEventType::InstanceStart { info } = &event.event
            && let Err(err) = validate_instance_info(info)
        {
            rejected.push(err);
            continue;
        }

        events.push(NormalizedAppEvent {
            event_id: event.id,
            sent_at: batch.sent_at,
            client_timestamp: event.timestamp,
            event_ts,
            ip_country: header_data.ip_country,
            installation_id: batch.installation_id,
            os: batch.os,
            arch: batch.arch,
            app_version: batch.app_version.clone(),
            source: event.source,
            data: event.event,
        })
    }

    Ok(ExtractedAppEvents { events, rejected })
}

/// Checks that a client-supplied app version is short and uses only the
/// characters of a semantic version, so the column stays low-cardinality and
/// free of arbitrary text.
fn is_valid_app_version(version: &str) -> bool {
    (1..=MAX_APP_VERSION_LEN).contains(&version.len())
        && version.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'.' | b'+' | b'-')
        })
}

/// Checks that the client-supplied game and loader versions of an instance
/// are within the accepted length.
fn validate_instance_info(
    info: &InstanceInfo,
) -> Result<(), SingleEventProcessingError> {
    if !is_valid_instance_version_field(&info.game_version) {
        return Err(SingleEventProcessingError::InvalidInstanceGameVersion);
    }

    if let Some(loader_version) = &info.loader_version
        && !is_valid_instance_version_field(loader_version)
    {
        return Err(SingleEventProcessingError::InvalidInstanceLoaderVersion);
    }

    Ok(())
}

fn is_valid_instance_version_field(version: &str) -> bool {
    (1..=MAX_INSTANCE_VERSION_FIELD_LEN).contains(&version.len())
}

/// Checks that an event source is a short PascalCase identifier, such as
/// `BrowseTab` or `NewInstance`: ASCII letters and digits, starting with an
/// uppercase letter.
#[must_use]
pub fn is_valid_source(source: &str) -> bool {
    (1..=MAX_SOURCE_LEN).contains(&source.len())
        && source.bytes().all(|b| b.is_ascii_alphanumeric())
        && source
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_uppercase())
}

/// Deserializes `T` from JSON, reporting the path to the value that failed.
///
/// Tracking the path allocates for every object key, so this first parses
/// without it and only parses again with [`serde_path_to_error`] if that fails.
fn from_str_with_error_path<'a, T: Deserialize<'a>>(
    json: &'a str,
) -> Result<T, serde_path_to_error::Error<serde_json::Error>> {
    let first_err = match serde_json::from_str(json) {
        Ok(value) => return Ok(value),
        Err(err) => err,
    };

    let mut track = serde_path_to_error::Track::new();
    let mut deserializer = serde_json::Deserializer::from_str(json);
    let result = T::deserialize(serde_path_to_error::Deserializer::new(
        &mut deserializer,
        &mut track,
    ))
    .and_then(|_| deserializer.end());

    let err = match result {
        Err(err) => err,
        Ok(()) => first_err,
    };
    Err(serde_path_to_error::Error::new(track.path(), err))
}

struct ExtractedHeaderData {
    pub ip_country: Country,
}

fn extract_headers<K: AsRef<str>, V: AsRef<str>>(
    headers: impl IntoIterator<Item = (K, V)>,
) -> ExtractedHeaderData {
    let mut cf_ipcountry = None;
    let mut alt_ipcountry = None;

    for (k, v) in headers {
        let (k, v) = (k.as_ref(), v.as_ref());
        if k.eq_ignore_ascii_case("cf-ipcountry") {
            cf_ipcountry = Country::parse(v);
        } else if k.eq_ignore_ascii_case("alt-ipcountry") {
            alt_ipcountry = Country::parse(v);
        }
    }

    ExtractedHeaderData {
        ip_country: alt_ipcountry.or(cf_ipcountry).unwrap_or(Country::UNKNOWN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSTALLATION_ID: &str = "67e55044-10b1-426f-9247-bb680e5fe0c8";
    const EVENT_ID_A: &str = "01890a5d-ac96-774b-bcce-b302099a8057";
    const EVENT_ID_B: &str = "01890a5d-ac96-774b-bcce-b302099a8058";
    const NO_HEADERS: [(&str, &str); 0] = [];

    fn arrived_at() -> DateTime<Utc> {
        "2026-11-23T12:00:00Z".parse().unwrap()
    }

    fn body(events: &str) -> String {
        format!(
            r#"{{"sent_at":"2026-11-23T11:59:59Z","installation_id":"{INSTALLATION_ID}","os":"linux","arch":"x86_64","app_version":"0.10.3","events":[{events}]}}"#
        )
    }

    fn event(id: &str, event: &str) -> String {
        format!(
            r#"{{"id":"{id}","timestamp":"2026-11-23T11:00:00Z","source":"BrowseTab","event":{event}}}"#
        )
    }
    #[test]
    fn rejects_unknown_event_without_rejecting_batch() {
        let body = body(&format!(
            "{},{}",
            event(EVENT_ID_A, r#"{"NewThing":{"x":1}}"#),
            event(EVENT_ID_B, r#"{"MicrosoftSignIn":{}}"#),
        ));

        let extracted =
            extract_events(&body, NO_HEADERS, arrived_at()).unwrap();

        assert_eq!(extracted.events.len(), 1);
        assert!(matches!(
            extracted.rejected.as_slice(),
            [SingleEventProcessingError::Deserialize(err)] if err.path().to_string() == "event"
        ));
    }

    #[test]
    fn reports_path_of_invalid_batch_field() {
        let body = body(&event(EVENT_ID_A, r#"{"MicrosoftSignIn":{}}"#))
            .replace(r#""os":"linux""#, r#""os":"beos""#);

        let err = extract_events(&body, NO_HEADERS, arrived_at()).unwrap_err();

        assert!(matches!(
            err,
            ExtractError::Deserialize(err) if err.path().to_string() == "os"
        ));
    }

    #[test]
    fn rejects_batch_with_invalid_app_version() {
        let body = body(&event(EVENT_ID_A, r#"{"MicrosoftSignIn":{}}"#))
            .replace(r#""app_version":"0.10.3""#, r#""app_version":"0.10 3""#);

        let err = extract_events(&body, NO_HEADERS, arrived_at()).unwrap_err();

        assert!(matches!(err, ExtractError::InvalidAppVersion));
    }

    #[test]
    fn rejects_event_with_invalid_source_without_rejecting_batch() {
        let body = body(&format!(
            "{},{}",
            event(EVENT_ID_A, r#"{"MicrosoftSignIn":{}}"#)
                .replace(r#""source":"BrowseTab""#, r#""source":"browse_tab""#),
            event(EVENT_ID_B, r#"{"MicrosoftSignIn":{}}"#),
        ));

        let extracted =
            extract_events(&body, NO_HEADERS, arrived_at()).unwrap();

        assert_eq!(extracted.events.len(), 1);
        assert!(matches!(
            extracted.rejected.as_slice(),
            [SingleEventProcessingError::InvalidSource]
        ));
    }

    #[test]
    fn rejects_app_start_with_invalid_app_version() {
        let body = body(&event(
            EVENT_ID_A,
            r#"{"AppStart":{"arch":"x86_64","os":"linux","app_version":"<script>"}}"#,
        ));

        let extracted =
            extract_events(&body, NO_HEADERS, arrived_at()).unwrap();

        assert!(extracted.events.is_empty());
        assert!(matches!(
            extracted.rejected.as_slice(),
            [SingleEventProcessingError::InvalidAppVersion]
        ));
    }

    #[test]
    fn validates_source_format() {
        assert!(is_valid_source("BrowseTab"));
        assert!(is_valid_source("NewInstance2"));
        assert!(is_valid_source("X"));
        assert!(!is_valid_source(""));
        assert!(!is_valid_source("browseTab"));
        assert!(!is_valid_source("Browse_Tab"));
        assert!(!is_valid_source("2Browse"));
        assert!(!is_valid_source(&format!(
            "A{}",
            "a".repeat(MAX_SOURCE_LEN)
        )));
    }
}
