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

	instance_rows::delete_instance_by_id(&instance.id, &state.pool).await?;

	let path = state.directories.instances_dir().join(&instance.path);
	if path.exists() {
		io::remove_dir_all(&path).await?;
	}

	Ok(())
}
