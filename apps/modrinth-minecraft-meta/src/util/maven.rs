use std::str::FromStr;

use anyhow::Context;
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Clone)]
#[display("{group_id}:{artifact_id}:{version}")]
pub struct MavenCoordinate {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
}

impl FromStr for MavenCoordinate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (group_id, s) = s.split_once(':').context("no separators")?;
        let (artifact_id, s) = s.split_once(':').context("no separators")?;
        let version = s.split_once(':').map(|(s, _)| s).unwrap_or(s);
        Ok(Self {
            group_id: group_id.to_string(),
            artifact_id: artifact_id.to_string(),
            version: version.to_string(),
        })
    }
}

impl Serialize for MavenCoordinate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MavenCoordinate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Self>().map_err(serde::de::Error::custom)
    }
}
