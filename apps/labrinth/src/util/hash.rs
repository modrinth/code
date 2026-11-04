use serde::{Deserialize, Deserializer, Serializer};

/// Serializes hash bytes as a lowercase hexadecimal string.
pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(bytes))
}

/// Deserializes a hexadecimal string into hash bytes.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    hex::decode(value).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Hash {
        #[serde(with = "super")]
        value: Vec<u8>,
    }

    #[test]
    fn serializes_as_lowercase_hex() {
        let hash = Hash {
            value: vec![0, 1, 171, 255],
        };
        assert_eq!(
            serde_json::to_string(&hash).unwrap(),
            r#"{"value":"0001abff"}"#
        );
    }

    #[test]
    fn deserializes_hex() {
        let expected = Hash {
            value: vec![0, 1, 171, 255],
        };
        for value in [r#"{"value":"0001abff"}"#, r#"{"value":"0001ABFF"}"#] {
            assert_eq!(serde_json::from_str::<Hash>(value).unwrap(), expected);
        }
    }

    #[test]
    fn rejects_invalid_hex_and_non_strings() {
        for value in [
            r#"{"value":"xyz"}"#,
            r#"{"value":"abc"}"#,
            r#"{"value":[0,1,171,255]}"#,
        ] {
            assert!(serde_json::from_str::<Hash>(value).is_err());
        }
    }
}
