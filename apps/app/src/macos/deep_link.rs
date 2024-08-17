use std::sync::Arc;
use tokio::sync::Mutex;

pub struct InitialPayload {
    pub payload: Arc<Mutex<Option<String>>>,
}
