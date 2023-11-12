use super::v3::project_creation::CreateError;
use crate::util::actix::{generate_multipart, MultipartSegment, MultipartSegmentData};
use actix_multipart::Multipart;
use actix_web::http::header::{HeaderMap, TryIntoHeaderPair};
use actix_web::HttpResponse;
use futures::{stream, StreamExt};
use serde_json::json;

pub async fn extract_ok_json<T>(response: HttpResponse) -> Result<T, HttpResponse>
where
    T: serde::de::DeserializeOwned,
{
    if response.status() == actix_web::http::StatusCode::OK {
        let failure_http_response = || {
            HttpResponse::InternalServerError().json(json!({
                "error": "reroute_error",
                "description": "Could not parse response from V2 redirection of route."
            }))
        };
        // Takes json out of HttpResponse, mutates it, then regenerates the HttpResponse
        let body = response.into_body();
        let bytes = actix_web::body::to_bytes(body)
            .await
            .map_err(|_| failure_http_response())?;
        let json_value: T = serde_json::from_slice(&bytes).map_err(|_| failure_http_response())?;
        Ok(json_value)
    } else {
        Err(response)
    }
}

pub async fn alter_actix_multipart<T, U>(
    mut multipart: Multipart,
    mut headers: HeaderMap,
    mut closure: impl FnMut(T) -> Result<U, CreateError>,
) -> Result<Multipart, CreateError>
where
    T: serde::de::DeserializeOwned,
    U: serde::Serialize,
{
    let mut segments: Vec<MultipartSegment> = Vec::new();

    if let Some(field) = multipart.next().await {
        let mut field = field?;
        let content_disposition = field.content_disposition().clone();
        let field_name = content_disposition.get_name().unwrap_or("");
        let field_filename = content_disposition.get_filename();
        let field_content_type = field.content_type();
        let field_content_type = field_content_type.map(|ct| ct.to_string());

        let mut buffer = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            buffer.extend_from_slice(&data);
        }

        {
            let json_value: T = serde_json::from_slice(&buffer)?;
            let json_value: U = closure(json_value)?;
            buffer = serde_json::to_vec(&json_value)?;
        }

        segments.push(MultipartSegment {
            name: field_name.to_string(),
            filename: field_filename.map(|s| s.to_string()),
            content_type: field_content_type,
            data: MultipartSegmentData::Binary(buffer),
        })
    }

    while let Some(field) = multipart.next().await {
        let mut field = field?;
        let content_disposition = field.content_disposition().clone();
        let field_name = content_disposition.get_name().unwrap_or("");
        let field_filename = content_disposition.get_filename();
        let field_content_type = field.content_type();
        let field_content_type = field_content_type.map(|ct| ct.to_string());

        let mut buffer = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            buffer.extend_from_slice(&data);
        }

        segments.push(MultipartSegment {
            name: field_name.to_string(),
            filename: field_filename.map(|s| s.to_string()),
            content_type: field_content_type,
            data: MultipartSegmentData::Binary(buffer),
        })
    }

    let (boundary, payload) = generate_multipart(segments);

    match (
        "Content-Type",
        format!("multipart/form-data; boundary={}", boundary).as_str(),
    )
        .try_into_pair()
    {
        Ok((key, value)) => {
            headers.insert(key, value);
        }
        Err(err) => {
            CreateError::InvalidInput(format!("Error inserting test header: {:?}.", err));
        }
    };

    let new_multipart = Multipart::new(&headers, stream::once(async { Ok(payload) }));

    Ok(new_multipart)
}
