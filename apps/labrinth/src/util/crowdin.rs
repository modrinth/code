use eyre::{Result, eyre};
use serde::Deserialize;
use serde_json::json;

use crate::env::ENV;
use crate::util::error::Context;
use crate::util::http::HTTP_CLIENT;

/// How many times to poll a freshly-generated Crowdin report before giving up.
const REPORT_POLL_ATTEMPTS: u32 = 10;
const REPORT_POLL_INTERVAL: std::time::Duration =
    std::time::Duration::from_secs(1);

pub fn authorize_url(state: &str, redirect_uri: &str) -> String {
    format!(
        "https://accounts.crowdin.com/oauth/authorize?client_id={}&response_type=code&scope=project&redirect_uri={}&state={state}",
        ENV.CROWDIN_CLIENT_ID,
        urlencoding::encode(redirect_uri),
    )
}

#[derive(Deserialize)]
struct CrowdinTokenResponse {
    access_token: String,
}

/// Exchanges a one-time OAuth code for a short-lived Crowdin access token.
pub async fn exchange_code(code: &str, redirect_uri: &str) -> Result<String> {
    let response: CrowdinTokenResponse = HTTP_CLIENT
        .post("https://accounts.crowdin.com/oauth/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", ENV.CROWDIN_CLIENT_ID.as_str()),
            ("client_secret", ENV.CROWDIN_CLIENT_SECRET.as_str()),
            ("redirect_uri", redirect_uri),
            ("code", code),
        ])
        .send()
        .await
        .wrap_err("exchanging Crowdin OAuth code")?
        .error_for_status()
        .wrap_err("exchanging Crowdin OAuth code")?
        .json()
        .await
        .wrap_err("parsing Crowdin token response")?;

    Ok(response.access_token)
}

#[derive(Deserialize)]
struct CrowdinUserResponse {
    data: CrowdinUser,
}

#[derive(Deserialize)]
struct CrowdinUser {
    id: i64,
}

/// Looks up the Crowdin account id for the given (single-use) access token.
pub async fn fetch_own_user_id(access_token: &str) -> Result<i64> {
    let response: CrowdinUserResponse = HTTP_CLIENT
        .get("https://api.crowdin.com/api/v2/user")
        .bearer_auth(access_token)
        .send()
        .await
        .wrap_err("fetching Crowdin user profile")?
        .error_for_status()
        .wrap_err("fetching Crowdin user profile")?
        .json()
        .await
        .wrap_err("parsing Crowdin user profile")?;

    Ok(response.data.id)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ContributionStats {
    pub translated: u32,
    pub approved: u32,
}

impl ContributionStats {
    /// Any translated or approved string is enough for the Translator badge.
    pub fn has_contribution(&self) -> bool {
        self.translated > 0 || self.approved > 0
    }
}

#[derive(Deserialize)]
struct GenerateReportResponse {
    data: ReportIdentifier,
}

#[derive(Deserialize)]
struct ReportIdentifier {
    identifier: String,
}

#[derive(Deserialize)]
struct ReportStatusResponse {
    data: ReportStatus,
}

#[derive(Deserialize)]
struct ReportStatus {
    status: String,
}

#[derive(Deserialize)]
struct ReportDownloadResponse {
    data: ReportDownloadUrl,
}

#[derive(Deserialize)]
struct ReportDownloadUrl {
    url: String,
}

#[derive(Deserialize)]
struct TopMembersReport {
    data: Vec<TopMembersReportEntry>,
}

#[derive(Deserialize)]
struct TopMembersReportEntry {
    user: TopMembersReportUser,
    #[serde(default)]
    translated: u32,
    #[serde(default)]
    approved: u32,
}

#[derive(Deserialize)]
struct TopMembersReportUser {
    id: i64,
}

/// Looks up how much a Crowdin user has translated/proofread on the Modrinth
/// project, using our own project-level API token.
pub async fn fetch_contribution_stats(
    crowdin_user_id: i64,
) -> Result<ContributionStats> {
    let project_id = &ENV.CROWDIN_PROJECT_ID;
    let reports_url = format!(
        "https://api.crowdin.com/api/v2/projects/{project_id}/reports"
    );

    let generated: GenerateReportResponse = HTTP_CLIENT
        .post(&reports_url)
        .bearer_auth(&ENV.CROWDIN_PROJECT_API_TOKEN)
        .json(&json!({
            "name": "top-members",
            "schema": { "unit": "strings", "format": "json" },
        }))
        .send()
        .await
        .wrap_err("generating Crowdin top-members report")?
        .error_for_status()
        .wrap_err("generating Crowdin top-members report")?
        .json()
        .await
        .wrap_err("parsing Crowdin report generation response")?;

    let report_url = format!("{reports_url}/{}", generated.data.identifier);
    let mut finished = false;

    for _ in 0..REPORT_POLL_ATTEMPTS {
        let status: ReportStatusResponse = HTTP_CLIENT
            .get(&report_url)
            .bearer_auth(&ENV.CROWDIN_PROJECT_API_TOKEN)
            .send()
            .await
            .wrap_err("polling Crowdin report status")?
            .error_for_status()
            .wrap_err("polling Crowdin report status")?
            .json()
            .await
            .wrap_err("parsing Crowdin report status response")?;

        if status.data.status == "finished" {
            finished = true;
            break;
        }

        tokio::time::sleep(REPORT_POLL_INTERVAL).await;
    }

    if !finished {
        return Err(eyre!("Crowdin report did not finish generating in time"));
    }

    let download: ReportDownloadResponse = HTTP_CLIENT
        .get(format!("{report_url}/download"))
        .bearer_auth(&ENV.CROWDIN_PROJECT_API_TOKEN)
        .send()
        .await
        .wrap_err("fetching Crowdin report download url")?
        .error_for_status()
        .wrap_err("fetching Crowdin report download url")?
        .json()
        .await
        .wrap_err("parsing Crowdin report download response")?;

    let report: TopMembersReport = HTTP_CLIENT
        .get(&download.data.url)
        .send()
        .await
        .wrap_err("downloading Crowdin report")?
        .error_for_status()
        .wrap_err("downloading Crowdin report")?
        .json()
        .await
        .wrap_err("parsing Crowdin report")?;

    let stats = report
        .data
        .into_iter()
        .find(|entry| entry.user.id == crowdin_user_id)
        .map(|entry| ContributionStats {
            translated: entry.translated,
            approved: entry.approved,
        })
        .unwrap_or_default();

    Ok(stats)
}

#[derive(Deserialize)]
struct MemberResponse {
    data: Member,
}

#[derive(Deserialize)]
struct Member {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    roles: Vec<MemberRole>,
}

#[derive(Deserialize)]
struct MemberRole {
    #[serde(default)]
    name: Option<String>,
}

/// Checks whether the user holds a proofreader-level role on the project.
pub async fn has_proofreader_role(crowdin_user_id: i64) -> Result<bool> {
    let project_id = &ENV.CROWDIN_PROJECT_ID;
    let response: MemberResponse = HTTP_CLIENT
        .get(format!(
            "https://api.crowdin.com/api/v2/projects/{project_id}/members/{crowdin_user_id}"
        ))
        .bearer_auth(&ENV.CROWDIN_PROJECT_API_TOKEN)
        .send()
        .await
        .wrap_err("fetching Crowdin project member")?
        .error_for_status()
        .wrap_err("fetching Crowdin project member")?
        .json()
        .await
        .wrap_err("parsing Crowdin project member response")?;
    let member = response.data;

    let primary_role = member.role.unwrap_or_default().to_lowercase();
    if matches!(primary_role.as_str(), "proofreader" | "owner" | "manager") {
        return Ok(true);
    }

    Ok(member.roles.iter().any(|role| {
        matches!(
            role.name.as_deref().unwrap_or_default().to_lowercase().as_str(),
            "proofreader" | "language_coordinator"
        )
    }))
}
