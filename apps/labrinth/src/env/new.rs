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
    pub image_explicit: Option<AiConfig>,
    pub image_ai_generated: Option<AiConfig>,
    pub text_content: Option<AiConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    pub base_url: Url,
    pub api_key: SecretString,
    pub model_name: String,
}
