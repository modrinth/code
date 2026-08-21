use eyre::Result;
use serde::Deserialize;

use crate::env::ENV;
use crate::util::error::Context;
use crate::util::http::HTTP_CLIENT;

#[derive(Deserialize)]
struct GitHubUserLookup {
    login: String,
}

#[derive(Deserialize)]
struct GitHubSearchResult {
    total_count: u32,
}

fn authed(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    if ENV.GITHUB_CONTRIBUTOR_PAT.is_empty()
        || ENV.GITHUB_CONTRIBUTOR_PAT == "none"
    {
        builder
    } else {
        builder.bearer_auth(&ENV.GITHUB_CONTRIBUTOR_PAT)
    }
}

/// Resolves a GitHub user id (as stored on `users.github_id`) to their current
/// login/username, since the search API only accepts usernames.
async fn username_for_id(github_id: i64) -> Result<String> {
    let user: GitHubUserLookup = authed(
        HTTP_CLIENT.get(format!("https://api.github.com/user/{github_id}")),
    )
    .send()
    .await
    .wrap_err("fetching GitHub user by id")?
    .error_for_status()
    .wrap_err("fetching GitHub user by id")?
    .json()
    .await
    .wrap_err("parsing GitHub user response")?;

    Ok(user.login)
}

/// Counts merged pull requests authored by `username` against the configured
/// contributor repository (`modrinth/code` by default).
async fn merged_pr_count(username: &str) -> Result<u32> {
    let repo = &ENV.GITHUB_CONTRIBUTOR_REPO;
    let query = format!("repo:{repo} is:pr is:merged author:{username}");

    let result: GitHubSearchResult = authed(
        HTTP_CLIENT
            .get("https://api.github.com/search/issues")
            .query(&[("q", query.as_str()), ("per_page", "1")]),
    )
    .send()
    .await
    .wrap_err("searching GitHub for merged pull requests")?
    .error_for_status()
    .wrap_err("searching GitHub for merged pull requests")?
    .json()
    .await
    .wrap_err("parsing GitHub search response")?;

    Ok(result.total_count)
}

/// Checks whether the GitHub account linked via `github_id` has enough merged
/// pull requests to earn the Contributor badge.
pub async fn is_eligible_contributor(github_id: i64) -> Result<bool> {
    let username = username_for_id(github_id).await?;
    let count = merged_pr_count(&username).await?;
    Ok(count >= ENV.GITHUB_CONTRIBUTOR_MERGED_PR_THRESHOLD)
}
