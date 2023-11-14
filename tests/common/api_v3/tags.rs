use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
};
use labrinth::routes::v3::tags::GameData;

use crate::common::database::ADMIN_USER_PAT;

use super::ApiV3;

impl ApiV3 {
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
