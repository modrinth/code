use crate::common::api_common::ApiProject;
use crate::common::api_common::ApiVersion;
use crate::common::api_v2::ApiV2;
use crate::common::api_v2::request_data::get_public_project_creation_data;
use crate::common::dummy_data::TestFile;
use crate::common::environment::TestEnvironment;
use crate::common::environment::with_test_environment;
use crate::common::scopes::ScopeTest;
use ariadne::ids::base62_impl::parse_base62;
use labrinth::models::ids::ProjectId;
use labrinth::models::pats::Scopes;

// Project version creation scopes
#[actix_rt::test]
pub async fn project_version_create_scopes() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let api = &test_env.api;
            // Create project
            let create_project = Scopes::PROJECT_CREATE;

            let req_gen = |pat: Option<String>| async move {
                let creation_data = get_public_project_creation_data(
                    "demo",
                    Some(TestFile::BasicMod),
                    None,
                );
                api.create_project(creation_data, pat.as_deref()).await
            };
            let (_, success) = ScopeTest::new(&test_env)
                .test(req_gen, create_project)
                .await
                .unwrap();
            let project_id = success["id"].as_str().unwrap();
            let project_id = ProjectId(parse_base62(project_id).unwrap());

            // Add version to project
            let create_version = Scopes::VERSION_CREATE;
            let req_gen = |pat: Option<String>| async move {
                api.add_public_version(
                    project_id,
                    "1.2.3.4",
                    TestFile::BasicModDifferent,
                    None,
                    None,
                    pat.as_deref(),
                )
                .await
            };
            ScopeTest::new(&test_env)
                .test(req_gen, create_version)
                .await
                .unwrap();
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn project_version_reads_scopes() {
    with_test_environment(
        None,
        |_test_env: TestEnvironment<ApiV2>| async move {
            // let api = &test_env.api;
            // let beta_file_hash = &test_env.dummy.project_beta.file_hash;

            // let read_version = Scopes::VERSION_READ;

            // Update individual version file
            // TODO: This scope currently fails still as the 'version' field of QueryProject only allows public versions.
            // TODO: This will be fixed when the 'extracts_versions' PR is merged.
            // let req_gen = |pat : Option<String>| async move {
            //     api.update_individual_files("sha1", vec![
            //         FileUpdateData {
            //             hash: beta_file_hash.clone(),
            //             loaders: None,
            //             game_versions: None,
            //             version_types: None
            //         }
            //     ], pat.as_deref())
            //         .await
            // };
            // let (failure, success) = ScopeTest::new(&test_env).with_failure_code(200).test(req_gen, read_version).await.unwrap();
            // assert!(!failure.as_object().unwrap().contains_key(beta_file_hash));
            // assert!(success.as_object().unwrap().contains_key(beta_file_hash));
        },
    )
    .await;
}
