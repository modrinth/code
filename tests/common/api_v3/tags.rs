use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use labrinth::routes::v3::tags::GameData;
use labrinth::{
    database::models::loader_fields::LoaderFieldEnumValue,
    routes::v3::tags::{CategoryData, LoaderData},
};

use crate::common::database::ADMIN_USER_PAT;

use super::ApiV3;

impl ApiV3 {
    pub async fn get_loaders(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v3/tag/loader")
            .append_header(("Authorization", ADMIN_USER_PAT))
            .to_request();
        self.call(req).await
    }

    pub async fn get_loaders_deserialized(&self) -> Vec<LoaderData> {
        let resp = self.get_loaders().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_categories(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v3/tag/category")
            .append_header(("Authorization", ADMIN_USER_PAT))
            .to_request();
        self.call(req).await
    }

    pub async fn get_categories_deserialized(&self) -> Vec<CategoryData> {
        let resp = self.get_categories().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_loader_field_variants(&self, loader_field: &str) -> ServiceResponse {
        let req = TestRequest::get()
            .uri(&format!("/v3/loader_field?loader_field={}", loader_field))
            .append_header(("Authorization", ADMIN_USER_PAT))
            .to_request();
        self.call(req).await
    }

    pub async fn get_loader_field_variants_deserialized(
        &self,
        loader_field: &str,
    ) -> Vec<LoaderFieldEnumValue> {
        let resp = self.get_loader_field_variants(loader_field).await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    // TODO: fold this into v3 API of other v3 testing PR
    pub async fn get_games(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v3/games")
            .append_header(("Authorization", ADMIN_USER_PAT))
            .to_request();
        self.call(req).await
    }

    pub async fn get_games_deserialized(&self) -> Vec<GameData> {
        let resp = self.get_games().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }
}
