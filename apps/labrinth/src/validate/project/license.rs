use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};
use crate::models::v2::projects::LegacyProject;

const INAPPROPRIATE_LICENSE_DOMAINS: &[&str] = &[
    "youtube.com",
    "youtu.be",
    "modrinth.com",
    "curseforge.com",
    "twitter.com",
    "x.com",
    "discord.gg",
    "discord.com",
    "instagram.com",
    "facebook.com",
    "tiktok.com",
    "reddit.com",
    "twitch.tv",
    "patreon.com",
    "ko-fi.com",
    "paypal.com",
    "buymeacoffee.com",
    "google.com",
    "example.com",
    "t.me",
];

fn hostname_matches_domain(hostname: &str, domain: &str) -> bool {
    hostname == domain
        || hostname
            .strip_suffix(domain)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

enum InvalidLicenseUrl {
    InappropriateDomain(String),
    Malformed,
}

fn get_invalid_license_url(url: &str) -> Option<InvalidLicenseUrl> {
    if url.is_empty() {
        return None;
    }

    let Ok(url) = url::Url::parse(url) else {
        return Some(InvalidLicenseUrl::Malformed);
    };
    let Some(hostname) = url.host_str() else {
        return Some(InvalidLicenseUrl::Malformed);
    };
    let hostname = hostname.to_ascii_lowercase();
    let hostname = hostname.trim_end_matches('.');

    INAPPROPRIATE_LICENSE_DOMAINS
        .iter()
        .any(|domain| hostname_matches_domain(hostname, domain))
        .then(|| InvalidLicenseUrl::InappropriateDomain(hostname.to_string()))
}

pub(super) fn validate(
    project: &crate::models::projects::Project,
) -> Vec<super::ProjectNag> {
    let mut nags = Vec::new();
    let is_minecraft_server = project.components.minecraft_server.is_some();
    let (project_type, _) =
        LegacyProject::get_project_type(&project.project_types);
    let license = &project.license;
    let has_unknown_license = matches!(
        license.id.as_str(),
        "LicenseRef-Unknown" | "NOASSERTION" | "LicenseRef-NOASSERTION"
    );

    if has_unknown_license && !is_minecraft_server {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::SelectLicense,
                ProjectNagSeverity::Required,
            )
            .with_details(serde_json::json!({ "project_type": project_type })),
        );
    }

    let has_license_url =
        license.url.as_deref().is_some_and(|url| !url.is_empty());
    let missing_custom_license_details = license.id == "LicenseRef-"
        || (license.id.starts_with("LicenseRef-")
            && !has_license_url
            && license.id != "LicenseRef-Unknown"
            && license.id != "LicenseRef-All-Rights-Reserved");
    if missing_custom_license_details && !is_minecraft_server {
        nags.push(ProjectNag::new(
            ProjectNagKind::AddCustomLicenseDetails,
            ProjectNagSeverity::Required,
        ));
    }

    if let Some(invalid_url) =
        license.url.as_deref().and_then(get_invalid_license_url)
    {
        let details = match invalid_url {
            InvalidLicenseUrl::InappropriateDomain(domain) => {
                serde_json::json!({
                    "domain": domain,
                    "reason": "inappropriate_domain",
                })
            }
            InvalidLicenseUrl::Malformed => serde_json::json!({
                "reason": "malformed",
            }),
        };
        nags.push(
            ProjectNag::new(
                ProjectNagKind::InvalidLicenseUrl,
                ProjectNagSeverity::Required,
            )
            .with_details(details),
        );
    }

    nags
}
