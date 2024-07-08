use uuid::Uuid;

use crate::State;

// We implement a store for safe loading bars such that we can wait for them to complete
// We create this store separately from the loading bars themselves, because this may be extended as needed
pub struct SafeProcesses {
    pub loading_bars: Vec<Uuid>,
}

#[derive(Debug, Copy, Clone)]
pub enum ProcessType {
    LoadingBar,
    // Potentially other types of processes (ie: IO operations?)
}

impl SafeProcesses {
    // init
    pub fn new() -> Self {
        Self {
            loading_bars: Vec::new(),
        }
    }

    // Adds a new running safe process to the list by uuid
    pub async fn add_uuid(
        r#type: ProcessType,
        uuid: Uuid,
    ) -> crate::Result<Uuid> {
        let state = State::get().await?;
        let mut safe_processes = state.safety_processes.write().await;
        match r#type {
            ProcessType::LoadingBar => {
                safe_processes.loading_bars.push(uuid);
            }
        }
        Ok(uuid)
    }

    // Mark a safe process as finishing
    pub async fn complete(
        r#type: ProcessType,
        uuid: Uuid,
    ) -> crate::Result<()> {
        let state = State::get().await?;
        let mut safe_processes = state.safety_processes.write().await;

        match r#type {
            ProcessType::LoadingBar => {
                safe_processes.loading_bars.retain(|x| *x != uuid);
            }
        }
        Ok(())
    }

    // Check if there are any pending safe processes of a given type
    pub async fn is_complete(r#type: ProcessType) -> crate::Result<bool> {
        let state = State::get().await?;
        let safe_processes = state.safety_processes.read().await;
        match r#type {
            ProcessType::LoadingBar => {
                if safe_processes.loading_bars.is_empty() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
