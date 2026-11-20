use std::{collections::BTreeMap, sync::Arc, time::Duration};

use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{
    Client,
    header::{ACCEPT, CONTENT_TYPE},
};
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::env::{DecisionsConfig, ENV, ProjectFlags};
use crate::file_hosting::{FileHost, FileHostPublicity};
use crate::routes::internal::globals::GLOBALS;
use xredis::RedisPool;

use super::image_scan::{
    ImageScanInput, InferenceQuotaReached, check_quota_reached_and_consume,
};
use crate::models::v3::ids::ProjectId;

pub struct ModelScanner {
    http: Client,
    pub(crate) config: ProjectFlags,
    redis: RedisPool,
    file_host: Arc<dyn FileHost>,
}

impl ModelScanner {
    pub fn new(
        config: ProjectFlags,
        redis: RedisPool,
        file_host: Arc<dyn FileHost>,
    ) -> Self {
        Self {
            http: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("failed to create model scanner HTTP client"),
            config,
            redis,
            file_host,
        }
    }

    async fn consume_inference_quota(
        &self,
        project_id: ProjectId,
    ) -> Result<()> {
        if let Some(retry_after) = check_quota_reached_and_consume(
            &self.redis,
            "project_flag_inference_quota:v1",
            &project_id.to_string(),
            self.config.inference_requests_limit,
            self.config.inference_requests_window,
        )
        .await?
        {
            return Err(InferenceQuotaReached {
                retry_after: retry_after.max(Duration::from_millis(1)),
            }
            .into());
        }
        Ok(())
    }

    pub(super) fn decision_model_version(&self) -> Result<Option<i64>> {
        self.config
            .infer_decisions
            .as_ref()
            .map(|config| i64::try_from(config.model_version))
            .transpose()
            .context("decisions model version exceeds PostgreSQL bigint range")
    }

    pub(super) fn image_model_versions(
        &self,
    ) -> Result<(Option<i64>, Option<i64>)> {
        let explicit = self.decision_model_version()?;
        let ai_generated = self
            .config
            .infer_ai_generated
            .as_ref()
            .map(|config| i64::try_from(config.model_version))
            .transpose()
            .context(
                "AiDetect model version exceeds PostgreSQL bigint range",
            )?;
        Ok((explicit, ai_generated))
    }

    pub(super) async fn read_image(
        &self,
        input: &ImageScanInput,
    ) -> Result<(bytes::Bytes, &'static str)> {
        let cdn_prefix = format!("{}/", ENV.CDN_URL.trim_end_matches('/'));
        let key = input
            .url
            .as_str()
            .strip_prefix(&cdn_prefix)
            .context("image URL must belong to the configured CDN")?;
        let key =
            urlencoding::decode(key).context("decoding image storage key")?;
        ensure!(
            input.url.query().is_none()
                && input.url.fragment().is_none()
                && !key.is_empty()
                && !key.contains(['\\', '\0'])
                && key.split('/').all(|part| !matches!(part, "" | "." | "..")),
            "invalid image storage key"
        );
        let bytes = self
            .file_host
            .read_file(&key, FileHostPublicity::Public)
            .await
            .context("reading image from storage")?;
        let limit = GLOBALS
            .max_gallery_image_size
            .max(GLOBALS.max_project_icon_size);
        ensure!(bytes.len() <= limit, "image exceeds the upload size limit");
        let content_type = image::guess_format(&bytes)
            .context("detecting image format")?
            .to_mime_type();
        Ok((bytes, content_type))
    }

    /// Returns the AI-generated probability, or `None` if AiDetect is disabled.
    pub async fn scan_image_ai_generated(
        &self,
        project_id: ProjectId,
        image: &[u8],
        content_type: &str,
    ) -> Result<Option<f64>> {
        let Some(config) = &self.config.infer_ai_generated else {
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
            .header(ACCEPT, "application/json")
            .body(image.to_vec());
        if let Some(api_key) = &config.api_key {
            request = request.bearer_auth(api_key.expose_secret());
        }

        let request = request
            .build()
            .context("building AiDetect inference request")?;
        self.consume_inference_quota(project_id).await?;
        let response = self
            .http
            .execute(request)
            .await
            .context("sending AiDetect inference request")?
            .error_for_status()
            .context("AiDetect inference request failed")?
            .json::<AiDetectResponse>()
            .await
            .context("decoding AiDetect inference response")?;

        validate_probability(response.score)
            .context("invalid AiDetect inference probability")
            .map(Some)
    }

    /// Returns the probability that the image satisfies the configured explicit
    /// content criteria, or `None` if the scanner is disabled.
    pub async fn scan_image_explicit(
        &self,
        project_id: ProjectId,
        image: &[u8],
        content_type: &str,
    ) -> Result<Option<f64>> {
        let Some(config) = &self.config.infer_decisions else {
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

        self.decide(
            project_id,
            config,
            state,
            std::slice::from_ref(&self.config.infer_explicit_prompt),
        )
        .await
        .context("scanning explicit image content")
        .and_then(|answers| {
            answers
                .into_iter()
                .next()
                .context("missing explicit image answer")
        })
        .map(Some)
    }

    /// Returns probabilities in question order for the supplied text,
    /// or `None` if the scanner is disabled.
    pub async fn scan_text_content(
        &self,
        project_id: ProjectId,
        text: &str,
        questions: &[String],
    ) -> Result<Option<Vec<f64>>> {
        let Some(config) = &self.config.infer_decisions else {
            return Ok(None);
        };
        ensure!(!text.trim().is_empty(), "text must not be empty");

        self.decide(
            project_id,
            config,
            Value::String(text.to_owned()),
            questions,
        )
        .await
        .context("scanning text content")
        .map(Some)
    }

    async fn decide(
        &self,
        project_id: ProjectId,
        config: &DecisionsConfig,
        state: Value,
        questions: &[String],
    ) -> Result<Vec<f64>> {
        if questions.is_empty() {
            return Ok(Vec::new());
        }
        let questions = decision_questions(questions)?;
        let request = self
            .http
            .post(config.endpoint_url.clone())
            .timeout(Duration::from_secs(30))
            .bearer_auth(config.api_key.expose_secret())
            .json(&json!({
                "model": config.model_name,
                "state": state,
                "questions": questions,
            }))
            .build()
            .context("building decisions inference request")?;
        self.consume_inference_quota(project_id).await?;
        let response = self
            .http
            .execute(request)
            .await
            .context("sending decisions inference request")?
            .error_for_status()
            .context("decisions inference request failed")?
            .json::<DecisionsResponse>()
            .await
            .context("decoding decisions inference response")?;

        decode_decisions(response, questions.len())
    }
}

#[derive(Deserialize)]
struct AiDetectResponse {
    /// Smaug's AI-generated probability. Its `flagged` decision is deliberately
    /// ignored so moderation rules retain control over thresholds.
    score: f64,
}

#[derive(Deserialize)]
struct DecisionsResponse {
    answers: BTreeMap<String, DecisionAnswer>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DecisionAnswer {
    Noul { noul: f64 },
}

fn decision_questions(questions: &[String]) -> Result<BTreeMap<String, Value>> {
    questions
        .iter()
        .enumerate()
        .map(|(index, instructions)| {
            ensure!(
                !instructions.trim().is_empty(),
                "decision instructions must not be empty"
            );
            Ok((
                format!("q{index}"),
                json!({ "type": "noul", "instructions": instructions }),
            ))
        })
        .collect()
}

fn decode_decisions(
    response: DecisionsResponse,
    count: usize,
) -> Result<Vec<f64>> {
    ensure!(
        response.answers.len() == count,
        "decisions answer count mismatch"
    );
    (0..count)
        .map(|index| {
            let key = format!("q{index}");
            let DecisionAnswer::Noul { noul } = response
                .answers
                .get(&key)
                .with_context(|| format!("missing decisions answer `{key}`"))?;
            validate_probability(*noul).with_context(|| {
                format!("invalid decisions inference probability for `{key}`")
            })
        })
        .collect()
}

fn validate_probability(probability: f64) -> Result<f64> {
    ensure!(
        probability.is_finite() && (0.0..=1.0).contains(&probability),
        "inference probability must be finite and between 0 and 1"
    );
    Ok(probability)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(value: Value, count: usize) -> Result<Vec<f64>> {
        decode_decisions(serde_json::from_value(value)?, count)
    }

    #[test]
    fn smaug_score_preserves_probability_independently_of_flagged() {
        for (score, flagged) in [
            (0.0002, false),
            (0.85, true),
            (0.4, true),
            (0.0, false),
            (1.0, true),
        ] {
            let response: AiDetectResponse = serde_json::from_value(json!({
                "classifier": "smaug-ai-image-detector",
                "score": score,
                "flagged": flagged,
            }))
            .unwrap();
            assert_eq!(validate_probability(response.score).unwrap(), score);
        }
    }

    #[test]
    fn smaug_rejects_missing_wrong_type_and_out_of_range_scores() {
        for value in [
            json!({"flagged": false}),
            json!({"score": null}),
            json!({"score": "0.5"}),
            json!({"ai_generated_probability": 0.5}),
        ] {
            assert!(serde_json::from_value::<AiDetectResponse>(value).is_err());
        }
        for score in [-0.1, 1.1] {
            let response: AiDetectResponse =
                serde_json::from_value(json!({"score": score})).unwrap();
            assert!(validate_probability(response.score).is_err());
        }
    }

    #[test]
    fn generated_keys_preserve_prompts_without_using_rule_names() {
        let prompts = vec!["A prompt with spaces?".into(), "q0".into()];
        let questions = decision_questions(&prompts).unwrap();
        assert_eq!(questions.len(), 2);
        assert_eq!(
            questions["q0"],
            json!({"type": "noul", "instructions": prompts[0]})
        );
        assert_eq!(questions["q1"]["instructions"], prompts[1]);
        assert!(decision_questions(&["  ".into()]).is_err());
        assert!(decision_questions(&[]).unwrap().is_empty());
    }

    #[test]
    fn answers_follow_numeric_request_order_not_response_order() {
        let answers = (0..12)
            .rev()
            .map(|index| {
                (
                    format!("q{index}"),
                    json!({"type": "noul", "noul": index as f64 / 12.0}),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        let probabilities = parse(json!({"answers": answers}), 12).unwrap();
        for (index, probability) in probabilities.into_iter().enumerate() {
            assert_eq!(probability, index as f64 / 12.0);
        }
    }

    #[test]
    fn rejects_missing_extra_wrong_type_and_invalid_answers() {
        for answers in [
            json!({}),
            json!({"other": {"type": "noul", "noul": 0.5}}),
            json!({"q0": {"type": "noul", "noul": 0.5}, "q1": {"type": "noul", "noul": 0.5}}),
            json!({"q0": {"type": "other", "noul": 0.5}}),
            json!({"q0": {"type": "noul", "noul": -0.1}}),
            json!({"q0": {"type": "noul", "noul": 1.1}}),
            json!({"q0": {"type": "noul", "noul": null}}),
        ] {
            assert!(parse(json!({"answers": answers}), 1).is_err());
        }
        for probability in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(validate_probability(probability).is_err());
        }
        assert_eq!(parse(json!({"answers": {"q0": {"type": "noul", "noul": 0.0}, "q1": {"type": "noul", "noul": 1.0}}}), 2).unwrap(), vec![0.0, 1.0]);
    }
}
