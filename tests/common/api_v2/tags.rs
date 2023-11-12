use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use labrinth::routes::v2::tags::{CategoryData, GameVersionQueryData, LoaderData};

use crate::common::database::ADMIN_USER_PAT;

use super::ApiV2;

impl ApiV2 {
    // Tag gets do not include PAT, as they are public.

    pub async fn get_side_types(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/side_type")
            .append_header(("Authorization", ADMIN_USER_PAT))
            .to_request();
        self.call(req).await
    }

    pub async fn get_side_types_deserialized(&self) -> Vec<String> {
        let resp = self.get_side_types().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_loaders(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/loader")
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
            .uri("/v2/tag/category")
            .append_header(("Authorization", ADMIN_USER_PAT))
            .to_request();
        self.call(req).await
    }

    pub async fn get_categories_deserialized(&self) -> Vec<CategoryData> {
        let resp = self.get_categories().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_game_versions(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/game_version")
            .append_header(("Authorization", ADMIN_USER_PAT))
            .to_request();
        self.call(req).await
    }

    pub async fn get_game_versions_deserialized(&self) -> Vec<GameVersionQueryData> {
        let resp = self.get_game_versions().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }
}
