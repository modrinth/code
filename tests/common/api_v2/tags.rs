use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use async_trait::async_trait;
use labrinth::routes::v2::tags::{CategoryData, GameVersionQueryData, LoaderData};

use crate::common::{
    api_common::{
        models::{CommonCategoryData, CommonLoaderData},
        Api, ApiTags,
    },
    database::ADMIN_USER_PAT,
};

use super::ApiV2;

// TODO: Tag gets do not include PAT, as they are public.

impl ApiV2 {
    async fn get_side_types(&self) -> ServiceResponse {
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

    pub async fn get_loaders_deserialized(&self) -> Vec<LoaderData> {
        let resp = self.get_loaders().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    pub async fn get_categories_deserialized(&self) -> Vec<CategoryData> {
        let resp = self.get_categories().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }
}

#[async_trait(?Send)]
impl ApiTags for ApiV2 {
    async fn get_loaders(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/loader")
            .append_header(("Authorization", ADMIN_USER_PAT))
            .to_request();
        self.call(req).await
    }

    async fn get_loaders_deserialized_common(&self) -> Vec<CommonLoaderData> {
        let resp = self.get_loaders().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }

    async fn get_categories(&self) -> ServiceResponse {
        let req = TestRequest::get()
            .uri("/v2/tag/category")
            .append_header(("Authorization", ADMIN_USER_PAT))
            .to_request();
        self.call(req).await
    }

    async fn get_categories_deserialized_common(&self) -> Vec<CommonCategoryData> {
        let resp = self.get_categories().await;
        assert_eq!(resp.status(), 200);
        test::read_body_json(resp).await
    }
}
