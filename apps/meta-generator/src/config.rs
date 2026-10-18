use std::num::NonZero;

use secrecy::SecretString;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: Url,
    pub concurrency: Concurrency,
    #[serde(default)]
    pub sentry: Option<Sentry>,
    #[serde(default)]
    pub public_s3: Option<S3>,
    #[serde(default)]
    pub private_s3: Option<S3>,
    #[serde(default)]
    pub fastly: Option<Fastly>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sentry {
    pub dsn: String,
    pub environment: String,
    #[serde(default = "default_sample_rate")]
    pub sample_rate: f32,
    #[serde(default = "default_sample_rate")]
    pub traces_sample_rate: f32,
}

fn default_sample_rate() -> f32 {
    1.0
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
    pub service_id: String,
    pub surrogate_key: String,
}
