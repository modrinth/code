use jiff::{Timestamp, civil::DateTime, tz::TimeZone};
use serde::{Deserialize, Deserializer};

/// Parses timestamps with an explicit offset, or assumes UTC for offsetless datetimes.
pub fn deserialize_timestamp<'de, D>(
    deserializer: D,
) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value
        .parse::<Timestamp>()
        .or_else(|_| {
            if !value.split_once('T').is_some_and(|(_, time)| {
                time.contains(':')
                    && time.bytes().all(|byte| {
                        byte.is_ascii_digit() || matches!(byte, b':' | b'.')
                    })
            }) {
                return value.parse::<Timestamp>();
            }
            value
                .parse::<DateTime>()?
                .to_zoned(TimeZone::UTC)
                .map(|datetime| datetime.timestamp())
        })
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Example {
        #[serde(deserialize_with = "deserialize_timestamp")]
        time: Timestamp,
    }

    #[test]
    fn explicit_offsets_are_preserved() {
        for value in ["2026-10-11T19:45:54Z", "2026-10-11T19:45:54+02:00"] {
            let example: Example =
                serde_json::from_value(serde_json::json!({"time": value}))
                    .unwrap();
            assert_eq!(example.time, value.parse::<Timestamp>().unwrap());
        }
    }

    #[test]
    fn invalid_timestamps_are_rejected() {
        for value in [
            "invalid",
            "2026-10-11",
            "2026-99-11T19:45:54",
            "2026-10-11T19:45:54+99:00",
        ] {
            assert!(
                serde_json::from_value::<Example>(
                    serde_json::json!({"time": value})
                )
                .is_err()
            );
        }
    }
}
