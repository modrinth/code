use std::num::NonZero;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub database_url: Url,
	pub public_base_url: Url,
	pub concurrency: Concurrency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concurrency {
	pub download: NonZero<usize>,
	pub extract_files: NonZero<usize>,
}
