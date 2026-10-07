use std::num::NonZero;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub database_url: Url,
	pub concurrency: Concurrency,
	#[serde(default)]
	pub public_s3: Option<S3Config>,
	#[serde(default)]
	pub private_s3: Option<S3Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
	pub bucket: String,
	pub region: String,
	pub endpoint: Url,
	pub base_url: Url,
	#[serde(default)]
	pub path_style: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concurrency {
	pub download: NonZero<usize>,
	pub extract_files: NonZero<usize>,
}
