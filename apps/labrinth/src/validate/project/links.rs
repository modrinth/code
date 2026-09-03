use std::net::IpAddr;

use url::Url;

use super::{ProjectNagKind, ProjectNagSeverity};
use crate::models::{projects::Version, v2::projects::LegacyProject};

const SOURCE_DOMAINS: &[&str] = &[
    "github.com",
    "gitlab.com",
    "bitbucket.org",
    "codeberg.org",
    "git.sr.ht",
    "tangled.org",
    "git.gay",
];
const ISSUES_DOMAINS: &[&str] = &[
    "github.com",
    "gitlab.com",
    "bitbucket.org",
    "codeberg.org",
    "docs.google.com",
    "tangled.org",
    "git.gay",
];
const DISCORD_DOMAINS: &[&str] = &["discord.gg", "discord.com", "dsc.gg"];
const URL_SHORTENERS: &[&str] =
    &["bit.ly", "adf.ly", "tinyurl.com", "short.io", "is.gd"];
const BLOCKED_EXTERNAL_LINK_DOMAINS: &[&str] = &[
    "twitter.com",
    "x.com",
    "instagram.com",
    "facebook.com",
    "tiktok.com",
    "telegram.org",
    "t.me",
    "bilibili.com",
    "bsky.app",
    "twitch.tv",
    "youtube.com",
    "youtu.be",
    "reddit.com",
    "redd.it",
    "modrinth.com",
    "minecraft.net",
    "curseforge.com",
    "planetminecraft.com",
    "9minecraft.net",
    "mcmod.cn",
    "creativemode.net",
    "orcaclient.com",
    "autoforged.cn",
];
const SOURCE_REQUIRING_LICENSES: &[&str] = &[
    "GPL-2.0",
    "GPL-2.0+",
    "GPL-2.0-only",
    "GPL-2.0-or-later",
    "GPL-3.0",
    "GPL-3.0+",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "LGPL-2.1",
    "LGPL-2.1+",
    "LGPL-2.1-only",
    "LGPL-2.1-or-later",
    "LGPL-3.0",
    "LGPL-3.0+",
    "LGPL-3.0-only",
    "LGPL-3.0-or-later",
    "AGPL-3.0",
    "AGPL-3.0+",
    "AGPL-3.0-only",
    "AGPL-3.0-or-later",
    "MPL-2.0",
];

pub(super) fn validate(
    project: &crate::models::projects::Project,
    versions: &[Version],
) -> Vec<super::ProjectNag> {
    let mut nags = Vec::new();
    let is_minecraft_server = project.components.minecraft_server.is_some();

    if !is_minecraft_server && project.link_urls.is_empty() {
        nags.push(super::ProjectNag::new(
            ProjectNagKind::AddLinks,
            ProjectNagSeverity::Suggestion,
        ));
    }

    if is_minecraft_server && project.link_urls.is_empty() {
        nags.push(super::ProjectNag::new(
            ProjectNagKind::AddLinksServer,
            ProjectNagSeverity::Suggestion,
        ));
    }

    let link_count = project.link_urls.len();
    let unique_link_count = project
        .link_urls
        .values()
        .map(|link| link.url.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    if unique_link_count != link_count {
        nags.push(super::ProjectNag::new(
            ProjectNagKind::IdenticalLinks,
            ProjectNagSeverity::Required,
        ));
    }

    let uncommon_link_fields = [
        ("source", SOURCE_DOMAINS),
        ("issues", ISSUES_DOMAINS),
        ("discord", DISCORD_DOMAINS),
    ]
    .into_iter()
    .filter_map(|(key, domains)| {
        named_link_is_uncommon(project, key, domains).then_some(key)
    })
    .collect::<Vec<_>>();
    if !uncommon_link_fields.is_empty() {
        nags.push(
            super::ProjectNag::new(
                ProjectNagKind::VerifyExternalLinks,
                ProjectNagSeverity::Warning,
            )
            .with_details(
                serde_json::json!({ "fields": uncommon_link_fields }),
            ),
        );
    }

    let misused_discord_link_fields =
        ["source", "issues", "wiki", "site", "store"]
            .into_iter()
            .filter(|key| named_link(project, key).is_some_and(is_discord_link))
            .collect::<Vec<_>>();
    if !misused_discord_link_fields.is_empty() {
        nags.push(
            super::ProjectNag::new(
                ProjectNagKind::MisusedDiscordLink,
                ProjectNagSeverity::Required,
            )
            .with_details(serde_json::json!({
                "fields": misused_discord_link_fields,
            })),
        );
    }

    if let Some(url) = find_blocked_external_link(project) {
        nags.push(
            super::ProjectNag::new(
                ProjectNagKind::BannedLinkUsage,
                ProjectNagSeverity::Required,
            )
            .with_details(serde_json::json!({ "url": url })),
        );
    }

    let is_source_project = project
        .project_types
        .iter()
        .any(|project_type| project_type == "mod" || project_type == "plugin");
    let is_datapack = project
        .project_types
        .iter()
        .any(|project_type| project_type == "datapack");
    let has_source_link =
        named_link(project, "source").is_some_and(|url| !url.is_empty());
    let every_version_has_additional_files =
        versions.iter().all(|version| version.files.len() >= 2);

    if !is_datapack
        && is_source_project
        && SOURCE_REQUIRING_LICENSES.contains(&project.license.id.as_str())
        && !has_source_link
        && !every_version_has_additional_files
    {
        let (project_type, _) =
            LegacyProject::get_project_type(&project.project_types);
        nags.push(
            super::ProjectNag::new(
                ProjectNagKind::GplLicenseSourceRequired,
                ProjectNagSeverity::Required,
            )
            .with_details(serde_json::json!({ "project_type": project_type })),
        );
    }

    nags
}

fn named_link<'a>(
    project: &'a crate::models::projects::Project,
    key: &str,
) -> Option<&'a str> {
    project.link_urls.get(key).map(|link| link.url.as_str())
}

fn named_link_is_uncommon(
    project: &crate::models::projects::Project,
    key: &str,
    domains: &[&str],
) -> bool {
    named_link(project, key)
        .is_some_and(|url| !is_link_from_domains(url, domains))
}

fn is_discord_link(url: &str) -> bool {
    is_link_from_domains(url, DISCORD_DOMAINS)
}

fn find_blocked_external_link(
    project: &crate::models::projects::Project,
) -> Option<&str> {
    const LEGACY_LINK_KEYS: &[&str] = &["source", "issues", "wiki", "discord"];

    for key in LEGACY_LINK_KEYS {
        if let Some(url) =
            named_link(project, key).filter(|url| is_blocked_external_link(url))
        {
            return Some(url);
        }
    }
    if let Some(url) = project
        .license
        .url
        .as_deref()
        .filter(|url| is_blocked_external_link(url))
    {
        return Some(url);
    }

    let mut remaining_links = project
        .link_urls
        .iter()
        .filter(|(key, _)| !LEGACY_LINK_KEYS.contains(&key.as_str()))
        .collect::<Vec<_>>();
    remaining_links.sort_unstable_by_key(|(key, _)| key.as_str());
    remaining_links
        .into_iter()
        .map(|(_, link)| link.url.as_str())
        .find(|url| is_blocked_external_link(url))
}

fn is_blocked_external_link(url: &str) -> bool {
    let Some(hostname) = get_link_hostname(url) else {
        return false;
    };

    is_ip_address(&hostname)
        || URL_SHORTENERS
            .iter()
            .any(|domain| hostname_matches_domain(&hostname, domain))
        || BLOCKED_EXTERNAL_LINK_DOMAINS
            .iter()
            .any(|domain| hostname_matches_domain(&hostname, domain))
}

fn is_link_from_domains(url: &str, domains: &[&str]) -> bool {
    let Some(hostname) = get_link_hostname(url) else {
        return false;
    };

    domains
        .iter()
        .any(|domain| hostname_matches_domain(&hostname, domain))
}

fn get_link_hostname(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let hostname = parsed
        .host_str()?
        .trim_end_matches('.')
        .to_ascii_lowercase();

    (!hostname.is_empty()).then_some(hostname)
}

fn hostname_matches_domain(hostname: &str, domain: &str) -> bool {
    hostname == domain
        || hostname
            .strip_suffix(domain)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

fn is_ip_address(hostname: &str) -> bool {
    hostname
        .trim_start_matches('[')
        .trim_end_matches(']')
        .parse::<IpAddr>()
        .is_ok()
}
