use crate::common::api_common::{ApiProject, ApiTeams};
use actix_web::test;
use common::dummy_data::TestFile;
use common::{
    database::{FRIEND_USER_ID, FRIEND_USER_PAT, USER_USER_ID, USER_USER_PAT},
    environment::{
        TestEnvironment, with_test_environment, with_test_environment_all,
    },
};
use labrinth::test::api_v3::ApiV3;

pub mod common;

// user GET (permissions, different users)
// users GET
// user auth
// user projects get
// user collections get
// patch user
// patch user icon
// user follows

#[actix_rt::test]
pub async fn search_users_returns_compact_prefix_matches_with_exact_first() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            sqlx::query(
                "
                INSERT INTO users (id, username, email, role)
                VALUES
                    (1000, 'userland', 'userland@modrinth.com', 'developer'),
                    (1001, 'useful', 'useful@modrinth.com', 'developer'),
                    (1002, 'Useless', 'useless@modrinth.com', 'developer')
                ",
            )
            .execute(&*test_env.db.pool)
            .await
            .unwrap();

            let req = test::TestRequest::get()
                .uri("/v3/users/search?query=user")
                .to_request();
            let resp = test_env.call(req).await;
            assert_status!(&resp, actix_http::StatusCode::OK);

            let users: Vec<serde_json::Value> =
                test::read_body_json(resp).await;
            assert_eq!(users.len(), 2);
            assert_eq!(users[0]["username"], "User");
            assert_eq!(users[1]["username"], "userland");
            assert!(users.iter().all(|user| {
                user.as_object().is_some_and(|object| {
                    object.len() == 3 && object.contains_key("avatar_url")
                })
            }));
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn search_users_escapes_wildcards_and_limits_results() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            for i in 0..30 {
                sqlx::query(
                    "
                    INSERT INTO users (id, username, email, role)
                    VALUES ($1, $2, $3, 'developer')
                    ",
                )
                .bind(2000 + i)
                .bind(format!("prefix{i:02}"))
                .bind(format!("prefix{i:02}@modrinth.com"))
                .execute(&*test_env.db.pool)
                .await
                .unwrap();
            }

            sqlx::query(
                "
                INSERT INTO users (id, username, email, role)
                VALUES (2100, 'prefix_under_score', 'prefix_under_score@modrinth.com', 'developer')
                ",
            )
            .execute(&*test_env.db.pool)
            .await
            .unwrap();

            let req = test::TestRequest::get()
                .uri("/v3/users/search?query=prefix")
                .to_request();
            let resp = test_env.call(req).await;
            assert_status!(&resp, actix_http::StatusCode::OK);

            let users: Vec<serde_json::Value> =
                test::read_body_json(resp).await;
            assert_eq!(users.len(), 25);
            assert!(users.iter().all(|user| {
                user["username"]
                    .as_str()
                    .is_some_and(|username| username.starts_with("prefix"))
            }));

            let req = test::TestRequest::get()
                .uri("/v3/users/search?query=%25")
                .to_request();
            let resp = test_env.call(req).await;
            assert_status!(&resp, actix_http::StatusCode::OK);

            let users: Vec<serde_json::Value> =
                test::read_body_json(resp).await;
            assert!(users.is_empty());

            let req = test::TestRequest::get()
                .uri("/v3/users/search?query=prefix_")
                .to_request();
            let resp = test_env.call(req).await;
            assert_status!(&resp, actix_http::StatusCode::OK);

            let users: Vec<serde_json::Value> =
                test::read_body_json(resp).await;
            assert_eq!(users.len(), 1);
            assert_eq!(users[0]["username"], "prefix_under_score");

            let req = test::TestRequest::get()
                .uri("/v3/users/search?query=%20%20")
                .to_request();
            let resp = test_env.call(req).await;
            assert_status!(&resp, actix_http::StatusCode::OK);

            let users: Vec<serde_json::Value> =
                test::read_body_json(resp).await;
            assert!(users.is_empty());
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn get_user_projects_after_creating_project_returns_new_project() {
    with_test_environment_all(None, |test_env| async move {
        let api = test_env.api;
        api.get_user_projects_deserialized_common(USER_USER_ID, USER_USER_PAT)
            .await;

        let (project, _) = api
            .add_public_project(
                "slug",
                Some(TestFile::BasicMod),
                None,
                USER_USER_PAT,
            )
            .await;

        let resp_projects = api
            .get_user_projects_deserialized_common(USER_USER_ID, USER_USER_PAT)
            .await;
        assert!(resp_projects.iter().any(|p| p.id == project.id));
    })
    .await;
}

#[actix_rt::test]
pub async fn get_user_projects_after_deleting_project_shows_removal() {
    with_test_environment_all(None, |test_env| async move {
        let api = test_env.api;
        let (project, _) = api
            .add_public_project(
                "iota",
                Some(TestFile::BasicMod),
                None,
                USER_USER_PAT,
            )
            .await;
        api.get_user_projects_deserialized_common(USER_USER_ID, USER_USER_PAT)
            .await;

        api.remove_project(project.slug.as_ref().unwrap(), USER_USER_PAT)
            .await;

        let resp_projects = api
            .get_user_projects_deserialized_common(USER_USER_ID, USER_USER_PAT)
            .await;
        assert!(!resp_projects.iter().any(|p| p.id == project.id));
    })
    .await;
}

#[actix_rt::test]
pub async fn get_user_projects_after_joining_team_shows_team_projects() {
    with_test_environment_all(None, |test_env| async move {
        let alpha_team_id = &test_env.dummy.project_alpha.team_id;
        let alpha_project_id = &test_env.dummy.project_alpha.project_id;
        let api = test_env.api;
        api.get_user_projects_deserialized_common(
            FRIEND_USER_ID,
            FRIEND_USER_PAT,
        )
        .await;

        api.add_user_to_team(
            alpha_team_id,
            FRIEND_USER_ID,
            None,
            None,
            USER_USER_PAT,
        )
        .await;
        api.join_team(alpha_team_id, FRIEND_USER_PAT).await;

        let projects = api
            .get_user_projects_deserialized_common(
                FRIEND_USER_ID,
                FRIEND_USER_PAT,
            )
            .await;
        assert!(
            projects
                .iter()
                .any(|p| p.id.to_string() == *alpha_project_id)
        );
    })
    .await;
}

#[actix_rt::test]
pub async fn get_user_projects_after_leaving_team_shows_no_team_projects() {
    with_test_environment_all(None, |test_env| async move {
        let alpha_team_id = &test_env.dummy.project_alpha.team_id;
        let alpha_project_id = &test_env.dummy.project_alpha.project_id;
        let api = test_env.api;
        api.add_user_to_team(
            alpha_team_id,
            FRIEND_USER_ID,
            None,
            None,
            USER_USER_PAT,
        )
        .await;
        api.join_team(alpha_team_id, FRIEND_USER_PAT).await;
        api.get_user_projects_deserialized_common(
            FRIEND_USER_ID,
            FRIEND_USER_PAT,
        )
        .await;

        api.remove_from_team(alpha_team_id, FRIEND_USER_ID, USER_USER_PAT)
            .await;

        let projects = api
            .get_user_projects_deserialized_common(
                FRIEND_USER_ID,
                FRIEND_USER_PAT,
            )
            .await;
        assert!(
            !projects
                .iter()
                .any(|p| p.id.to_string() == *alpha_project_id)
        );
    })
    .await;
}
