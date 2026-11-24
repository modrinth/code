use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;
use serde_json::value::RawValue;

use crate::enums::Country;
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
            source: batch.source.clone(),
            data: event.event,
        })
    }

    Ok(ExtractedAppEvents { events, rejected })
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
    use crate::event::AppEventType;

    const INSTALLATION_ID: &str = "67e55044-10b1-426f-9247-bb680e5fe0c8";
    const EVENT_ID_A: &str = "01890a5d-ac96-774b-bcce-b302099a8057";
    const EVENT_ID_B: &str = "01890a5d-ac96-774b-bcce-b302099a8058";
    const NO_HEADERS: [(&str, &str); 0] = [];

    fn arrived_at() -> DateTime<Utc> {
        "2026-11-23T12:00:00Z".parse().unwrap()
    }

    fn body(events: &str) -> String {
        format!(
            r#"{{"sent_at":"2026-11-23T11:59:59Z","installation_id":"{INSTALLATION_ID}","os":"linux","arch":"x86_64","app_version":"0.10.3","source":"app","events":[{events}]}}"#
        )
    }

    fn event(id: &str, event: &str) -> String {
        format!(
            r#"{{"id":"{id}","timestamp":"2026-11-23T11:00:00Z","event":{event}}}"#
        )
    }

    #[test]
    fn extracts_externally_tagged_events() {
        let body = body(&format!(
            "{},{}",
            event(
                EVENT_ID_A,
                r#"{"InstanceStart":{"loader":"fabric","loader_version":"0.16.9","game_version":"1.21.4"}}"#
            ),
            event(EVENT_ID_B, r#"{"MicrosoftSignIn":{}}"#),
        ));

        let extracted =
            extract_events(&body, NO_HEADERS, arrived_at()).unwrap();

        assert!(extracted.rejected.is_empty(), "{:?}", extracted.rejected);
        let data: Vec<_> =
            extracted.events.into_iter().map(|e| e.data).collect();
        assert_eq!(
            data,
            [
                AppEventType::InstanceStart {
                    loader: crate::enums::AppLoader::Fabric,
                    loader_version: Some("0.16.9".into()),
                    game_version: "1.21.4".into(),
                },
                AppEventType::MicrosoftSignIn {},
            ]
        );
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
    fn rejects_trailing_characters() {
        let body = body(&event(EVENT_ID_A, r#"{"MicrosoftSignIn":{}}"#)) + "x";

        let err = extract_events(&body, NO_HEADERS, arrived_at()).unwrap_err();

        assert!(matches!(err, ExtractError::Deserialize(_)));
    }

    #[test]
    fn prefers_alt_ipcountry_over_cf_ipcountry() {
        let country = |headers: &[(&str, &str)]| {
            extract_headers(headers.iter().copied()).ip_country
        };

        assert_eq!(
            country(&[("CF-IPCountry", "FR"), ("Alt-IPCountry", "DE")]),
            Country::parse("DE").unwrap()
        );
        assert_eq!(
            country(&[("cf-ipcountry", "FR"), ("alt-ipcountry", "??")]),
            Country::parse("FR").unwrap()
        );
        assert_eq!(country(&[]), Country::UNKNOWN);
    }
}
