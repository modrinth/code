use crate::routes::project_creation::CreateError;
use crate::routes::ApiError;
use actix_multipart::Field;
use actix_web::web::Payload;
use actix_web::HttpResponse;
use bytes::BytesMut;
use futures::StreamExt;
use serde::Serialize;

pub async fn read_from_payload(
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

pub async fn read_from_field(
    field: &mut Field,
    cap: usize,
    err_msg: &'static str,
) -> Result<BytesMut, CreateError> {
    let mut bytes = BytesMut::new();
    while let Some(chunk) = field.next().await {
        if bytes.len() >= cap {
            return Err(CreateError::InvalidInput(String::from(err_msg)));
        } else {
            bytes.extend_from_slice(&chunk?);
        }
    }
    Ok(bytes)
}

pub(crate) fn ok_or_not_found<T, U>(
    version_data: Option<T>,
) -> Result<HttpResponse, ApiError>
where
    U: From<T> + Serialize,
{
    if let Some(data) = version_data {
        Ok(HttpResponse::Ok().json(U::from(data)))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
