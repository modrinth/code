use crate::state::State;
use crate::state::content_store::adapters::filesystem;
use crate::state::content_store::adapters::sqlite as catalog;
use crate::state::dirs::move_app_directory::{
    relocate_tree, remove_migrated_tree,
};

pub(crate) async fn migrate(state: &State) -> crate::Result<()> {
    let store = &state.content_store;
    if catalog::setting(&state.pool, "game_locales_layout_version")
        .await?
        .as_deref()
        != Some("1")
    {
        let source = state.directories.metadata_dir().join("game-locales");
        let destination = state.directories.caches_dir().join("game-locales");
        relocate_tree(&source, &destination).await?;
        if filesystem::path_exists(&source).await?
            && filesystem::path_exists(&destination).await?
        {
            remove_migrated_tree(&source).await?;
        }
        catalog::set_setting(&state.pool, "game_locales_layout_version", "1")
            .await?;
    }
    catalog::set_setting(&state.pool, "store_layout_version", "1").await?;
    for owner in catalog::retained_owners(&state.pool, "rollback").await? {
        if uuid::Uuid::parse_str(&owner).is_ok()
            && !filesystem::path_exists(
                &state.directories.install_backups_dir().join(&owner),
            )
            .await?
        {
            store.release("rollback", &owner).await?;
        }
    }
    store.remove_abandoned_staging().await?;
    store.recover_unregistered_files().await?;
    Ok(())
}
