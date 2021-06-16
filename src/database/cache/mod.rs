//pub mod project_cache;
//pub mod project_query_cache;
#[macro_export]
macro_rules! generate_cache {
    ($name:ident,$id:ty, $val:ty, $cache_name:ident, $mod_name:ident, $getter_name:ident, $setter_name:ident, $remover_name:ident) => {
        pub mod $mod_name {
            use cached::async_mutex::Mutex;
            use cached::{Cached, SizedCache};
            use lazy_static::lazy_static;
            lazy_static! {
                static ref $cache_name: Mutex<SizedCache<$id, $val>> =
                    Mutex::new(SizedCache::with_size(400));
            }

            pub async fn $getter_name<'a>(id: $id) -> Option<$val> {
                let mut cache = $cache_name.lock().await;
                Cached::cache_get(&mut *cache, &id).map(|e| e.clone())
            }
            pub async fn $setter_name<'a>(id: $id, val: &$val) {
                let mut cache = $cache_name.lock().await;
                Cached::cache_set(&mut *cache, id, val.clone());
            }
            pub async fn $remover_name<'a>(id: $id) {
                let mut cache = $cache_name.lock().await;
                Cached::cache_remove(&mut *cache, &id);
            }
        }
    };
}

generate_cache!(
    project,
    String,
    crate::database::Project,
    PROJECT_CACHE,
    project_cache,
    get_cache_project,
    set_cache_project,
    remove_cache_project
);
generate_cache!(
    query_project,
    String,
    crate::database::models::project_item::QueryProject,
    QUERY_PROJECT_CACHE,
    query_project_cache,
    get_cache_query_project,
    set_cache_query_project,
    remove_cache_query_project
);
