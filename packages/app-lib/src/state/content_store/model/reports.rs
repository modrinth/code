use serde::Serialize;

#[derive(Serialize)]
pub struct StoreUsage {
    pub unique_bytes: u64,
    pub shared_bytes: u64,
    pub unused_cache_bytes: u64,
    pub estimated_saved_bytes: u64,
    pub private_copy_bytes: u64,
    pub object_count: usize,
    pub damaged_objects: usize,
    pub cache_limit_bytes: u64,
}

#[derive(Serialize)]
pub struct StoreIssue {
    pub sha512: String,
    pub instance_ids: Vec<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct StoreVerification {
    pub checked: usize,
    pub repaired: usize,
    pub issues: Vec<StoreIssue>,
}
