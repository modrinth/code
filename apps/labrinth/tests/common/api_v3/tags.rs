use actix_http::StatusCode;
use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use async_trait::async_trait;
use labrinth::routes::v3::tags::{GameData, LoaderData};
use labrinth::{
    database::models::loader_fields::LoaderFieldEnumValue,
    routes::v3::tags::CategoryData,
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

use super::ApiV3;

#[async_trait(?Send)]
impl ApiTags for ApiV3 {
    async fn get_loaders(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v3/tag/loader")
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
            .uri("/v3/tag/category")
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

impl ApiV3 {
    pub async fn get_loaders_deserialized(&self) -> Vec<LoaderData> {
        let resp = self.get_loaders().await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    pub async fn get_loader_field_variants(
        &self,
        loader_field: &str,
    ) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v3/loader_field?loader_field={loader_field}"))
            .append_pat(ADMIN_USER_PAT)
            .to_request();
        self.call(req).await
    }

    pub async fn get_loader_field_variants_deserialized(
        &self,
        loader_field: &str,
    ) -> Vec<LoaderFieldEnumValue> {
        let resp = self.get_loader_field_variants(loader_field).await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }

    async fn get_games(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v3/games")
            .append_pat(ADMIN_USER_PAT)
            .to_request();
        self.call(req).await
    }

    pub async fn get_games_deserialized(&self) -> Vec<GameData> {
        let resp = self.get_games().await;
        assert_status!(&resp, StatusCode::OK);
        test::read_body_json(resp).await
    }
}
