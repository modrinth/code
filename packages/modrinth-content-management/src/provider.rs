use async_trait::async_trait;

use crate::model::{Error, Version};

#[async_trait]
pub trait ContentMetadataProvider {
    async fn get_version(
        &self,
        version_id: &str,
    ) -> Result<Option<Version>, Error>;

    async fn get_project_versions(
        &self,
        project_id: &str,
    ) -> Result<Vec<Version>, Error>;
}
