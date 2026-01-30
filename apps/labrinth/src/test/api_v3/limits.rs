use crate::models::v3::user_limits::UserLimits;
use actix_http::StatusCode;
use actix_web::test;

use crate::test::asserts::assert_status;
use crate::test::{
    api_common::{Api, AppendsOptionalPat},
    api_v3::ApiV3,
};

impl ApiV3 {
    pub async fn get_project_limits(&self, pat: Option<&str>) -> UserLimits {
        let req = test::TestRequest::get()
            .uri("/v3/limits/projects")
            .append_pat(pat)
            .to_request();
        let resp = self.call(req).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_organization_limits(
        &self,
        pat: Option<&str>,
    ) -> UserLimits {
        let req = test::TestRequest::get()
            .uri("/v3/limits/organizations")
            .append_pat(pat)
            .to_request();
        let resp = self.call(req).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_collection_limits(&self, pat: Option<&str>) -> UserLimits {
        let req = test::TestRequest::get()
            .uri("/v3/limits/collections")
            .append_pat(pat)
            .to_request();
        let resp = self.call(req).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }
}
