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
    let _content_lock = state.lock_instance_content(instance_id).await;

    delete_instance_row_and_content_lock(&instance.id, state).await?;

    let path = state.directories.instances_dir().join(&instance.path);
    if path.exists() {
        io::remove_dir_all(&path).await?;
    }

    Ok(())
}

async fn delete_instance_row_and_content_lock(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    // Keep these together so deleted instances cannot leave stale entries in the per-instance lock map.
    instance_rows::delete_instance_by_id(instance_id, &state.pool).await?;
    state.remove_instance_content_lock(instance_id);

    Ok(())
}
