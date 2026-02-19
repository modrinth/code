use async_trait::async_trait;
use eyre::Result;

use crate::search::SearchBackend;

pub struct Elasticsearch {}

impl Elasticsearch {
    pub fn new(meta_namespace: Option<String>) -> Result<Self> {
        Elasticsearch {}
    }
}

#[async_trait]
impl SearchBackend for Elasticsearch {}
