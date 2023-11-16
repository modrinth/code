use common::environment::TestEnvironment;
use std::collections::HashSet;

mod common;

#[actix_rt::test]
async fn get_tags() {
    let test_env = TestEnvironment::build(None).await;
    let api = &test_env.v3;

    let loaders = api.get_loaders_deserialized().await;
    let categories = api.get_categories_deserialized().await;

    let loader_names = loaders.into_iter().map(|x| x.name).collect::<HashSet<_>>();
    assert_eq!(
        loader_names,
        ["fabric", "forge", "mrpack"]
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
