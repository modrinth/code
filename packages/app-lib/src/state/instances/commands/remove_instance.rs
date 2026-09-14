use crate::state::State;
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::util::io;

pub(crate) async fn remove_instance(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let instance = instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let _synced_options_lock = state.lock_synced_options().await;
    let _content_lock = state.lock_instance_content(instance_id).await;
    let _store_lock = state.content_store.files_lock.lock().await;
    let _store_lease = state.content_store.lease().await;
    if crate::state::instance_has_running_process(instance_id, state).await? {
        return Err(crate::state::content_store::input(
            "Stop this instance before removing it",
        ));
    }
    state.content_store.recover(Some(instance_id)).await?;
    crate::api::instance::remove_generated_instance_files(instance_id, state)
        .await?;

    delete_instance_row_and_locks(&instance.id, state).await?;

    let path = state.directories.instances_dir().join(&instance.path);
    if path.exists() {
        io::remove_dir_all(&path).await?;
    }

    Ok(())
}

async fn delete_instance_row_and_locks(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    // Keep these together so deleted instances cannot leave stale entries in the per-instance lock maps.
    instance_rows::delete_instance_by_id(instance_id, &state.pool).await?;
    state.remove_instance_locks(instance_id);

    Ok(())
}
