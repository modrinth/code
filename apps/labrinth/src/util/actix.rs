use actix_web::test::TestRequest;
use bytes::{Bytes, BytesMut};

// Multipart functionality for actix
// Primarily for testing or some implementations of route-redirection
// (actix-test does not innately support multipart)
#[derive(Debug, Clone)]
pub struct MultipartSegment {
    pub name: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub data: MultipartSegmentData,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum MultipartSegmentData {
    Text(String),
    Binary(Vec<u8>),
}

pub trait AppendsMultipart {
    fn set_multipart(
        self,
        data: impl IntoIterator<Item = MultipartSegment>,
    ) -> Self;
}

impl AppendsMultipart for TestRequest {
    fn set_multipart(
        self,
        data: impl IntoIterator<Item = MultipartSegment>,
    ) -> Self {
        let (boundary, payload) = generate_multipart(data);
        self.append_header((
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        ))
        .set_payload(payload)
    }
}

pub fn generate_multipart(
    data: impl IntoIterator<Item = MultipartSegment>,
) -> (String, Bytes) {
    let mut boundary: String = String::from("----WebKitFormBoundary");
    boundary.push_str(&rand::random::<u64>().to_string());
    boundary.push_str(&rand::random::<u64>().to_string());
    boundary.push_str(&rand::random::<u64>().to_string());

    let mut payload = BytesMut::new();

    for segment in data {
        payload.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"",
                boundary = boundary,
                name = segment.name
            )
            .as_bytes(),
        );

        if let Some(filename) = &segment.filename {
            payload.extend_from_slice(
                format!("; filename=\"{filename}\"").as_bytes(),
            );
        }
        if let Some(content_type) = &segment.content_type {
            payload.extend_from_slice(
                format!("\r\nContent-Type: {content_type}").as_bytes(),
            );
        }
        payload.extend_from_slice(b"\r\n\r\n");

        match &segment.data {
            MultipartSegmentData::Text(text) => {
                payload.extend_from_slice(text.as_bytes());
            }
            MultipartSegmentData::Binary(binary) => {
                payload.extend_from_slice(binary);
            }
        }
        payload.extend_from_slice(b"\r\n");
    }
    payload.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    (boundary, Bytes::from(payload))
}
