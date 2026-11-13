use eyre::{Result, WrapErr};
use figment::{Figment, providers::Env};
use secrecy::SecretString;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub project_flags: ProjectFlags,
}

impl Config {
    pub fn load() -> Result<Self> {
        Figment::new()
            .merge(Env::prefixed("MR_").split("__"))
            .extract()
            .wrap_err("loading application configuration")
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectFlags {
    pub image_explicit: Option<DecisionsConfig>,
    pub image_ai_generated: Option<SmaugConfig>,
    pub text_content: Option<DecisionsConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DecisionsConfig {
    /// Full Decisions API endpoint, e.g. `https://openrouter.ai/api/alpha/decisions`.
    pub endpoint_url: Url,
    pub api_key: SecretString,
    pub model_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SmaugConfig {
    /// Full prediction endpoint. Accepts a POST of raw image bytes with the
    /// matching image Content-Type and returns JSON containing
    /// `ai_generated_probability`, a number in [0, 1].
    pub endpoint_url: Url,
    /// Optional bearer token for authenticated deployments.
    pub api_key: Option<SecretString>,
}
