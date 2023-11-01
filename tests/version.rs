use crate::common::{asserts::assert_status, get_json_val_str};
use actix_http::StatusCode;
use common::{
    asserts::assert_version_ids, database::USER_USER_PAT, environment::with_test_environment,
};

mod common;

#[actix_rt::test]
async fn can_create_version_with_ordering() {
    with_test_environment(|env| async move {
        let alpha_project_id = env.dummy.as_ref().unwrap().project_alpha.project_id.clone();

        let new_version_id = get_json_val_str(
            env.v2
                .create_default_version(&alpha_project_id, Some(1), USER_USER_PAT)
                .await
                .id,
        );

        let versions = env
            .v2
            .get_versions(vec![new_version_id.clone()], USER_USER_PAT)
            .await;
        assert_eq!(versions[0].ordering, Some(1));
    })
    .await;
}

#[actix_rt::test]
async fn edit_version_ordering_works() {
    with_test_environment(|env| async move {
        let alpha_version_id = env.dummy.as_ref().unwrap().project_alpha.version_id.clone();

        let resp = env
            .v2
            .edit_version_ordering(&alpha_version_id, Some(10), USER_USER_PAT)
            .await;
        assert_status(&resp, StatusCode::NO_CONTENT);

        let versions = env
            .v2
            .get_versions(vec![alpha_version_id.clone()], USER_USER_PAT)
            .await;
        assert_eq!(versions[0].ordering, Some(10));
    })
    .await;
}

#[actix_rt::test]
async fn version_ordering_for_specified_orderings_orders_lower_order_first() {
    with_test_environment(|env| async move {
        let alpha_project_id = env.dummy.as_ref().unwrap().project_alpha.project_id.clone();
        let alpha_version_id = env.dummy.as_ref().unwrap().project_alpha.version_id.clone();
        let new_version_id = get_json_val_str(
            env.v2
                .create_default_version(&alpha_project_id, Some(1), USER_USER_PAT)
                .await
                .id,
        );
        env.v2
            .edit_version_ordering(&alpha_version_id, Some(10), USER_USER_PAT)
            .await;

        let versions = env
            .v2
            .get_versions(
                vec![alpha_version_id.clone(), new_version_id.clone()],
                USER_USER_PAT,
            )
            .await;
        assert_version_ids(&versions, vec![new_version_id, alpha_version_id]);
    })
    .await;
}

#[actix_rt::test]
async fn version_ordering_when_unspecified_orders_oldest_first() {
    with_test_environment(|env| async move {
        let alpha_project_id = &env.dummy.as_ref().unwrap().project_alpha.project_id.clone();
        let alpha_version_id = env.dummy.as_ref().unwrap().project_alpha.version_id.clone();
        let new_version_id = get_json_val_str(
            env.v2
                .create_default_version(&alpha_project_id, None, USER_USER_PAT)
                .await
                .id,
        );

        let versions = env
            .v2
            .get_versions(
                vec![alpha_version_id.clone(), new_version_id.clone()],
                USER_USER_PAT,
            )
            .await;
        assert_version_ids(&versions, vec![alpha_version_id, new_version_id]);
    })
    .await
}

#[actix_rt::test]
async fn version_ordering_when_specified_orders_specified_before_unspecified() {
    with_test_environment(|env| async move {
        let alpha_project_id = &env.dummy.as_ref().unwrap().project_alpha.project_id.clone();
        let alpha_version_id = env.dummy.as_ref().unwrap().project_alpha.version_id.clone();
        let new_version_id = get_json_val_str(
            env.v2
                .create_default_version(&alpha_project_id, Some(10000), USER_USER_PAT)
                .await
                .id,
        );
        env.v2
            .edit_version_ordering(&alpha_version_id, None, USER_USER_PAT)
            .await;

        let versions = env
            .v2
            .get_versions(
                vec![alpha_version_id.clone(), new_version_id.clone()],
                USER_USER_PAT,
            )
            .await;
        assert_version_ids(&versions, vec![new_version_id, alpha_version_id]);
    })
    .await;
}
