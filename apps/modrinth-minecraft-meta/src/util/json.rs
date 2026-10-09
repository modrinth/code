use anyhow::{Context, anyhow};
use serde::{Serialize, de::DeserializeOwned};

pub fn from_json_value<T: DeserializeOwned>(
    value: &serde_json::Value,
) -> anyhow::Result<T> {
    serde_path_to_error::deserialize::<_, T>(value).map_err(|err| {
        let path = err.path().to_string();
        let err = anyhow!(err.into_inner());
        anyhow!("at `{path}`: {err:#}")
    })
}

pub fn from_json_slice<T: DeserializeOwned>(data: &[u8]) -> anyhow::Result<T> {
    let value = serde_json::from_slice::<serde_json::Value>(data)
        .with_context(|| {
            anyhow!("invalid JSON\n\n{}", String::from_utf8_lossy(data))
        })?;
    from_json_value(&value)
}

pub fn from_json_str<T: DeserializeOwned>(str: &str) -> anyhow::Result<T> {
    let value = serde_json::from_str::<serde_json::Value>(str)
        .with_context(|| anyhow!("invalid JSON\n\n{str}"))?;
    from_json_value(&value)
}

pub fn to_json_string<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("serialization should never fail")
}

pub fn de_error<'de, D: serde::Deserializer<'de>>(
    err: impl Into<anyhow::Error>,
) -> D::Error {
    let err = err.into();
    serde::de::Error::custom(format!("{err:#}"))
}
