#![allow(dead_code)]
use serde_json::json;

use super::{
    actix::MultipartSegment,
    dummy_data::{DummyImage, DummyJarFile},
};
use crate::common::actix::MultipartSegmentData;

pub struct ProjectCreationRequestData {
    pub slug: String,
    pub jar: Option<DummyJarFile>,
    pub segment_data: Vec<MultipartSegment>,
}

pub struct ImageData {
    pub filename: String,
    pub extension: String,
    pub icon: Vec<u8>,
}

pub fn get_public_project_creation_data(
    slug: &str,
    version_jar: Option<DummyJarFile>,
) -> ProjectCreationRequestData {
    let initial_versions = if let Some(ref jar) = version_jar {
        json!([{
            "file_parts": [jar.filename()],
            "version_number": "1.2.3",
            "version_title": "start",
            "dependencies": [],
            "game_versions": ["1.20.1"] ,
            "release_channel": "release",
            "loaders": ["fabric"],
            "featured": true
        }])
    } else {
        json!([])
    };

    let is_draft = version_jar.is_none();

    let json_data = json!(
        {
            "title": format!("Test Project {slug}"),
            "slug": slug,
            "description": "A dummy project for testing with.",
            "body": "This project is approved, and versions are listed.",
            "client_side": "required",
            "server_side": "optional",
            "initial_versions": initial_versions,
            "is_draft": is_draft,
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

    let segment_data = if let Some(ref jar) = version_jar {
        // Basic file
        let file_segment = MultipartSegment {
            name: jar.filename(),
            filename: Some(jar.filename()),
            content_type: Some("application/java-archive".to_string()),
            data: MultipartSegmentData::Binary(jar.bytes()),
        };

        vec![json_segment.clone(), file_segment]
    } else {
        vec![json_segment.clone()]
    };

    ProjectCreationRequestData {
        slug: slug.to_string(),
        jar: version_jar,
        segment_data,
    }
}

pub fn get_icon_data(dummy_icon: DummyImage) -> ImageData {
    ImageData {
        filename: dummy_icon.filename(),
        extension: dummy_icon.extension(),
        icon: dummy_icon.bytes(),
    }
}
