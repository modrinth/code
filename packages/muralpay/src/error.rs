use std::{collections::HashMap, fmt};

use bytes::Bytes;
use derive_more::{Display, Error, From};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Display, Error, From)]
pub enum MuralError {
    #[display("API error")]
    Api(ApiError),
    #[display("request error")]
    Request(reqwest::Error),
    #[display("failed to decode response\n{json:?}")]
    #[from(skip)]
    Decode {
        source: serde_json::Error,
        json: Bytes,
    },
    #[display("failed to decode error response\n{json:?}")]
    #[from(skip)]
    DecodeError {
        source: serde_json::Error,
        json: Bytes,
    },
}

pub type Result<T, E = MuralError> = std::result::Result<T, E>;

#[derive(Debug, Display, Error, From)]
pub enum TransferError {
    #[display("no transfer API key")]
    NoTransferKey,
    #[display("API error")]
    Api(Box<ApiError>),
    #[display("request error")]
    Request(reqwest::Error),
    #[display("failed to decode response\n{json:?}")]
    #[from(skip)]
    Decode {
        source: serde_json::Error,
        json: Bytes,
    },
    #[display("failed to decode error response\n{json:?}")]
    #[from(skip)]
    DecodeError {
        source: serde_json::Error,
        json: Bytes,
    },
}

impl From<MuralError> for TransferError {
    fn from(value: MuralError) -> Self {
        match value {
            MuralError::Api(x) => Self::Api(Box::new(x)),
            MuralError::Request(x) => Self::Request(x),
            MuralError::Decode { source, json } => {
                Self::Decode { source, json }
            }
            MuralError::DecodeError { source, json } => {
                Self::DecodeError { source, json }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub error_instance_id: Uuid,
    pub name: String,
    pub message: String,
    #[serde(deserialize_with = "one_or_many")]
    #[serde(default)]
    pub details: Vec<String>,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

fn one_or_many<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }

    match OneOrMany::deserialize(deserializer)? {
        OneOrMany::One(s) => Ok(vec![s]),
        OneOrMany::Many(v) => Ok(v),
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut lines = vec![self.message.to_string()];

        if !self.details.is_empty() {
            lines.push("details:".into());
            lines.extend(self.details.iter().map(|s| format!("- {s}")));
        }

        if !self.params.is_empty() {
            lines.push("params:".into());
            lines
                .extend(self.params.iter().map(|(k, v)| format!("- {k}: {v}")));
        }

        lines.push(format!("error name: {}", self.name));
        lines.push(format!("error instance id: {}", self.error_instance_id));

        write!(f, "{}", lines.join("\n"))
    }
}
