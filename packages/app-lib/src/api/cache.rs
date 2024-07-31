use crate::state::{
    CachedEntry, Organization, Project, SearchResults, TeamMember, User,
    Version,
};

macro_rules! impl_cache_methods {
    ($(($variant:ident, $type:ty)),*) => {
        $(
            paste::paste! {
                #[tracing::instrument]
                pub async fn [<get_ $variant:snake>](
                    id: &str,
                ) -> crate::Result<Option<$type>>
                {
                    let state = crate::State::get().await?;
                    Ok(CachedEntry::[<get_ $variant:snake _many>](&[id], None, &state.pool, &state.api_semaphore).await?.into_iter().next())
                }

                #[tracing::instrument]
                pub async fn [<get_ $variant:snake _many>](
                    ids: &[&str],
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
