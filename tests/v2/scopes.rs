use crate::common::environment::TestEnvironment;
use crate::common::scopes::ScopeTest;
use actix_web::test;
use labrinth::models::pats::Scopes;
use labrinth::util::actix::AppendsMultipart;
use labrinth::util::actix::MultipartSegment;
use labrinth::util::actix::MultipartSegmentData;

use serde_json::json;
// Project version creation scopes
#[actix_rt::test]
pub async fn project_version_create_scopes() {
    let test_env = TestEnvironment::build(None).await;

    // Create project
    let create_project = Scopes::PROJECT_CREATE;
    let json_data = json!(
        {
            "title": "Test_Add_Project project",
            "slug": "demo",
            "description": "Example description.",
            "body": "Example body.",
            "initial_versions": [{
                "file_parts": ["basic-mod.jar"],
                "version_number": "1.2.3",
                "version_title": "start",
                "dependencies": [],
                "game_versions": ["1.20.1"] ,
                "client_side": "required",
                "server_side": "optional",
                "release_channel": "release",
                "loaders": ["fabric"],
                "featured": true
            }],
            "categories": [],
            "license_id": "MIT"
        }
    );
    let json_segment = MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
    };
    let file_segment = MultipartSegment {
        name: "basic-mod.jar".to_string(),
        filename: Some("basic-mod.jar".to_string()),
        content_type: Some("application/java-archive".to_string()),
        data: MultipartSegmentData::Binary(
            include_bytes!("../../tests/files/basic-mod.jar").to_vec(),
        ),
    };

    let req_gen = || {
        test::TestRequest::post()
            .uri("/v3/project")
            .set_multipart(vec![json_segment.clone(), file_segment.clone()])
    };
    let (_, success) = ScopeTest::new(&test_env)
        .test(req_gen, create_project)
        .await
        .unwrap();
    let project_id = success["id"].as_str().unwrap();

    // Add version to project
    let create_version = Scopes::VERSION_CREATE;
    let json_data = json!(
            {
                "project_id": project_id,
                "file_parts": ["basic-mod-different.jar"],
                "version_number": "1.2.3.4",
                "version_title": "start",
                "dependencies": [],
                "game_versions": ["1.20.1"] ,
                "client_side": "required",
                "server_side": "optional",
                "release_channel": "release",
                "loaders": ["fabric"],
                "featured": true
            }
    );
    let json_segment = MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
    };
    let file_segment = MultipartSegment {
        name: "basic-mod-different.jar".to_string(),
        filename: Some("basic-mod.jar".to_string()),
        content_type: Some("application/java-archive".to_string()),
        data: MultipartSegmentData::Binary(
            include_bytes!("../../tests/files/basic-mod-different.jar").to_vec(),
        ),
    };

    let req_gen = || {
        test::TestRequest::post()
            .uri("/v3/version")
            .set_multipart(vec![json_segment.clone(), file_segment.clone()])
    };
    ScopeTest::new(&test_env)
        .test(req_gen, create_version)
        .await
        .unwrap();

    // Cleanup test db
    test_env.cleanup().await;
}
