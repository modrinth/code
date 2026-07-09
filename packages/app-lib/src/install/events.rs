use super::model::{
    InstallJobEventKind, InstallJobSnapshot, InstallJobState,
    InstallPhaseDetails, InstallPhaseId, InstallProgress,
};
use super::store;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct InstallProgressReporter {
    job_id: Uuid,
    state: Arc<Mutex<InstallJobState>>,
}

impl InstallProgressReporter {
    pub fn new(job_id: Uuid, state: InstallJobState) -> Self {
        Self {
            job_id,
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub async fn update(
        &self,
        phase: InstallPhaseId,
        progress: Option<InstallProgress>,
        details: InstallPhaseDetails,
    ) -> crate::Result<()> {
        self.update_with_events(phase, progress, details, Vec::new())
            .await
    }

    pub async fn update_with_events(
        &self,
        phase: InstallPhaseId,
        progress: Option<InstallProgress>,
        details: InstallPhaseDetails,
        events: Vec<InstallJobEventKind>,
    ) -> crate::Result<()> {
        let app_state = crate::State::get().await?;
        let mut state = self.state.lock().await;
        state.set_progress(phase, progress, details);
        for event in events {
            state.record_event(event);
        }

        let record =
            store::update_state(self.job_id, &state, &app_state).await?;
        emit_install_job(&record.snapshot()).await
    }
}

#[allow(unused_variables)]
pub async fn emit_install_job(
    snapshot: &InstallJobSnapshot,
) -> crate::Result<()> {
    #[cfg(feature = "tauri")]
    {
        use tauri::Emitter;

        let event_state = crate::EventState::get()?;
        event_state
            .app
            .emit("install_job", snapshot)
            .map_err(crate::event::EventError::from)?;
    }

    Ok(())
}
