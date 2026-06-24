use super::events::emit_install_job;
use super::model::{
	InstallCleanup, InstallErrorView, InstallJobDisplay, InstallJobState,
	InstallJobStatus, InstallPhaseDetails, InstallPhaseId, InstallRequest,
	InstallTarget,
};
use super::store;
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::State;

pub async fn recover_interrupted_jobs(state: &State) -> crate::Result<()> {
	let jobs = store::list_interrupted_candidates(state).await?;

	for mut job in jobs {
		if job.state.display.is_none() {
			job.state.display = display_from_request(&job.state);
		}
		job.state.progress.phase = InstallPhaseId::RollingBack;
		job.state.progress.progress = None;
		job.state.progress.details = InstallPhaseDetails::Empty;
		job.state.error = Some(InstallErrorView {
			code: "interrupted".to_string(),
			message: "interrupted".to_string(),
		});

		if let Err(error) = apply_cleanup(&job.state, state).await {
			tracing::error!(
				"Error cleaning up interrupted install job {}: {error}",
				job.id
			);
		}
		clear_deleted_new_instance_id(&mut job.state);

		let record = store::update_status(
			job.id,
			InstallJobStatus::Interrupted,
			&job.state,
			state,
		)
		.await?;
		emit_install_job(&record.snapshot()).await?;
	}

	Ok(())
}

fn clear_deleted_new_instance_id(job_state: &mut InstallJobState) {
	if matches!(job_state.cleanup, InstallCleanup::DeleteNewInstance { .. }) {
		job_state.target = InstallTarget::NewInstance { instance_id: None };
		job_state.cleanup = InstallCleanup::DeleteNewInstance { instance_id: None };
	}
}

fn display_from_request(state: &InstallJobState) -> Option<InstallJobDisplay> {
	match &state.request {
		InstallRequest::CreateInstance { name, icon_path, .. } => {
			Some(InstallJobDisplay {
				title: name.clone(),
				icon: icon_path.clone(),
			})
		}
		InstallRequest::CreateModpackInstance { location, .. } => match location {
			crate::api::pack::install_from::CreatePackLocation::FromVersionId {
				title,
				icon_url,
				..
			} => Some(InstallJobDisplay {
				title: title.clone(),
				icon: icon_url.clone(),
			}),
			crate::api::pack::install_from::CreatePackLocation::FromFile { .. } => None,
		},
		InstallRequest::ImportInstance {
			instance_folder, ..
		} => Some(InstallJobDisplay {
			title: instance_folder.clone(),
			icon: None,
		}),
		InstallRequest::DuplicateInstance { .. }
		| InstallRequest::InstallExistingInstance { .. }
		| InstallRequest::InstallPackToExistingInstance { .. } => {
			state.rollback.as_ref().map(|rollback| InstallJobDisplay {
				title: rollback.instance.instance.name.clone(),
				icon: rollback.instance.instance.icon_path.clone(),
			})
		}
	}
}

pub async fn apply_cleanup(
	job_state: &InstallJobState,
	state: &State,
) -> crate::Result<()> {
	match &job_state.cleanup {
		InstallCleanup::DeleteNewInstance { instance_id } => {
			if let Some(instance_id) = instance_id {
				let _ = crate::state::remove_instance(instance_id, state).await;
				let _ = emit_instance(instance_id, InstancePayloadType::Removed).await;
			}
		}
		InstallCleanup::RestoreExistingInstance { instance_id } => {
			if let Some(rollback) = &job_state.rollback {
				crate::state::instances::commands::set_instance_install_stage(
					instance_id,
					rollback.install_stage,
					&state.pool,
				)
				.await?;
				emit_instance(instance_id, InstancePayloadType::Edited).await?;
			}
		}
	}

	Ok(())
}
