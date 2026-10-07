use anyhow::Result;
use tracing::info_span;
use tracing_anyhow::FutureContext;

use crate::{
	model,
	store::{BlobStore, ContentType},
	util::{Sha1, Sha256},
};

/// Stores arbitrary binary [`Vec<u8>`] blobs keyed by the binary data's
/// [`Sha256`] hash.
///
/// This is a _content-addressed store_ (CAS) model.
#[derive(Debug, Clone)]
pub struct BlobCas {
	imp: BlobStore,
}

impl BlobCas {
	#[must_use]
	pub const fn new(imp: BlobStore) -> Self {
		Self { imp }
	}

	pub async fn get(&self, sha256: Sha256) -> Result<Vec<u8>> {
		self.imp.get(&path_for(sha256)).await
	}

	pub async fn put(
		&self,
		exec: &mut dyn toasty::Executor,
		data: &[u8],
	) -> Result<Sha256> {
		let sha256 = Sha256::from_digest(data);
		let sha1 = Sha1::from_digest(data);
		model::BlobHash::upsert_by_sha256(sha256)
			.sha1(sha1)
			.or_ignore()
			.exec(exec)
			.context(info_span!("upserting blob hash record"))
			.await?;
		self.imp
			.put(&path_for(sha256), data, ContentType::Binary)
			.await?;
		Ok(sha256)
	}
}

fn path_for(sha256: Sha256) -> String {
	let sha256 = sha256.to_string();
	let (first, _) = sha256.split_at(2);
	format!("blobs/{first}/{sha256}")
}
