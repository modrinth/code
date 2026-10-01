use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone)]
pub struct Sha1(pub [u8; 20]);

impl fmt::Debug for Sha1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl Serialize for Sha1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Sha1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let mut bytes = [0; 20];
        hex::decode_to_slice(value, &mut bytes)
            .map_err(serde::de::Error::custom)?;
        Ok(Self(bytes))
    }
}
