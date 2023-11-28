use crate::common::api_v2::request_data::get_public_project_creation_data;
use crate::common::api_v2::request_data::get_public_version_creation_data;
use crate::common::api_v2::ApiV2;
use crate::common::dummy_data::TestFile;
use crate::common::environment::with_test_environment;
use crate::common::environment::TestEnvironment;
use crate::common::scopes::ScopeTest;
use actix_web::test;
use labrinth::models::ids::base62_impl::parse_base62;
use labrinth::models::pats::Scopes;
use labrinth::models::projects::ProjectId;
use labrinth::util::actix::AppendsMultipart;

// Project version creation scopes
#[actix_rt::test]
pub async fn project_version_create_scopes() {
    with_test_environment(None, |test_env: TestEnvironment<ApiV2>| async move {
        // Create project
        let create_project = Scopes::PROJECT_CREATE;

        let req_gen = || {
            let creation_data =
                get_public_project_creation_data("demo", Some(TestFile::BasicMod), None);
            test::TestRequest::post()
                .uri("/v2/project")
                .set_multipart(creation_data.segment_data)
        };
        let (_, success) = ScopeTest::new(&test_env)
            .test(req_gen, create_project)
            .await
            .unwrap();
        let project_id = success["id"].as_str().unwrap();
        let project_id = ProjectId(parse_base62(project_id).unwrap());

        // Add version to project
        let create_version = Scopes::VERSION_CREATE;
        let req_gen = || {
            let creation_data = get_public_version_creation_data(
                project_id,
                "1.2.3.4",
                TestFile::BasicModDifferent,
                None,
                None,
            );
            test::TestRequest::post()
                .uri("/v2/version")
                .set_multipart(creation_data.segment_data)
        };
        ScopeTest::new(&test_env)
            .test(req_gen, create_version)
            .await
            .unwrap();
    })
    .await;
}
