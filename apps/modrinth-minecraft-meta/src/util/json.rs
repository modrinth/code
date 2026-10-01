use anyhow::{Context, anyhow};
use serde::de::DeserializeOwned;

pub fn json_from_value<T: DeserializeOwned>(
    value: &serde_json::Value,
) -> anyhow::Result<T> {
    serde_path_to_error::deserialize::<_, T>(value).map_err(|err| {
        let path = err.path().to_string();
        let err = anyhow!(err.into_inner());
        anyhow!("at `{path}`: {err:#}")
    })
}

pub fn json_from_str<T: DeserializeOwned>(str: &str) -> anyhow::Result<T> {
    let value = serde_json::from_str::<serde_json::Value>(str)
        .with_context(|| anyhow!("invalid JSON\n\n{str}"))?;
    json_from_value(&value)
}
