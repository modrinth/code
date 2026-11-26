use std::{num::NonZero, time::Duration};

use anyhow::{Context, Result};
use figment::{Figment, providers::Env};
use secrecy::SecretString;
use serde::Deserialize;
use serde_with::{DurationSeconds, serde_as};
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
            .context("loading application configuration")
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ProjectFlags {
    /// Allow automatic approval after the configured delay and final checks.
    pub auto_approve_enabled: bool,
    /// Allow project flag rules to automatically withhold projects.
    pub auto_withhold_enabled: bool,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub auto_approve_min_delay: Duration,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub auto_approve_max_delay: Duration,
    pub infer_decisions: Option<DecisionsConfig>,
    pub infer_explicit_prompt: String,
    pub infer_ai_generated: Option<AiDetectConfig>,
    /// Maximum inference requests per project, per `inference_requests_window`.
    pub inference_requests_limit: NonZero<u64>,
    /// Per-project inference request window, configured in seconds.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub inference_requests_window: Duration,
}

impl Default for ProjectFlags {
    fn default() -> Self {
        Self {
            auto_approve_enabled: false,
            auto_withhold_enabled: false,
            auto_approve_min_delay: Duration::ZERO,
            auto_approve_max_delay: Duration::ZERO,
            infer_decisions: None,
            infer_explicit_prompt: default_explicit_prompt(),
            infer_ai_generated: None,
            inference_requests_limit: NonZero::new(20).unwrap(),
            inference_requests_window: Duration::from_secs(3600),
        }
    }
}

fn default_explicit_prompt() -> String {
    "Does this image depict sexually explicit content, \
including explicit sexual activity or exposed genitalia?"
        .to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct DecisionsConfig {
    /// Full Decisions API endpoint, e.g. `https://openrouter.ai/api/alpha/decisions`.
    pub endpoint_url: Url,
    pub api_key: SecretString,
    pub model_name: String,
    /// Increment when the model, prompt, or preprocessing changes.
    #[serde(default = "default_model_version")]
    pub model_version: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiDetectConfig {
    /// Full Smaug prediction endpoint.
    /// Accepts a POST of raw image bytes with the matching image Content-Type
    /// and returns JSON containing `score`, an AI-generated probability in
    /// [0, 1]. Moderation rules apply their own thresholds, not `flagged`.
    pub endpoint_url: Url,
    /// Optional bearer token for authenticated deployments.
    pub api_key: Option<SecretString>,
    /// Increment when the model or preprocessing changes.
    #[serde(default = "default_model_version")]
    pub model_version: u64,
}

fn default_model_version() -> u64 {
    1
}
