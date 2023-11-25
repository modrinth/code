use std::collections::HashSet;

use common::environment::with_test_environment_all;

use crate::common::api_common::ApiTags;

mod common;

#[actix_rt::test]
async fn get_tags() {
    with_test_environment_all(None, |test_env| async move {
        let api = &test_env.api;
        let loaders = api.get_loaders_deserialized_common().await;
        let categories = api.get_categories_deserialized_common().await;

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
    })
    .await;
}
