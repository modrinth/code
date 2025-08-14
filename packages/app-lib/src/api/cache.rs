use crate::state::{
    CacheBehaviour, CacheValueType, CachedEntry, Organization, Project,
    SearchResults, TeamMember, User, Version,
};

macro_rules! impl_cache_methods {
    ($(($variant:ident, $type:ty)),*) => {
        $(
            paste::paste! {
                #[tracing::instrument]
                pub async fn [<get_ $variant:snake>](
                    id: &str,
                    cache_behaviour: Option<CacheBehaviour>,
                ) -> crate::Result<Option<$type>>
                {
                    let state = crate::State::get().await?;
                    Ok(CachedEntry::[<get_ $variant:snake _many>](&[id], cache_behaviour, &state.pool, &state.api_semaphore).await?.into_iter().next())
                }

                #[tracing::instrument]
                pub async fn [<get_ $variant:snake _many>](
                    ids: &[&str],
                    cache_behaviour: Option<CacheBehaviour>,
                ) -> crate::Result<Vec<$type>>
                {
                    let state = crate::State::get().await?;
                    let entries =
                        CachedEntry::[<get_ $variant:snake _many>](ids, None, &state.pool, &state.api_semaphore).await?;

                    Ok(entries)
                }
            }
        )*
    }
}

impl_cache_methods!(
    (Project, Project),
    (Version, Version),
    (User, User),
    (Team, Vec<TeamMember>),
    (Organization, Organization),
    (SearchResults, SearchResults)
);

pub async fn purge_cache_types(
    cache_types: &[CacheValueType],
) -> crate::Result<()> {
    let state = crate::State::get().await?;
    CachedEntry::purge_cache_types(cache_types, &state.pool).await?;

    Ok(())
}
