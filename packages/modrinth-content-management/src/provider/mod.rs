use async_trait::async_trait;

use crate::model::{Error, Version};

#[async_trait]
pub trait ContentMetadataProvider: Send + Sync {
    async fn get_version(
        &mut self,
        version_id: &str,
    ) -> Result<Option<Version>, Error>;

    async fn get_project_versions(
        &mut self,
        project_id: &str,
    ) -> Result<Vec<Version>, Error>;
}
