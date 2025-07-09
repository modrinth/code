use actix_web::{dev::ServiceResponse, test};
use async_trait::async_trait;

use crate::common::api_common::{Api, ApiUser, AppendsOptionalPat};

use super::ApiV3;

#[async_trait(?Send)]
impl ApiUser for ApiV3 {
    async fn get_user(
        &self,
        user_id_or_username: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri(&format!("/v3/user/{user_id_or_username}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn get_current_user(&self, pat: Option<&str>) -> ServiceResponse {
        let req = test::TestRequest::get()
            .uri("/v3/user")
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }

    async fn edit_user(
        &self,
        user_id_or_username: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::patch()
            .uri(&format!("/v3/user/{user_id_or_username}"))
            .append_pat(pat)
            .set_json(patch)
            .to_request();

        self.call(req).await
    }

    async fn delete_user(
        &self,
        user_id_or_username: &str,
        pat: Option<&str>,
    ) -> ServiceResponse {
        let req = test::TestRequest::delete()
            .uri(&format!("/v3/user/{user_id_or_username}"))
            .append_pat(pat)
            .to_request();
        self.call(req).await
    }
}
