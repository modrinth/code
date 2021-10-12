use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct PodInfo {
    pub pod_name: String,
    pub node_name: String,
    pod_id: Arc<RwLock<Option<String>>>,
}

impl PodInfo {
    pub fn new() -> Self {
        Self {
            pod_name: dotenv::var("POD_NAME").unwrap_or("DEV".to_string()),
            node_name: dotenv::var("NODE_NAME").unwrap_or("self-hosted".to_string()),
            pod_id: Arc::new(RwLock::new(None)),
        }
    }
    pub fn get_id(&self) -> String {
        {
            let lock = self.pod_id.read().unwrap();
            if lock.is_some() {
                return lock.clone().unwrap();
            }
        }
        let mut lock = self.pod_id.write().unwrap();
        let id = self.generate_id();
        lock.replace(id.clone());
        id
    }
    fn generate_id(&self) -> String {
        base64::encode(format!("{}-{}", self.node_name, self.pod_name))
    }
}
