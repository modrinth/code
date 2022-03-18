use super::DataError;
use crate::LAUNCHER_WORK_DIR;
use std::path::Path;
use tokio::sync::OnceCell;

static VERSIONS: OnceCell<Versions> = OnceCell::const_new();
pub const VERSIONS_SLED: &str = "versions.sled";

pub struct Versions(sled::Db);

impl Versions {
    pub async fn init() -> Result<(), DataError> {
        let versions = Path::new(LAUNCHER_WORK_DIR).join(VERSIONS_SLED);
        VERSIONS.get_or_try_init(|| sled::open(versions)).await?;
    }
}
