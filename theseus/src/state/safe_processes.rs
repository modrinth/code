use uuid::Uuid;

use crate::State;

pub struct SafeProcesses {
    pub save_settings: Vec<Uuid>,
    pub loading_bars: Vec<Uuid>,
}

#[derive(Debug, Copy, Clone)]
pub enum ProcessType {
    SaveSettings,
    LoadingBar,
}

impl SafeProcesses {
    // init
    pub fn new() -> Self {
        Self {
            save_settings: Vec::new(),
            loading_bars: Vec::new(),
        }
    }

    pub async fn add(r#type: ProcessType) -> crate::Result<Uuid> {
        let uuid = Uuid::new_v4();
        Self::add_uuid(r#type, uuid).await
    }

    pub async fn add_uuid(
        r#type: ProcessType,
        uuid: Uuid,
    ) -> crate::Result<Uuid> {
        let state = State::get().await?;
        let mut safe_processes = state.safety_processes.write().await;
        match r#type {
            ProcessType::SaveSettings => {
                safe_processes.save_settings.push(uuid);
            }
            ProcessType::LoadingBar => {
                safe_processes.loading_bars.push(uuid);
            }
        }
        Ok(uuid)
    }

    pub async fn complete(
        r#type: ProcessType,
        uuid: Uuid,
    ) -> crate::Result<()> {
        let state = State::get().await?;
        let mut safe_processes = state.safety_processes.write().await;

        match r#type {
            ProcessType::SaveSettings => {
                safe_processes.save_settings.retain(|x| *x != uuid);
            }
            ProcessType::LoadingBar => {
                safe_processes.loading_bars.retain(|x| *x != uuid);
            }
        }
        Ok(())
    }

    pub async fn is_complete(r#type: ProcessType) -> crate::Result<bool> {
        let state = State::get().await?;
        let safe_processes = state.safety_processes.read().await;
        match r#type {
            ProcessType::SaveSettings => {
                if safe_processes.save_settings.is_empty() {
                    return Ok(true);
                }
            }
            ProcessType::LoadingBar => {
                if safe_processes.loading_bars.is_empty() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub async fn wait_for_completion(r#type: ProcessType) -> crate::Result<()> {
        loop {
            if Self::is_complete(r#type).await? {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        Ok(())
    }
}
