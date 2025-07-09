use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use async_trait::async_trait;
use labrinth::routes::v2::tags::{
    CategoryData, DonationPlatformQueryData, GameVersionQueryData, LoaderData,
};

use crate::{
    assert_status,
    common::{
        api_common::{
            Api, ApiTags, AppendsOptionalPat,
            models::{CommonCategoryData, CommonLoaderData},
        },
        database::ADMIN_USER_PAT,
    },
};

use super::ApiV2;

impl ApiV2 {
    async fn get_side_types(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/side_type")
            .append_pat(ADMIN_USER_PAT)
            .to_request();
        self.call(req).await
    }

    pub async fn get_side_types_deserialized(&self) -> Vec<String> {
        let resp = self.get_side_types().await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_game_versions(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/game_version")
            .append_pat(ADMIN_USER_PAT)
            .to_request();
        self.call(req).await
    }

    pub async fn get_game_versions_deserialized(
        &self,
    ) -> Vec<GameVersionQueryData> {
        let resp = self.get_game_versions().await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_loaders_deserialized(&self) -> Vec<LoaderData> {
        let resp = self.get_loaders().await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_categories_deserialized(&self) -> Vec<CategoryData> {
        let resp = self.get_categories().await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_donation_platforms(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/donation_platform")
            .append_pat(ADMIN_USER_PAT)
            .to_request();
        self.call(req).await
    }

    pub async fn get_donation_platforms_deserialized(
        &self,
    ) -> Vec<DonationPlatformQueryData> {
        let resp = self.get_donation_platforms().await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }
}

#[async_trait(?Send)]
impl ApiTags for ApiV2 {
    async fn get_loaders(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/loader")
            .append_pat(ADMIN_USER_PAT)
            .to_request();
        self.call(req).await
    }

    async fn get_loaders_deserialized_common(&self) -> Vec<CommonLoaderData> {
        let resp = self.get_loaders().await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: Vec<LoaderData> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }

    async fn get_categories(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/category")
            .append_pat(ADMIN_USER_PAT)
            .to_request();
        self.call(req).await
    }

    async fn get_categories_deserialized_common(
        &self,
    ) -> Vec<CommonCategoryData> {
        let resp = self.get_categories().await;
        assert_status!(&resp, StatusCode::OK);
        // First, deserialize to the non-common format (to test the response is valid for this api version)
        let v: Vec<CategoryData> = test::read_body_json(resp).await;
        // Then, deserialize to the common format
        let value = serde_json::to_value(v).unwrap();
        serde_json::from_value(value).unwrap()
    }
}
