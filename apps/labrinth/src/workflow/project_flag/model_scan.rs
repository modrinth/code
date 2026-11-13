use std::time::Duration;

use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{Client, header::CONTENT_TYPE};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::env::{DecisionsConfig, ENV, ProjectFlags};
use crate::routes::internal::globals::GLOBALS;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageScanInput {
    pub url: Url,
    /// Increment when the scanner models, configuration, or policy changes.
    pub generation: i64,
    pub explicit_instructions: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageScanResult {
    pub explicit_probability: Option<f64>,
    pub ai_generated_probability: Option<f64>,
}

#[derive(Debug)]
pub struct ModelScanner {
    http: Client,
    config: ProjectFlags,
}

impl ModelScanner {
    pub fn new(config: ProjectFlags) -> Self {
        Self {
            http: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("failed to create model scanner HTTP client"),
            config,
        }
    }

    pub(super) async fn scan_image_url(
        &self,
        input: &ImageScanInput,
    ) -> Result<ImageScanResult> {
        if self.config.image_explicit.is_none()
            && self.config.image_ai_generated.is_none()
        {
            return Ok(ImageScanResult::default());
        }

        let cdn = Url::parse(&ENV.CDN_URL).context("parsing CDN URL")?;
        ensure!(
            matches!(input.url.scheme(), "http" | "https")
                && input.url.origin() == cdn.origin()
                && input.url.path().starts_with(&format!(
                    "{}/",
                    cdn.path().trim_end_matches('/')
                ))
                && input.url.username().is_empty()
                && input.url.password().is_none(),
            "image URL must belong to the configured CDN"
        );

        let mut response = self
            .http
            .get(input.url.clone())
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .context("downloading image for scanning")?
            .error_for_status()
            .context("image download failed")?;
        ensure!(
            response.status().is_success(),
            "image download did not return a success status"
        );
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .map(str::trim)
            .ok_or_else(|| {
                anyhow::anyhow!("image response is missing its content type")
            })?
            .to_owned();
        let limit = GLOBALS
            .max_gallery_image_size
            .max(GLOBALS.max_project_icon_size);
        if let Some(length) = response.content_length() {
            ensure!(
                length <= limit as u64,
                "image exceeds the upload size limit"
            );
        }
        let mut bytes = Vec::new();
        while let Some(chunk) =
            response.chunk().await.context("reading image download")?
        {
            ensure!(
                chunk.len() <= limit - bytes.len(),
                "image exceeds the upload size limit"
            );
            bytes.extend_from_slice(&chunk);
        }

        let (explicit_probability, ai_generated_probability) = tokio::try_join!(
            self.scan_image_explicit(
                &bytes,
                &content_type,
                &input.explicit_instructions
            ),
            self.scan_image_ai_generated(&bytes, &content_type),
        )?;
        Ok(ImageScanResult {
            explicit_probability,
            ai_generated_probability,
        })
    }

    /// Returns the AI-generated probability, or `None` if Smaug is disabled.
    pub async fn scan_image_ai_generated(
        &self,
        image: &[u8],
        content_type: &str,
    ) -> Result<Option<f64>> {
        let Some(config) = &self.config.image_ai_generated else {
            return Ok(None);
        };
        ensure!(!image.is_empty(), "image must not be empty");
        ensure!(
            content_type.starts_with("image/"),
            "content type must be an image media type"
        );

        let mut request = self
            .http
            .post(config.endpoint_url.clone())
            .timeout(Duration::from_secs(30))
            .header(CONTENT_TYPE, content_type)
            .body(image.to_vec());
        if let Some(api_key) = &config.api_key {
            request = request.bearer_auth(api_key.expose_secret());
        }

        let response = request
            .send()
            .await
            .context("sending Smaug inference request")?
            .error_for_status()
            .context("Smaug inference request failed")?
            .json::<SmaugResponse>()
            .await
            .context("decoding Smaug inference response")?;

        validate_probability(response.ai_generated_probability)
            .context("invalid Smaug inference probability")
            .map(Some)
    }

    /// Returns the probability that the image satisfies the supplied explicit
    /// content criteria, or `None` if the scanner is disabled.
    pub async fn scan_image_explicit(
        &self,
        image: &[u8],
        content_type: &str,
        instructions: &str,
    ) -> Result<Option<f64>> {
        let Some(config) = &self.config.image_explicit else {
            return Ok(None);
        };
        ensure!(!image.is_empty(), "image must not be empty");
        ensure!(
            matches!(content_type, "image/png" | "image/jpeg" | "image/webp"),
            "decisions image must be PNG, JPEG, or WebP"
        );

        let state = json!([{
            "type": "image_url",
            "image_url": {
                "url": format!("data:{content_type};base64,{}", STANDARD.encode(image)),
            },
        }]);

        self.decide(config, state, instructions)
            .await
            .context("scanning explicit image content")
            .map(Some)
    }

    /// Returns the probability that the text satisfies the supplied criteria,
    /// or `None` if the scanner is disabled.
    pub async fn scan_text_content(
        &self,
        text: &str,
        instructions: &str,
    ) -> Result<Option<f64>> {
        let Some(config) = &self.config.text_content else {
            return Ok(None);
        };
        ensure!(!text.trim().is_empty(), "text must not be empty");

        self.decide(config, Value::String(text.to_owned()), instructions)
            .await
            .context("scanning text content")
            .map(Some)
    }

    async fn decide(
        &self,
        config: &DecisionsConfig,
        state: Value,
        instructions: &str,
    ) -> Result<f64> {
        ensure!(
            !instructions.trim().is_empty(),
            "decision instructions must not be empty"
        );
        let response = self
            .http
            .post(config.endpoint_url.clone())
            .timeout(Duration::from_secs(30))
            .bearer_auth(config.api_key.expose_secret())
            .json(&json!({
                "model": config.model_name,
                "state": state,
                "questions": {
                    "result": {
                        "type": "noul",
                        "instructions": instructions,
                    },
                },
            }))
            .send()
            .await
            .context("sending decisions inference request")?
            .error_for_status()
            .context("decisions inference request failed")?
            .json::<DecisionsResponse>()
            .await
            .context("decoding decisions inference response")?;

        let DecisionAnswer::Noul { noul } = response.answers.result;
        validate_probability(noul)
            .context("invalid decisions inference probability")
    }
}

#[derive(Deserialize)]
struct SmaugResponse {
    ai_generated_probability: f64,
}

#[derive(Deserialize)]
struct DecisionsResponse {
    answers: DecisionAnswers,
}

#[derive(Deserialize)]
struct DecisionAnswers {
    result: DecisionAnswer,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DecisionAnswer {
    Noul { noul: f64 },
}

fn validate_probability(probability: f64) -> Result<f64> {
    ensure!(
        probability.is_finite() && (0.0..=1.0).contains(&probability),
        "inference probability must be finite and between 0 and 1"
    );
    Ok(probability)
}
