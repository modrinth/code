use crate::routes::ApiError;
use crate::routes::v3::project_creation::CreateError;
use crate::util::validate::validation_errors_to_string;
use actix_multipart::Field;
use actix_web::web::Payload;
use bytes::BytesMut;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use validator::Validate;

pub async fn read_limited_from_payload(
    payload: &mut Payload,
    cap: usize,
    err_msg: &'static str,
) -> Result<BytesMut, ApiError> {
    let mut bytes = BytesMut::new();
    while let Some(item) = payload.next().await {
        if bytes.len() >= cap {
            return Err(ApiError::InvalidInput(String::from(err_msg)));
        } else {
            bytes.extend_from_slice(&item.map_err(|_| {
                ApiError::InvalidInput(
                    "Unable to parse bytes in payload sent!".to_string(),
                )
            })?);
        }
    }
    Ok(bytes)
}

pub async fn read_typed_from_payload<T>(
    payload: &mut Payload,
) -> Result<T, ApiError>
where
    T: DeserializeOwned + Validate,
{
    let mut bytes = BytesMut::new();
    while let Some(item) = payload.next().await {
        bytes.extend_from_slice(&item.map_err(|_| {
            ApiError::InvalidInput(
                "Unable to parse bytes in payload sent!".to_string(),
            )
        })?);
    }

    let parsed: T = serde_json::from_slice(&bytes)?;
    parsed.validate().map_err(|err| {
        ApiError::InvalidInput(validation_errors_to_string(err, None))
    })?;
    Ok(parsed)
}

pub async fn read_from_field(
    field: &mut Field,
    cap: usize,
    err_msg: &'static str,
) -> Result<BytesMut, CreateError> {
    let mut bytes = BytesMut::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk?;

        if bytes.len().saturating_add(chunk.len()) > cap {
            return Err(CreateError::InvalidInput(String::from(err_msg)));
        }

        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
