use std::collections::HashMap;

use super::ApiError;
use super::v3::project_creation::CreateError;
use crate::models::v2::projects::LegacySideType;
use crate::util::actix::{
    MultipartSegment, MultipartSegmentData, generate_multipart,
};
use actix_multipart::Multipart;
use actix_web::HttpResponse;
use actix_web::http::header::{
    ContentDisposition, HeaderMap, TryIntoHeaderPair,
};
use futures::{Future, StreamExt, stream};
use serde_json::{Value, json};

pub async fn extract_ok_json<T>(
    response: HttpResponse,
) -> Result<T, HttpResponse>
where
    T: serde::de::DeserializeOwned,
{
    // If the response is StatusCode::OK, parse the json and return it
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
        let json_value: T = serde_json::from_slice(&bytes)
            .map_err(|_| failure_http_response())?;
        Ok(json_value)
    } else {
        Err(response)
    }
}

// This only removes the body of 404 responses
// This should not be used on the fallback no-route-found handler
pub fn flatten_404_error(res: ApiError) -> Result<HttpResponse, ApiError> {
    match res {
        ApiError::NotFound => Ok(HttpResponse::NotFound().body("")),
        _ => Err(res),
    }
}

// Allows internal modification of an actix multipart file
// Expected:
// 1. A json segment
// 2. Any number of other binary segments
// 'closure' is called with the json value, and the content disposition of the other segments
pub async fn alter_actix_multipart<T, U, Fut>(
    mut multipart: Multipart,
    mut headers: HeaderMap,
    mut closure: impl FnMut(T, Vec<ContentDisposition>) -> Fut,
) -> Result<Multipart, CreateError>
where
    T: serde::de::DeserializeOwned,
    U: serde::Serialize,
    Fut: Future<Output = Result<U, CreateError>>,
{
    let mut segments: Vec<MultipartSegment> = Vec::new();

    let mut json = None;
    let mut json_segment = None;
    let mut content_dispositions = Vec::new();

    if let Some(field) = multipart.next().await {
        let mut field = field?;
        let content_disposition = field.content_disposition().unwrap().clone();
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
            json = Some(json_value);
        }

        json_segment = Some(MultipartSegment {
            name: field_name.to_string(),
            filename: field_filename.map(|s| s.to_string()),
            content_type: field_content_type,
            data: MultipartSegmentData::Binary(vec![]), // Initialize to empty, will be finished after
        });
    }

    while let Some(field) = multipart.next().await {
        let mut field = field?;
        let content_disposition = field.content_disposition().unwrap().clone();
        let field_name = content_disposition.get_name().unwrap_or("");
        let field_filename = content_disposition.get_filename();
        let field_content_type = field.content_type();
        let field_content_type = field_content_type.map(|ct| ct.to_string());

        let mut buffer = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            buffer.extend_from_slice(&data);
        }

        content_dispositions.push(content_disposition.clone());
        segments.push(MultipartSegment {
            name: field_name.to_string(),
            filename: field_filename.map(|s| s.to_string()),
            content_type: field_content_type,
            data: MultipartSegmentData::Binary(buffer),
        })
    }

    // Finishes the json segment, with aggregated content dispositions
    {
        let json_value = json.ok_or(CreateError::InvalidInput(
            "No json segment found in multipart.".to_string(),
        ))?;
        let mut json_segment =
            json_segment.ok_or(CreateError::InvalidInput(
                "No json segment found in multipart.".to_string(),
            ))?;

        // Call closure, with the json value and names of the other segments
        let json_value: U = closure(json_value, content_dispositions).await?;
        let buffer = serde_json::to_vec(&json_value)?;
        json_segment.data = MultipartSegmentData::Binary(buffer);

        // Insert the json segment at the beginning
        segments.insert(0, json_segment);
    }

    let (boundary, payload) = generate_multipart(segments);

    match (
        "Content-Type",
        format!("multipart/form-data; boundary={boundary}").as_str(),
    )
        .try_into_pair()
    {
        Ok((key, value)) => {
            headers.insert(key, value);
        }
        Err(err) => {
            CreateError::InvalidInput(format!(
                "Error inserting test header: {err:?}."
            ));
        }
    };

    let new_multipart =
        Multipart::new(&headers, stream::once(async { Ok(payload) }));

    Ok(new_multipart)
}

/// Converts V2 side types to V3 side types.
pub fn convert_v2_side_types_to_v3_side_types(
    client_side: LegacySideType,
    server_side: LegacySideType,
) -> HashMap<String, Value> {
    use LegacySideType::{Optional, Required, Unsupported};

    let environment = match (client_side, server_side) {
        (Required, Required) => "client_and_server", // Or "singleplayer_only"
        (Required, Unsupported) => "client_only",
        (Required, Optional) => "client_only_server_optional",
        (Unsupported, Required) => "server_only", // Or "dedicated_server_only"
        (Optional, Required) => "server_only_client_optional",
        (Optional, Optional) => "client_or_server", // Or "client_or_server_prefers_both"
        _ => "unknown",
    };

    [("environment".to_string(), json!(environment))]
        .into_iter()
        .collect()
}

/// Converts a V3 side types map into the corresponding V2 side types.
pub fn convert_v3_side_types_to_v2_side_types(
    side_types: &HashMap<String, Value>,
    project_type: Option<&str>,
) -> (LegacySideType, LegacySideType) {
    convert_v3_environment_to_v2_side_types(
        side_types
            .get("environment")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown"),
        project_type,
    )
}

/// Converts a V3 environment and project type into the corresponding V2 side types.
/// The first side type is for the client, the second is for the server.
pub fn convert_v3_environment_to_v2_side_types(
    environment: &str,
    project_type: Option<&str>,
) -> (LegacySideType, LegacySideType) {
    use LegacySideType::{Optional, Required, Unknown, Unsupported};

    match project_type {
        Some("plugin") => (Unsupported, Required),
        Some("datapack") => (Optional, Required),
        Some("shader") => (Required, Unsupported),
        Some("resourcepack") => (Required, Unsupported),
        _ => match environment {
            "client_and_server" => (Required, Required),
            "client_only" => (Required, Unsupported),
            "client_only_server_optional" => (Required, Optional),
            "singleplayer_only" => (Required, Required),
            "server_only" => (Unsupported, Required),
            "server_only_client_optional" => (Optional, Required),
            "dedicated_server_only" => (Unsupported, Required),
            "client_or_server" => (Optional, Optional),
            "client_or_server_prefers_both" => (Optional, Optional),
            _ => (Unknown, Unknown), // "unknown"
        },
    }
}

pub fn capitalize_first(input: &str) -> String {
    input
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::v2::projects::LegacySideType::{
        Optional, Required, Unsupported,
    };

    #[test]
    fn v2_v3_side_type_conversion() {
        // Only nonsensical V2 side types cannot be round-tripped from V2 to V3 and back.
        // When converting from V3 to V2, only additional information about the
        // singleplayer-only, multiplayer-only, or install on both sides nature of the
        // project is lost.
        let lossy_pairs = [
            (Optional, Unsupported),
            (Unsupported, Optional),
            (Unsupported, Unsupported),
        ];

        for client_side in [Required, Optional, Unsupported] {
            for server_side in [Required, Optional, Unsupported] {
                if lossy_pairs.contains(&(client_side, server_side)) {
                    continue;
                }
                let side_types = convert_v2_side_types_to_v3_side_types(
                    client_side,
                    server_side,
                );
                let (client_side2, server_side2) =
                    convert_v3_side_types_to_v2_side_types(&side_types, None);

                assert_eq!(client_side, client_side2);
                assert_eq!(server_side, server_side2);
            }
        }
    }
}
