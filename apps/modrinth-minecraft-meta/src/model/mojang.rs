use toasty::{Embed, Model};
use uuid::Uuid;

use crate::model::{DownloadBlob, DownloadBlobId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct MinecraftVersionId(pub Uuid);

#[derive(Debug, Clone, Model)]
pub struct MinecraftVersion {
    #[key]
    #[auto]
    pub id: MinecraftVersionId,
    pub source_blob_id: DownloadBlobId,
    #[belongs_to]
    pub source_blob: toasty::Deferred<DownloadBlob>,
}
