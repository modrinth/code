use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use crate::env::ENV;
use crate::util::http::HTTP_CLIENT;

pub const DEFAULT_API_URL: &str = "https://api.crowdin.com/api/v2";

const CACHE_NAMESPACE: &str = "crowdin:v1";
const CACHE_EXPIRY_SECONDS: i64 = 60 * 60 * 24;

/// Suffixes Crowdin may append to a FormatJS message ID when it stores the
/// nested `defaultMessage` field as its own string identifier.
const IDENTIFIER_SUFFIXES: [&str; 2] = [".defaultMessage", "->defaultMessage"];

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    GeneralQuestion,
    TranslationMistake,
    ContextRequest,
    SourceMistake,
}

#[derive(Debug, Deserialize)]
struct DataWrapper<T> {
    data: T,
}

/// A language the Crowdin project is translated into.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetLanguage {
    /// Crowdin language ID, e.g. `de` or `pt-BR`.
    pub id: String,
    /// English name, e.g. `German` or `Portuguese, Brazilian`.
    pub name: String,
}

/// Client for the Crowdin project that holds Modrinth's translations.
pub struct CrowdinClient {
    base_url: String,
    token: String,
    project_id: u64,
}

impl CrowdinClient {
    /// Builds a client from environment variables, or returns `None` if
    /// Crowdin is not configured.
    pub fn from_env() -> Option<Self> {
        if ENV.CROWDIN_API_TOKEN.is_empty() || ENV.CROWDIN_PROJECT_ID == 0 {
            return None;
        }

        Some(Self {
            base_url: ENV.CROWDIN_API_URL.trim_end_matches('/').to_owned(),
            token: ENV.CROWDIN_API_TOKEN.clone(),
            project_id: ENV.CROWDIN_PROJECT_ID,
        })
    }

    fn project_url(&self, path: &str) -> String {
        format!("{}/projects/{}{path}", self.base_url, self.project_id)
    }

    /// Maps a Modrinth locale code (e.g. `de-DE`) to one of the project's
    /// Crowdin target languages (e.g. `de`), preferring an exact match.
    pub async fn resolve_language(
        &self,
        redis: &RedisPool,
        locale: &str,
    ) -> Result<Option<TargetLanguage>> {
        let targets = self.target_languages(redis).await?;

        let exact = targets
            .iter()
            .find(|language| language.id.eq_ignore_ascii_case(locale));
        let by_language = || {
            let code = locale.split('-').next()?;
            targets
                .iter()
                .find(|language| language.id.eq_ignore_ascii_case(code))
        };

        Ok(exact.or_else(by_language).cloned())
    }

    async fn target_languages(
        &self,
        redis: &RedisPool,
    ) -> Result<Vec<TargetLanguage>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Project {
            target_languages: Vec<TargetLanguage>,
        }

        let mut conn = redis.connect().await?;
        let cache_key = conn.key().entity(
            CACHE_NAMESPACE,
            format!("target_languages:{}", self.project_id),
        );

        if let Some(cached) = conn.get_deserialized(&cache_key).await? {
            return Ok(cached);
        }

        let project = HTTP_CLIENT
            .get(self.project_url(""))
            .bearer_auth(&self.token)
            .send()
            .await
            .and_then(|res| res.error_for_status())
            .wrap_err("requesting Crowdin project")?
            .json::<DataWrapper<Project>>()
            .await
            .wrap_err("parsing Crowdin project")?
            .data;

        conn.set_serialized(
            &cache_key,
            &project.target_languages,
            Some(CACHE_EXPIRY_SECONDS),
        )
        .await?;

        Ok(project.target_languages)
    }

    /// Finds the Crowdin string ID for a message ID.
    pub async fn find_string_id(
        &self,
        redis: &RedisPool,
        identifier: &str,
    ) -> Result<Option<u64>> {
        #[derive(Deserialize)]
        struct CrowdinString {
            id: u64,
            identifier: String,
        }

        let mut conn = redis.connect().await?;
        let cache_key = conn.key().entity(
            CACHE_NAMESPACE,
            format!("string:{}:{identifier}", self.project_id),
        );

        if let Some(cached) = conn.get_deserialized(&cache_key).await? {
            return Ok(Some(cached));
        }

        let strings = HTTP_CLIENT
            .get(self.project_url("/strings"))
            .bearer_auth(&self.token)
            .query(&[
                ("filter", identifier),
                ("scope", "identifier"),
                ("limit", "50"),
            ])
            .send()
            .await
            .and_then(|res| res.error_for_status())
            .wrap_err("requesting Crowdin strings")?
            .json::<DataWrapper<Vec<DataWrapper<CrowdinString>>>>()
            .await
            .wrap_err("parsing Crowdin strings")?
            .data;

        let Some(string_id) = strings
            .into_iter()
            .map(|string| string.data)
            .find(|string| identifier_matches(&string.identifier, identifier))
            .map(|string| string.id)
        else {
            return Ok(None);
        };

        conn.set_serialized(&cache_key, string_id, Some(CACHE_EXPIRY_SECONDS))
            .await?;

        Ok(Some(string_id))
    }

    /// Opens an issue on a string for a specific target language.
    pub async fn create_issue(
        &self,
        string_id: u64,
        language_id: &str,
        issue_type: IssueType,
        text: &str,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct NewComment<'a> {
            string_id: u64,
            target_language_id: &'a str,
            text: &'a str,
            #[serde(rename = "type")]
            kind: &'static str,
            issue_type: IssueType,
        }

        HTTP_CLIENT
            .post(self.project_url("/comments"))
            .bearer_auth(&self.token)
            .json(&NewComment {
                string_id,
                target_language_id: language_id,
                text,
                kind: "issue",
                issue_type,
            })
            .send()
            .await
            .and_then(|res| res.error_for_status())
            .wrap_err("creating Crowdin string issue")?;

        Ok(())
    }
}

fn identifier_matches(crowdin_identifier: &str, message_id: &str) -> bool {
    crowdin_identifier == message_id
        || IDENTIFIER_SUFFIXES.iter().any(|suffix| {
            crowdin_identifier.strip_suffix(suffix) == Some(message_id)
        })
}
