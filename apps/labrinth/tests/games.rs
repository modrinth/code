// TODO: fold this into loader_fields.rs or tags.rs of other v3 testing PR

use common::{
    api_v3::ApiV3,
    environment::{TestEnvironment, with_test_environment},
};

mod common;

#[actix_rt::test]
async fn get_games() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = test_env.api;

            let games = api.get_games_deserialized().await;

            // There should be 2 games in the dummy data
            assert_eq!(games.len(), 2);
            assert_eq!(games[0].name, "minecraft-java");
            assert_eq!(games[1].name, "minecraft-bedrock");

            assert_eq!(games[0].slug, "minecraft-java");
            assert_eq!(games[1].slug, "minecraft-bedrock");
        },
    )
    .await;
}
