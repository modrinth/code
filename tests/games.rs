// TODO: fold this into loader_fields.rs or tags.rs of other v3 testing PR

use crate::common::environment::TestEnvironment;

mod common;

#[actix_rt::test]
async fn get_games() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v3;

    let games = api.get_games_deserialized().await;

    // There should be 2 games in the dummy data
    assert_eq!(games.len(), 2);
    assert_eq!(games[0].name, "minecraft-java");
    assert_eq!(games[1].name, "minecraft-bedrock");

    assert_eq!(games[0].slug, "minecraft-java");
    assert_eq!(games[1].slug, "minecraft-bedrock");

    test_env.cleanup().await;
}
