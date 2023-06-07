use uuid::Uuid;

use crate::State;

pub struct SafeProcesses {
    pub save_settings: Vec<Uuid>,
    pub download_modpacks: Vec<Uuid>,
}

#[derive(Debug)]
pub enum ProcessType {
    SaveSettings,
    DownloadModpacks,
}

impl SafeProcesses {
    // init
    pub fn new() -> Self {
        Self {
            save_settings: Vec::new(),
            download_modpacks: Vec::new(),
        }
    }

    pub async fn add(r#type: ProcessType) -> crate::Result<Uuid> {
        let state = State::get().await?;
        let mut safe_processes = state.safety_processes.write().await;

        let uuid = Uuid::new_v4();
        match r#type {
            ProcessType::SaveSettings => {
                safe_processes.save_settings.push(uuid);
            }
            ProcessType::DownloadModpacks => {
                safe_processes.download_modpacks.push(uuid);
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
            ProcessType::DownloadModpacks => {
                safe_processes.download_modpacks.retain(|x| *x != uuid);
            }
        }
        Ok(())
    }

    pub async fn wait_for_completion(r#type: ProcessType) -> crate::Result<()> {
        let state = State::get().await?;
        loop {
            let safe_processes = state.safety_processes.read().await;
            match r#type {
                ProcessType::SaveSettings => {
                    if safe_processes.save_settings.is_empty() {
                        break;
                    }
                }
                ProcessType::DownloadModpacks => {
                    if safe_processes.save_settings.is_empty() {
                        break;
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        Ok(())
    }
}
