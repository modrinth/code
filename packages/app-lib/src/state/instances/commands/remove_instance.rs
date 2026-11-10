use crate::ErrorKind;
use crate::state::State;
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::util::io;
use anyhow::bail;

pub(crate) async fn remove_instance(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let instance = instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .ok_or(crate::ErrorKind::InputError("unknown instance".to_string()))?;
    let _synced_options_lock = state.synced_options.lock().await;
    let _content_lock = state.instance_locks.lock_content(instance_id).await;
    let _store_lock = state.content_store.files_lock.lock().await;
    let _store_lease = state.content_store.lease().await;
    if crate::state::instance_has_running_process(instance_id, state).await? {
        bail!(ErrorKind::InputError(
            "stop this instance before removing it".to_string(),
        ));
    }
    state.content_store.recover(Some(instance_id)).await?;
    crate::api::instance::remove_generated_instance_files(instance_id, state)
        .await?;

    delete_instance_row_and_locks(&instance.id, state).await?;
    super::super::watcher::unwatch_instance_folder(
        &instance.id,
        &instance.path,
        &state.file_watcher,
        &state.directories,
    )
    .await;

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
    state.instance_locks.remove(instance_id);

    Ok(())
}
