use crate::api::Result;
use theseus::prelude::*;

macro_rules! impl_cache_methods {
    ($(($variant:ident, $type:ty)),*) => {
        $(
            paste::paste! {
                #[tauri::command]
                pub async fn [<get_ $variant:snake>](id: &str) -> Result<Option<$type>>
                {
                    Ok(theseus::cache::[<get_ $variant:snake>](id).await?)
                }

                #[tauri::command]
                pub async fn [<get_ $variant:snake _many>](
                    ids: Vec<String>,
                ) -> Result<Vec<$type>>
                {
                    let ids = ids.iter().map(|x| &**x).collect::<Vec<&str>>();
                    let entries =
                        theseus::cache::[<get_ $variant:snake _many>](&*ids).await?;

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

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("cache")
        .invoke_handler(tauri::generate_handler![
            get_project,
            get_project_many,
            get_version,
            get_version_many,
            get_user,
            get_user_many,
            get_team,
            get_team_many,
            get_organization,
            get_organization_many,
            get_search_results,
            get_search_results_many,
        ])
        .build()
}
