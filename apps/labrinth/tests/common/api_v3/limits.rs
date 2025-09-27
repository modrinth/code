use actix_http::StatusCode;
use actix_web::test;
use labrinth::database::models::user_limits::UserLimits;

use crate::{
    assert_status,
    common::{
        api_common::{Api, AppendsOptionalPat},
        api_v3::ApiV3,
    },
};

impl ApiV3 {
    pub async fn get_limits(&self, pat: Option<&str>) -> UserLimits {
        let req = test::TestRequest::get()
            .uri("/v3/limits")
            .append_pat(pat)
            .to_request();
        let resp = self.call(req).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }
}
