#![allow(dead_code)]
use serde_json::json;

use crate::common::{
    api_common::request_data::{
        ProjectCreationRequestData, VersionCreationRequestData,
    },
    dummy_data::TestFile,
};
use labrinth::models::ids::ProjectId;
use labrinth::util::actix::{MultipartSegment, MultipartSegmentData};

pub fn get_public_project_creation_data(
    slug: &str,
    version_jar: Option<TestFile>,
    modify_json: Option<json_patch::Patch>,
) -> ProjectCreationRequestData {
    let mut json_data =
        get_public_project_creation_data_json(slug, version_jar.as_ref());
    if let Some(modify_json) = modify_json {
        json_patch::patch(&mut json_data, &modify_json).unwrap();
    }
    let multipart_data =
        get_public_creation_data_multipart(&json_data, version_jar.as_ref());
    ProjectCreationRequestData {
        slug: slug.to_string(),
        jar: version_jar,
        segment_data: multipart_data,
    }
}

pub fn get_public_version_creation_data(
    project_id: ProjectId,
    version_number: &str,
    version_jar: TestFile,
    ordering: Option<i32>,
    modify_json: Option<json_patch::Patch>,
) -> VersionCreationRequestData {
    let mut json_data = get_public_version_creation_data_json(
        version_number,
        ordering,
        &version_jar,
    );
    json_data["project_id"] = json!(project_id);
    if let Some(modify_json) = modify_json {
        json_patch::patch(&mut json_data, &modify_json).unwrap();
    }
    let multipart_data =
        get_public_creation_data_multipart(&json_data, Some(&version_jar));
    VersionCreationRequestData {
        version: version_number.to_string(),
        jar: Some(version_jar),
        segment_data: multipart_data,
    }
}

pub fn get_public_version_creation_data_json(
    version_number: &str,
    ordering: Option<i32>,
    version_jar: &TestFile,
) -> serde_json::Value {
    let mut j = json!({
        "file_parts": [version_jar.filename()],
        "version_number": version_number,
        "version_title": "start",
        "dependencies": [],
        "game_versions": ["1.20.1"] ,
        "release_channel": "release",
        "loaders": ["fabric"],
        "featured": true
    });
    if let Some(ordering) = ordering {
        j["ordering"] = json!(ordering);
    }
    j
}

pub fn get_public_project_creation_data_json(
    slug: &str,
    version_jar: Option<&TestFile>,
) -> serde_json::Value {
    let initial_versions = if let Some(jar) = version_jar {
        json!([get_public_version_creation_data_json("1.2.3", None, jar)])
    } else {
        json!([])
    };

    let is_draft = version_jar.is_none();
    json!(
        {
            "title": format!("Test Project {slug}"),
            "slug": slug,
            "project_type": version_jar.as_ref().map(|f| f.project_type()).unwrap_or("mod".to_string()),
            "description": "A dummy project for testing with.",
            "body": "This project is approved, and versions are listed.",
            "client_side": "required",
            "server_side": "optional",
            "initial_versions": initial_versions,
            "is_draft": is_draft,
            "categories": [],
            "license_id": "MIT",
        }
    )
}

pub fn get_public_creation_data_multipart(
    json_data: &serde_json::Value,
    version_jar: Option<&TestFile>,
) -> Vec<MultipartSegment> {
    // Basic json
    let json_segment = MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: MultipartSegmentData::Text(
            serde_json::to_string(json_data).unwrap(),
        ),
    };

    if let Some(jar) = version_jar {
        // Basic file
        let file_segment = MultipartSegment {
            name: jar.filename(),
            filename: Some(jar.filename()),
            content_type: Some("application/java-archive".to_string()),
            data: MultipartSegmentData::Binary(jar.bytes()),
        };

        vec![json_segment, file_segment]
    } else {
        vec![json_segment]
    }
}
