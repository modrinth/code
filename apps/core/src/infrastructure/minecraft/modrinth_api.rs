use serde::{Deserialize, Serialize};

const MODRINTH_API: &str = "https://api.modrinth.com/v2";

#[derive(Debug, thiserror::Error)]
pub enum ModrinthError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthProject {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub project_type: String,
    pub client_side: Option<String>,
    pub server_side: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub version_number: String,
    pub name: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<ModrinthFile>,
    #[serde(default)]
    pub dependencies: Vec<ModrinthDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub hashes: ModrinthHashes,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthHashes {
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub sha256: Option<String>,
}

pub struct ModrinthClient {
    http: reqwest::Client,
}

impl ModrinthClient {
    pub fn new(http: reqwest::Client) -> Self {
        Self { http }
    }

    pub async fn get_project(
        &self,
        id_or_slug: &str,
    ) -> Result<ModrinthProject, ModrinthError> {
        self.get(&format!("/project/{id_or_slug}")).await
    }

    /// Looks up many projects at once. Unknown ids are left out of the result.
    pub async fn get_projects(
        &self,
        ids: &[String],
    ) -> Result<Vec<ModrinthProject>, ModrinthError> {
        let mut projects = Vec::new();
        for chunk in ids.chunks(200) {
            let resp = self
                .http
                .get(format!("{MODRINTH_API}/projects"))
                .query(&[(
                    "ids",
                    serde_json::to_string(chunk).unwrap_or_default(),
                )])
                .send()
                .await?
                .error_for_status()?;
            projects.extend(resp.json::<Vec<ModrinthProject>>().await?);
        }
        Ok(projects)
    }

    pub async fn get_version(
        &self,
        version_id: &str,
    ) -> Result<ModrinthVersion, ModrinthError> {
        self.get(&format!("/version/{version_id}")).await
    }

    pub async fn list_versions(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ModrinthVersion>, ModrinthError> {
        let mut url = format!("{MODRINTH_API}/project/{project_id}/version");
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(gv) = game_version {
            params.push(("game_versions", format!("[\"{gv}\"]")));
        }
        if let Some(l) = loader {
            params.push(("loaders", format!("[\"{l}\"]")));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(
                &params
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("&"),
            );
        }
        let resp = self.http.get(&url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, ModrinthError> {
        let resp = self
            .http
            .get(format!("{MODRINTH_API}{path}"))
            .send()
            .await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ModrinthError::NotFound);
        }
        Ok(resp.error_for_status()?.json().await?)
    }
}
