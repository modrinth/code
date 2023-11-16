use crate::common::environment::TestEnvironment;
use std::collections::HashSet;

#[actix_rt::test]
async fn get_tags() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v2;

    let game_versions = api.get_game_versions_deserialized().await;
    let loaders = api.get_loaders_deserialized().await;
    let side_types = api.get_side_types_deserialized().await;
    let categories = api.get_categories_deserialized().await;

    // These tests match dummy data and will need to be updated if the dummy data changes;
    let game_version_versions = game_versions
        .into_iter()
        .map(|x| x.version)
        .collect::<HashSet<_>>();
    assert_eq!(
        game_version_versions,
        [
            "1.20.1",
            "1.20.2",
            "1.20.3",
            "1.20.4",
            "1.20.5",
            "Ordering_Negative1",
            "Ordering_Positive100"
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    );

    let loader_names = loaders.into_iter().map(|x| x.name).collect::<HashSet<_>>();
    assert_eq!(
        loader_names,
        ["fabric", "forge", "mrpack"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    );

    let side_type_names = side_types.into_iter().collect::<HashSet<_>>();
    assert_eq!(
        side_type_names,
        ["unknown", "required", "optional", "unsupported"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    );

    let category_names = categories
        .into_iter()
        .map(|x| x.name)
        .collect::<HashSet<_>>();
    assert_eq!(
        category_names,
        [
            "combat",
            "economy",
            "food",
            "optimization",
            "decoration",
            "mobs",
            "magic"
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    );

    test_env.cleanup().await;
}
