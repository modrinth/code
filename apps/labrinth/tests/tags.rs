use std::collections::{HashMap, HashSet};

use common::{
    api_v3::ApiV3,
    environment::{
        TestEnvironment, with_test_environment, with_test_environment_all,
    },
};

use crate::common::api_common::ApiTags;

mod common;

#[actix_rt::test]
async fn get_tags() {
    with_test_environment_all(None, |test_env| async move {
        let api = &test_env.api;
        let categories = api.get_categories_deserialized_common().await;

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
    })
    .await;
}

#[actix_rt::test]
async fn get_tags_v3() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let loaders = api.get_loaders_deserialized().await;

            let loader_metadata = loaders
                .into_iter()
                .map(|x| {
                    (
                        x.name,
                        x.metadata.get("platform").and_then(|x| x.as_bool()),
                    )
                })
                .collect::<HashMap<_, _>>();
            let loader_names =
                loader_metadata.keys().cloned().collect::<HashSet<String>>();
            assert_eq!(
                loader_names,
                ["fabric", "forge", "mrpack", "bukkit", "waterfall"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            );
            assert_eq!(loader_metadata["fabric"], None);
            assert_eq!(loader_metadata["bukkit"], Some(false));
            assert_eq!(loader_metadata["waterfall"], Some(true));
        },
    )
    .await;
}
