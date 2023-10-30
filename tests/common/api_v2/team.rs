use actix_http::StatusCode;
use actix_web::{dev::ServiceResponse, test};
use labrinth::models::{
    notifications::Notification,
    teams::{OrganizationPermissions, ProjectPermissions, TeamMember},
};
use serde_json::json;

use crate::common::asserts::assert_status;

use super::ApiV2;

impl ApiV2 {
    pub async fn get_team_members(&self, id_or_title: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/team/{id_or_title}/members"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_team_members_deserialized(
        &self,
        id_or_title: &str,
        pat: &str,
    ) -> Vec<TeamMember> {
        let resp = self.get_team_members(id_or_title, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_project_members(&self, id_or_title: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/project/{id_or_title}/members"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_project_members_deserialized(
        &self,
        id_or_title: &str,
        pat: &str,
    ) -> Vec<TeamMember> {
        let resp = self.get_project_members(id_or_title, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_organization_members(&self, id_or_title: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/organization/{id_or_title}/members"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_organization_members_deserialized(
        &self,
        id_or_title: &str,
        pat: &str,
    ) -> Vec<TeamMember> {
        let resp = self.get_organization_members(id_or_title, pat).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
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

    pub async fn edit_team_member(
        &self,
        team_id: &str,
        user_id: &str,
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/team/{team_id}/members/{user_id}"))
            .append_header(("Authorization", pat))
            .set_json(patch)
            .to_request();
        self.call(req).await
    }

    pub async fn transfer_team_ownership(
        &self,
        team_id: &str,
        user_id: &str,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/team/{team_id}/owner"))
            .append_header(("Authorization", pat))
            .set_json(json!({
                "user_id": user_id,
            }))
            .to_request();
        self.call(req).await
    }

    pub async fn get_user_notifications(&self, user_id: &str, pat: &str) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/user/{user_id}/notifications"))
            .append_header(("Authorization", pat))
            .to_request();
        self.call(req).await
    }

    pub async fn get_user_notifications_deserialized(
        &self,
        user_id: &str,
        pat: &str,
    ) -> Vec<Notification> {
        let resp = self.get_user_notifications(user_id, pat).await;
        assert_status(&resp, StatusCode::OK);
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
    pub async fn add_user_to_team(
        &self,
        team_id: &str,
        user_id: &str,
        project_permissions: Option<ProjectPermissions>,
        organization_permissions: Option<OrganizationPermissions>,
        pat: &str,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!("/v2/team/{team_id}/members"))
            .append_header(("Authorization", pat))
            .set_json(json!( {
                "user_id": user_id,
                "permissions" : project_permissions.map(|p| p.bits()).unwrap_or_default(),
                "organization_permissions" : organization_permissions.map(|p| p.bits()),
            }))
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
