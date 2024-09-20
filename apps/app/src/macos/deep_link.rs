use std::sync::Arc;
use tauri::{Manager, Runtime};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct InitialPayload {
    pub payload: Arc<Mutex<Option<String>>>,
}

pub fn get_or_init_payload<R: Runtime, M: Manager<R>>(
    manager: &M,
) -> InitialPayload {
    let initial_payload = manager.try_state::<InitialPayload>();
    let mtx = if let Some(initial_payload) = initial_payload {
        initial_payload.inner().clone()
    } else {
        tracing::info!("No initial payload found, creating new");
        let payload = InitialPayload {
            payload: Arc::new(Mutex::new(None)),
        };

        manager.manage(payload.clone());

        payload
    };

    mtx
}
