//! Theseus process management interface

use crate::state::ProcessMetadata;
pub use crate::{
    State,
    state::{Hooks, MemorySettings, Settings, WindowSize},
};
use uuid::Uuid;

// Gets each running stored process in the state
#[tracing::instrument]
pub async fn get_all() -> crate::Result<Vec<ProcessMetadata>> {
	let state = State::get().await?;
	let processes = state.process_manager.get_all();
	Ok(processes)
}

pub async fn resolve_instance_id(instance: &str) -> crate::Result<String> {
	let state = State::get().await?;
	resolve_instance_id_with_state(instance, &state)
		.await?
		.ok_or_else(|| {
			crate::ErrorKind::InputError(format!(
				"Unknown instance id or path: {instance}"
			))
			.as_error()
		})
}

async fn resolve_instance_id_with_state(
	instance: &str,
	state: &State,
) -> crate::Result<Option<String>> {
	sqlx::query_scalar::<_, String>(
		"
		SELECT id
		FROM instances
		WHERE id = ? OR path = ?
		ORDER BY CASE WHEN id = ? THEN 0 ELSE 1 END
		LIMIT 1
		",
	)
	.bind(instance)
	.bind(instance)
	.bind(instance)
	.fetch_optional(&state.pool)
	.await
	.map_err(Into::into)
}

// Gets the UUID of each stored process in the state by instance id
#[tracing::instrument]
pub async fn get_by_instance_id(
	instance_id: &str,
) -> crate::Result<Vec<ProcessMetadata>> {
	let state = State::get().await?;
	let processes = state
		.process_manager
		.get_all()
		.into_iter()
		.filter(|x| x.instance_id == instance_id)
		.collect();
	Ok(processes)
}

// Kill a child process stored in the state by UUID, as a string
#[tracing::instrument]
pub async fn kill(uuid: Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    state.process_manager.kill(uuid).await?;

    Ok(())
}

// Wait for a child process stored in the state by UUID
#[tracing::instrument]
pub async fn wait_for(uuid: Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    state.process_manager.wait_for(uuid).await?;

    Ok(())
}
