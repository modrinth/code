#![allow(dead_code)]

use super::{
    actix::AppendsMultipart,
    asserts::assert_status,
    database::{MOD_USER_PAT, USER_USER_PAT},
    environment::LocalService,
    request_data::ProjectCreationRequestData,
};
use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use labrinth::models::{
    notifications::Notification,
    projects::{Project, Version},
};
use serde_json::json;
use std::rc::Rc;

pub struct ApiV2 {
    pub test_app: Rc<Box<dyn LocalService>>,
}

impl ApiV2 {
    pub async fn call(&self, req: actix_http::Request) -> ServiceResponse {
        self.test_app.call(req).await.unwrap()
    }

    pub async fn add_public_project(
        &self,
        creation_data: ProjectCreationRequestData,
    ) -> (Project, Version) {
        // Add a project.
        let req = TestRequest::post()
            .uri("/v2/project")
            .append_header(("Authorization", USER_USER_PAT))
            .set_multipart(creation_data.segment_data)
            .to_request();
        let resp = self.call(req).await;
        assert_status(resp, StatusCode::OK);

        // Approve as a moderator.
        let req = TestRequest::patch()
            .uri(&format!("/v2/project/{}", creation_data.slug))
            .append_header(("Authorization", MOD_USER_PAT))
            .set_json(json!(
                {
                    "status": "approved"
                }
            ))
            .to_request();
        let resp = self.call(req).await;
        assert_status(resp, StatusCode::NO_CONTENT);

        let project = self
            .get_project_deserialized(&creation_data.slug, USER_USER_PAT)
            .await;

        // Get project's versions
        let req = TestRequest::get()
            .uri(&format!("/v2/project/{}/version", creation_data.slug))
            .append_header(("Authorization", USER_USER_PAT))
            .to_request();
        let resp = self.call(req).await;
        let versions: Vec<Version> = test::read_body_json(resp).await;
        let version = versions.into_iter().next().unwrap();

        (project, version)
    }

    pub async fn remove_project(&self, project_slug_or_id: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/project/{project_slug_or_id}"))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        assert_eq!(resp.status(), 204);
        resp
    }

    pub async fn get_project_deserialized(&self, slug: &str, pat: &str) -> Project {
        let req = TestRequest::get()
            .uri(&format!("/v2/project/{slug}"))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        test::read_body_json(resp).await
    }

    pub async fn get_user_projects_deserialized(
        &self,
        user_id_or_username: &str,
        pat: &str,
    ) -> Vec<Project> {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/user/{}/projects", user_id_or_username))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn add_user_to_team(
        &self,
        team_id: &str,
        user_id: &str,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!("/v2/team/{team_id}/members"))
            .append_header(("Authorization", pat))
            .set_json(json!( {
                "user_id": user_id
            }))
            .to_request();
        self.call(req).await
    }

    pub async fn join_team(&self, team_id: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!("/v2/team/{team_id}/join"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn remove_from_team(
        &self,
        team_id: &str,
        user_id: &str,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/team/{team_id}/members/{user_id}"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_user_notifications_deserialized(
        &self,
        user_id: &str,
        pat: &str,
    ) -> Vec<Notification> {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/user/{user_id}/notifications"))
            .append_header(("Authorization", pat))
            .to_request();
        let resp = self.call(req).await;
        test::read_body_json(resp).await
    }

    pub async fn mark_notification_read(
        &self,
        notification_id: &str,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/notification/{notification_id}"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn delete_notification(&self, notification_id: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/notification/{notification_id}"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }
}
