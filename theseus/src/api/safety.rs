use crate::state::{ProcessType, SafeProcesses};

pub async fn check_safe_loading_bars() -> crate::Result<bool> {
    SafeProcesses::is_complete(ProcessType::LoadingBar).await
}
