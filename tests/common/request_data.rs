use serde_json::json;

use super::{actix::MultipartSegment, dummy_data::DummyJarFile};
use crate::common::actix::MultipartSegmentData;

pub struct ProjectCreationRequestData {
    pub slug: String,
    pub jar: DummyJarFile,
    pub segment_data: Vec<MultipartSegment>,
}

pub fn get_public_project_creation_data(
    slug: &str,
    jar: DummyJarFile,
) -> ProjectCreationRequestData {
    let json_data = json!(
        {
            "title": format!("Test Project {slug}"),
            "slug": slug,
            "description": "A dummy project for testing with.",
            "body": "This project is approved, and versions are listed.",
            "client_side": "required",
            "server_side": "optional",
            "initial_versions": [{
                "file_parts": [jar.filename()],
                "version_number": "1.2.3",
                "version_title": "start",
                "dependencies": [],
                "game_versions": ["1.20.1"] ,
                "release_channel": "release",
                "loaders": ["fabric"],
                "featured": true
            }],
            "categories": [],
            "license_id": "MIT"
        }
    );

    // Basic json
    let json_segment = MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
    };

    // Basic file
    let file_segment = MultipartSegment {
        name: jar.filename(),
        filename: Some(jar.filename()),
        content_type: Some("application/java-archive".to_string()),
        data: MultipartSegmentData::Binary(jar.bytes()),
    };

    ProjectCreationRequestData {
        slug: slug.to_string(),
        jar,
        segment_data: vec![json_segment.clone(), file_segment.clone()],
    }
}
