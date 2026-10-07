use anyhow::Result;

use crate::{
	store::{BlobStore, ContentType},
	util::MavenCoordinate,
};

#[derive(Debug, Clone)]
pub struct MavenStore {
	imp: BlobStore,
}

impl MavenStore {
	#[must_use]
	pub const fn new(imp: BlobStore) -> Self {
		Self { imp }
	}

	pub async fn put(
		&self,
		coordinate: &MavenCoordinate,
		data: &[u8],
	) -> Result<()> {
		let path = coordinate.to_maven_path();
		self.imp.put(&path, data, ContentType::Binary).await
	}
}
