use crate::routes::ApiError;
use crate::routes::v3::project_creation::CreateError;
use crate::util::error::Context as _;
use actix_multipart::Field;
use actix_web::web::Payload;
use bytes::BytesMut;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use validator::Validate;

pub async fn read_limited_from_payload(
    payload: &mut Payload,
    cap: usize,
    err_msg: impl Into<String>,
) -> Result<BytesMut, ApiError> {
    let err_msg = err_msg.into();
    let mut bytes = BytesMut::new();
    while let Some(item) = payload.next().await {
        let item = item.map_err(|err| eyre::eyre!(err)).wrap_request_err(
            "unable to parse bytes in payload sent!".to_string(),
        )?;
        if bytes.len().saturating_add(item.len()) > cap {
            return Err(ApiError::Request(eyre::eyre!(err_msg)));
        }
        bytes.extend_from_slice(&item);
    }
    Ok(bytes)
}

pub async fn read_typed_from_payload<T>(
    payload: &mut Payload,
    cap: usize,
    err_msg: impl Into<String>,
) -> Result<T, ApiError>
where
    T: DeserializeOwned + Validate,
{
    let bytes = read_limited_from_payload(payload, cap, err_msg).await?;

    let parsed: T = serde_json::from_slice(&bytes)
        .wrap_request_err("deserializing JSON data")?;
    parsed
        .validate()
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("validating request")?;
    Ok(parsed)
}

pub async fn read_from_field(
    field: &mut Field,
    cap: usize,
    err_msg: impl Into<String>,
) -> Result<BytesMut, CreateError> {
    let mut bytes = BytesMut::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk?;

        if bytes.len().saturating_add(chunk.len()) > cap {
            return Err(CreateError::InvalidInput(err_msg.into()));
        }

        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
