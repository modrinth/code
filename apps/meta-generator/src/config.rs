use std::num::NonZero;

use secrecy::SecretString;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: Url,
    pub concurrency: Concurrency,
    #[serde(default)]
    pub public_s3: Option<S3>,
    #[serde(default)]
    pub private_s3: Option<S3>,
    #[serde(default)]
    pub fastly: Option<Fastly>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3 {
    pub bucket: String,
    pub region: String,
    pub endpoint: Url,
    pub base_url: Url,
    #[serde(default)]
    pub path_style: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Concurrency {
    pub download: NonZero<usize>,
    pub extract_files: NonZero<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fastly {
    pub base_url: Url,
    pub key: SecretString,
}
