use clap::Parser;
use eyre::{Result, WrapErr, eyre};
use labrinth::models::projects::{Project, Version};
use labrinth::validate::project::validate;
use serde::de::DeserializeOwned;
use url::Url;

const API_BASE_URL: &str = "https://api.modrinth.com/v3/";

#[derive(Parser)]
#[command(version, about = "Validate a project from the Modrinth API")]
struct Args {
    /// The ID of the project to validate
    project_id: String,

    /// A Modrinth token to send as bearer authentication
    #[arg(long, value_name = "TOKEN")]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().wrap_err("installing color-eyre")?;
    let args = Args::parse();
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            "labrinth-check-project/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .wrap_err("building HTTP client")?;

    let project_url = api_url(&["project", &args.project_id])?;
    let mut versions_url = api_url(&["project", &args.project_id, "version"])?;
    versions_url
        .query_pairs_mut()
        .append_pair("include_changelog", "false");

    let token = args.token.as_deref();
    let (project, versions) = tokio::try_join!(
        fetch::<Project>(&client, project_url, token),
        fetch::<Vec<Version>>(&client, versions_url, token),
    )?;
    let nags = validate(&project, &versions);

    println!(
        "{}",
        serde_json::to_string_pretty(&nags)
            .wrap_err("serializing project nags")?
    );

    Ok(())
}

fn api_url(path: &[&str]) -> Result<Url> {
    let mut url = Url::parse(API_BASE_URL).wrap_err("parsing API base URL")?;
    url.path_segments_mut()
        .map_err(|_| eyre!("API base URL cannot be a base URL"))?
        .extend(path);
    Ok(url)
}

async fn fetch<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: Url,
    token: Option<&str>,
) -> Result<T> {
    let mut request = client.get(url.clone());
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }

    let response = request
        .send()
        .await
        .wrap_err_with(|| format!("requesting `{url}`"))?
        .error_for_status()
        .wrap_err_with(|| format!("requesting `{url}`"))?;

    response
        .json()
        .await
        .wrap_err_with(|| format!("deserializing response from `{url}`"))
}
