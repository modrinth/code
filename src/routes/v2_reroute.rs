use std::collections::HashMap;

use super::v3::project_creation::CreateError;
use super::ApiError;
use crate::models::v2::projects::LegacySideType;
use crate::util::actix::{generate_multipart, MultipartSegment, MultipartSegmentData};
use actix_multipart::Multipart;
use actix_web::http::header::{HeaderMap, TryIntoHeaderPair};
use actix_web::HttpResponse;
use futures::{stream, Future, StreamExt};
use itertools::Itertools;
use serde_json::{json, Value};

pub async fn extract_ok_json<T>(response: HttpResponse) -> Result<T, HttpResponse>
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
        let json_value: T = serde_json::from_slice(&bytes).map_err(|_| failure_http_response())?;
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

pub async fn alter_actix_multipart<T, U, Fut>(
    mut multipart: Multipart,
    mut headers: HeaderMap,
    mut closure: impl FnMut(T) -> Fut,
) -> Result<Multipart, CreateError>
where
    T: serde::de::DeserializeOwned,
    U: serde::Serialize,
    Fut: Future<Output = Result<U, CreateError>>,
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
            let json_value: U = closure(json_value).await?;
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

// Converts a "client_side" and "server_side" pair into the new v3 corresponding fields
pub fn convert_side_types_v3(
    client_side: LegacySideType,
    server_side: LegacySideType,
) -> HashMap<String, Value> {
    use LegacySideType::{Optional, Required};

    let singleplayer = client_side == Required
        || client_side == Optional
        || server_side == Required
        || server_side == Optional;
    let client_and_server = singleplayer;
    let client_only =
        (client_side == Required || client_side == Optional) && server_side != Required;
    let server_only =
        (server_side == Required || server_side == Optional) && client_side != Required;

    let mut fields = HashMap::new();
    fields.insert("singleplayer".to_string(), json!(singleplayer));
    fields.insert("client_and_server".to_string(), json!(client_and_server));
    fields.insert("client_only".to_string(), json!(client_only));
    fields.insert("server_only".to_string(), json!(server_only));
    fields
}

// Convert search facets from V2 to V3
// Less trivial as we need to handle the case where one side is set and the other is not, which does not convert cleanly
pub fn convert_side_type_facets_v3(facets: Vec<Vec<Vec<String>>>) -> Vec<Vec<Vec<String>>> {
    use LegacySideType::{Optional, Required, Unsupported};
    let possible_side_types = [Required, Optional, Unsupported]; // Should not include Unknown

    let mut v3_facets = vec![];

    // Outer facets are joined by AND
    for inner_facets in facets {
        // Inner facets are joined by OR
        // These may change as the inner facets are converted
        // ie:
        // for A v B v C, if A is converted to X^Y v Y^Z, then the new facets are X^Y v Y^Z v B v C
        let mut new_inner_facets = vec![];

        for inner_inner_facets in inner_facets {
            // Inner inner facets are joined by AND
            let mut client_side = None;
            let mut server_side = None;

            // Extract client_side and server_side facets, and remove them from the list
            let inner_inner_facets = inner_inner_facets
                .into_iter()
                .filter_map(|facet| {
                    let val = match facet.split(':').nth(1) {
                        Some(val) => val,
                        None => return Some(facet.to_string()),
                    };

                    if facet.starts_with("client_side:") {
                        client_side = Some(LegacySideType::from_string(val));
                        None
                    } else if facet.starts_with("server_side:") {
                        server_side = Some(LegacySideType::from_string(val));
                        None
                    } else {
                        Some(facet.to_string())
                    }
                })
                .collect_vec();

            // Depending on whether client_side and server_side are set, we can convert the facets to the new loader fields differently
            let mut new_possibilities = match (client_side, server_side) {
                // Both set or unset is a trivial case
                (Some(client_side), Some(server_side)) => {
                    vec![convert_side_types_v3(client_side, server_side)
                        .into_iter()
                        .map(|(k, v)| format!("{}:{}", k, v))
                        .collect()]
                }
                (None, None) => vec![vec![]],

                (Some(client_side), None) => possible_side_types
                    .iter()
                    .map(|server_side| {
                        convert_side_types_v3(client_side, *server_side)
                            .into_iter()
                            .map(|(k, v)| format!("{}:{}", k, v))
                            .unique()
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
                (None, Some(server_side)) => possible_side_types
                    .iter()
                    .map(|client_side| {
                        convert_side_types_v3(*client_side, server_side)
                            .into_iter()
                            .map(|(k, v)| format!("{}:{}", k, v))
                            .unique()
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            };

            // Add the new possibilities to the list
            for new_possibility in &mut new_possibilities {
                new_possibility.extend(inner_inner_facets.clone());
            }
            new_inner_facets.extend(new_possibilities);
        }
        v3_facets.push(new_inner_facets);
    }
    v3_facets
}

// Convert search facets from V3 back to v2
// this is not lossless. (See tests)
pub fn convert_side_types_v2(
    side_types: &HashMap<String, Value>,
    project_type: Option<&str>,
) -> (LegacySideType, LegacySideType) {
    let client_and_server = side_types
        .get("client_and_server")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let singleplayer = side_types
        .get("singleplayer")
        .and_then(|x| x.as_bool())
        .unwrap_or(client_and_server);
    let client_only = side_types
        .get("client_only")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let server_only = side_types
        .get("server_only")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    convert_side_types_v2_bools(
        Some(singleplayer),
        client_only,
        server_only,
        Some(client_and_server),
        project_type,
    )
}

// Client side, server side
pub fn convert_side_types_v2_bools(
    singleplayer: Option<bool>,
    client_only: bool,
    server_only: bool,
    client_and_server: Option<bool>,
    project_type: Option<&str>,
) -> (LegacySideType, LegacySideType) {
    use LegacySideType::{Optional, Required, Unknown, Unsupported};

    match project_type {
        Some("plugin") => (Unsupported, Required),
        Some("datapack") => (Optional, Required),
        Some("shader") => (Required, Unsupported),
        Some("resourcepack") => (Required, Unsupported),
        _ => {
            let singleplayer = singleplayer.or(client_and_server).unwrap_or(false);

            match (singleplayer, client_only, server_only) {
                // Only singleplayer
                (true, false, false) => (Required, Required),

                // Client only and not server only
                (false, true, false) => (Required, Unsupported),
                (true, true, false) => (Required, Unsupported),

                // Server only and not client only
                (false, false, true) => (Unsupported, Required),
                (true, false, true) => (Unsupported, Required),

                // Both server only and client only
                (true, true, true) => (Optional, Optional),
                (false, true, true) => (Optional, Optional),

                // Bad type
                (false, false, false) => (Unknown, Unknown),
            }
        }
    }
}

pub fn capitalize_first(input: &str) -> String {
    let mut result = input.to_owned();
    if let Some(first_char) = result.get_mut(0..1) {
        first_char.make_ascii_uppercase();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::v2::projects::LegacySideType::{Optional, Required, Unsupported};

    #[test]
    fn convert_types() {
        // Converting types from V2 to V3 and back should be idempotent- for certain pairs
        let lossy_pairs = [
            (Optional, Unsupported),
            (Unsupported, Optional),
            (Required, Optional),
            (Optional, Required),
            (Unsupported, Unsupported),
        ];

        for client_side in [Required, Optional, Unsupported] {
            for server_side in [Required, Optional, Unsupported] {
                if lossy_pairs.contains(&(client_side, server_side)) {
                    continue;
                }
                let side_types = convert_side_types_v3(client_side, server_side);
                let (client_side2, server_side2) = convert_side_types_v2(&side_types, None);
                assert_eq!(client_side, client_side2);
                assert_eq!(server_side, server_side2);
            }
        }
    }

    #[test]
    fn convert_facets() {
        let pre_facets = vec![
            // Test combinations of both sides being set
            vec![vec![
                "client_side:required".to_string(),
                "server_side:required".to_string(),
            ]],
            vec![vec![
                "client_side:required".to_string(),
                "server_side:optional".to_string(),
            ]],
            vec![vec![
                "client_side:required".to_string(),
                "server_side:unsupported".to_string(),
            ]],
            vec![vec![
                "client_side:optional".to_string(),
                "server_side:required".to_string(),
            ]],
            vec![vec![
                "client_side:optional".to_string(),
                "server_side:optional".to_string(),
            ]],
            // Test multiple inner facets
            vec![
                vec![
                    "client_side:required".to_string(),
                    "server_side:required".to_string(),
                ],
                vec![
                    "client_side:required".to_string(),
                    "server_side:optional".to_string(),
                ],
            ],
            // Test additional fields
            vec![
                vec![
                    "random_field_test_1".to_string(),
                    "client_side:required".to_string(),
                    "server_side:required".to_string(),
                ],
                vec![
                    "random_field_test_2".to_string(),
                    "client_side:required".to_string(),
                    "server_side:optional".to_string(),
                ],
            ],
            // Test only one facet being set
            vec![vec!["client_side:required".to_string()]],
        ];

        let converted_facets = convert_side_type_facets_v3(pre_facets)
            .into_iter()
            .map(|x| {
                x.into_iter()
                    .map(|mut y| {
                        y.sort();
                        y
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let post_facets = vec![
            vec![vec![
                "singleplayer:true".to_string(),
                "client_and_server:true".to_string(),
                "client_only:false".to_string(),
                "server_only:false".to_string(),
            ]],
            vec![vec![
                "singleplayer:true".to_string(),
                "client_and_server:true".to_string(),
                "client_only:true".to_string(),
                "server_only:false".to_string(),
            ]],
            vec![vec![
                "singleplayer:true".to_string(),
                "client_and_server:true".to_string(),
                "client_only:true".to_string(),
                "server_only:false".to_string(),
            ]],
            vec![vec![
                "singleplayer:true".to_string(),
                "client_and_server:true".to_string(),
                "client_only:false".to_string(),
                "server_only:true".to_string(),
            ]],
            vec![vec![
                "singleplayer:true".to_string(),
                "client_and_server:true".to_string(),
                "client_only:true".to_string(),
                "server_only:true".to_string(),
            ]],
            vec![
                vec![
                    "singleplayer:true".to_string(),
                    "client_and_server:true".to_string(),
                    "client_only:false".to_string(),
                    "server_only:false".to_string(),
                ],
                vec![
                    "singleplayer:true".to_string(),
                    "client_and_server:true".to_string(),
                    "client_only:true".to_string(),
                    "server_only:false".to_string(),
                ],
            ],
            vec![
                vec![
                    "random_field_test_1".to_string(),
                    "singleplayer:true".to_string(),
                    "client_and_server:true".to_string(),
                    "client_only:false".to_string(),
                    "server_only:false".to_string(),
                ],
                vec![
                    "random_field_test_2".to_string(),
                    "singleplayer:true".to_string(),
                    "client_and_server:true".to_string(),
                    "client_only:true".to_string(),
                    "server_only:false".to_string(),
                ],
            ],
            // Test only one facet being set
            // Iterates over all possible side types
            vec![
                // C: Required, S: Required
                vec![
                    "singleplayer:true".to_string(),
                    "client_and_server:true".to_string(),
                    "client_only:false".to_string(),
                    "server_only:false".to_string(),
                ],
                // C: Required, S: Optional
                vec![
                    "singleplayer:true".to_string(),
                    "client_and_server:true".to_string(),
                    "client_only:true".to_string(),
                    "server_only:false".to_string(),
                ],
                // C: Required, S: Unsupported
                vec![
                    "singleplayer:true".to_string(),
                    "client_and_server:true".to_string(),
                    "client_only:true".to_string(),
                    "server_only:false".to_string(),
                ],
            ],
        ]
        .into_iter()
        .map(|x| {
            x.into_iter()
                .map(|mut y| {
                    y.sort();
                    y
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

        assert_eq!(converted_facets, post_facets);
    }
}
