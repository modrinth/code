use actix_http::StatusCode;
use actix_web::{dev::ServiceResponse, test};
use async_trait::async_trait;
use labrinth::models::{
    teams::{OrganizationPermissions, ProjectPermissions},
    v2::{notifications::LegacyNotification, teams::LegacyTeamMember},
};
use serde_json::json;

use crate::{
    assert_status,
    common::api_common::{
        Api, ApiTeams, AppendsOptionalPat,
        models::{CommonNotification, CommonTeamMember},
    },
};

use super::ApiV2;

impl ApiV2 {
    pub async fn get_organization_members_deserialized(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> Vec<LegacyTeamMember> {
        let resp = self.get_organization_members(id_or_title, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_team_members_deserialized(
        &self,
        team_id: &str,
        pat: Option<&str>,
    ) -> Vec<LegacyTeamMember> {
        let resp = self.get_team_members(team_id, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_user_notifications_deserialized(
        &self,
        user_id: &str,
        pat: Option<&str>,
    ) -> Vec<LegacyNotification> {
        let resp = self.get_user_notifications(user_id, pat).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }
}

#[async_trait(?Send)]
impl ApiTeams for ApiV2 {
    async fn get_team_members(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/team/{id_or_title}/members"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_team_members_deserialized_common(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> Vec<CommonTeamMember> {
        let resp = self.get_team_members(id_or_title, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: Vec<LegacyTeamMember> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn get_teams_members(
        &self,
        ids_or_titles: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let ids_or_titles = serde_json::to_string(ids_or_titles).unwrap();
        let req = test::TestRequest::get()
            .uri(&format!(
                "/v2/teams?ids={}",
                urlencoding::encode(&ids_or_titles)
            ))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_project_members(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/project/{id_or_title}/members"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_project_members_deserialized_common(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> Vec<CommonTeamMember> {
        let resp = self.get_project_members(id_or_title, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: Vec<LegacyTeamMember> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn get_organization_members(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/organization/{id_or_title}/members"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_organization_members_deserialized_common(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> Vec<CommonTeamMember> {
        let resp = self.get_organization_members(id_or_title, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: Vec<LegacyTeamMember> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn join_team(
        &self,
        team_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!("/v2/team/{team_id}/join"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn remove_from_team(
        &self,
        team_id: &str,
        user_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/team/{team_id}/members/{user_id}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn edit_team_member(
        &self,
        team_id: &str,
        user_id: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/team/{team_id}/members/{user_id}"))
            .append_pat(pat)
            .set_json(patch)
            .to_request();
        self.call(req).await
    }

    async fn transfer_team_ownership(
        &self,
        team_id: &str,
        user_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/team/{team_id}/owner"))
            .append_pat(pat)
            .set_json(json!({
                "user_id": user_id,
            }))
            .to_request();
        self.call(req).await
    }

    async fn get_user_notifications(
        &self,
        user_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/user/{user_id}/notifications"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_user_notifications_deserialized_common(
        &self,
        user_id: &str,
        pat: Option<&str>,
    ) -> Vec<CommonNotification> {
        let resp = self.get_user_notifications(user_id, pat).await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: Vec<LegacyNotification> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn get_notification(
        &self,
        notification_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v2/notification/{notification_id}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_notifications(
        &self,
        notification_ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let notification_ids = serde_json::to_string(notification_ids).unwrap();
        let req = test::TestRequest::get()
            .uri(&format!(
                "/v2/notifications?ids={}",
                urlencoding::encode(&notification_ids)
            ))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn mark_notification_read(
        &self,
        notification_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v2/notification/{notification_id}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn mark_notifications_read(
        &self,
        notification_ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let notification_ids = serde_json::to_string(notification_ids).unwrap();
        let req = test::TestRequest::patch()
            .uri(&format!(
                "/v2/notifications?ids={}",
                urlencoding::encode(&notification_ids)
            ))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn add_user_to_team(
        &self,
        team_id: &str,
        user_id: &str,
        project_permissions: Option<ProjectPermissions>,
        organization_permissions: Option<OrganizationPermissions>,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::post()
            .uri(&format!("/v2/team/{team_id}/members"))
            .append_pat(pat)
            .set_json(json!( {
                "user_id": user_id,
                "permissions" : project_permissions.map(|p| p.bits()).unwrap_or_default(),
                "organization_permissions" : organization_permissions.map(|p| p.bits()),
            }))
            .to_request();
        self.call(req).await
    }

    async fn delete_notification(
        &self,
        notification_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v2/notification/{notification_id}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn delete_notifications(
        &self,
        notification_ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse {
        let notification_ids = serde_json::to_string(notification_ids).unwrap();
        let req = test::TestRequest::delete()
            .uri(&format!(
                "/v2/notifications?ids={}",
                urlencoding::encode(&notification_ids)
            ))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }
}
