use anyhow::{Context, Result, anyhow, ensure};
use std::collections::HashMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    AppState,
    model::{Processor, SidedDataEntry},
    upstream::mojang::{
        Argument, ArgumentType, LibraryDownloads, LibraryExtract,
        OperatingSystem, Rule, VersionType,
    },
    util::{MavenCoordinate, ResponseExt},
};

pub const GAME_VERSION_PLACEHOLDER: &str = "${modrinth.gameVersion}";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMetadata {
    pub id: String,
    pub inherits_from: String,
    pub release_time: Timestamp,
    pub time: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    pub libraries: Vec<Library>,
    #[serde(rename = "type")]
    pub ty: VersionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, SidedDataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<LibraryExtract>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<OperatingSystem, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub include_in_classpath: bool,
    #[serde(default = "default_true")]
    pub downloadable: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub coordinate: MavenCoordinate,
    pub repository: Url,
    pub source: Option<String>,
    pub repository_source: &'static str,
}

/// Normalizes a Fabric or Quilt profile for multiple game versions.
/// Artifacts are concrete; callers should deduplicate them by Maven path.
pub fn normalize_profile(
    mut profile: ProfileMetadata,
    template: &str,
    games: &[String],
    fallback_maven: &Url,
    public_maven: &Url,
) -> Result<(ProfileMetadata, Vec<Artifact>)> {
    profile.id = profile.id.replace(template, GAME_VERSION_PLACEHOLDER);
    profile.inherits_from = profile
        .inherits_from
        .replace(template, GAME_VERSION_PLACEHOLDER);

    let mut artifacts = Vec::new();
    for (index, library) in profile.libraries.iter_mut().enumerate() {
        let name = library.name.clone();
        let coordinate =
            name.parse::<MavenCoordinate>().with_context(|| {
                format!("library {index} has an invalid Maven name")
            })?;

        let repository_source = if name == "net.minecraft:launchwrapper:1.12" {
            "launchwrapper override"
        } else if library.url.is_some() {
            "profile library.url"
        } else {
            "loader Maven fallback"
        };
        let repository = if name == "net.minecraft:launchwrapper:1.12" {
            // special case this because it's not available on Fabric's Maven repo
            Url::parse("https://libraries.minecraft.net/")?
        } else {
            match &library.url {
				None => fallback_maven.clone(),
				Some(value) => Url::parse(value).with_context(|| {
					anyhow!(
						"library {index} has an invalid repository URL {value:?}"
					)
				})?,
			}
        };

        // rewrite the specific `template` (game version)
        // to `GAME_VERSION_PLACEHOLDER`, so that app consumer
        // can replace the placeholder to the real game version later.
        // we only rewrite the coordinate if it's one of the ones
        // that we know contains a game version
        // (like fabric intermediary coordinate).
        if matches!(
            (
                coordinate.group_id.as_str(),
                coordinate.artifact_id.as_str()
            ),
            ("net.fabricmc", "intermediary") | ("org.quiltmc", "hashed")
        ) {
            ensure!(
                coordinate.version.as_str() == template,
                "library {index} mapping version {} does not match template {template:?}",
                coordinate.version
            );
            for game in games {
                let mut concrete = coordinate.clone();
                concrete.version = game.parse().with_context(|| {
                    format!(
                        "library {index} has an invalid game version {game:?}"
                    )
                })?;
                artifacts.push(Artifact {
                    coordinate: concrete,
                    repository: repository.clone(),
                    source: None,
                    repository_source,
                });
            }
            let classifier = coordinate
                .classifier
                .as_ref()
                .map(|value| format!(":{value}"))
                .unwrap_or_default();
            let extension = coordinate
                .extension
                .as_ref()
                .map(|value| format!("@{value}"))
                .unwrap_or_default();
            library.name = format!(
                "{}:{}:{GAME_VERSION_PLACEHOLDER}{classifier}{extension}",
                coordinate.group_id, coordinate.artifact_id
            );
        } else {
            artifacts.push(Artifact {
                coordinate,
                repository,
                source: None,
                repository_source,
            });
        }

        library.url = Some(public_maven.to_string());
    }
    Ok((profile, artifacts))
}

fn artifact_url(artifact: &Artifact) -> Result<Url> {
    let mut url = artifact.repository.clone();

    // some old Fabric artifacts, 0.1.0.48 through 0.1.0.52, have libraries
    // which reference apache maven using HTTP.
    // apache maven no longer accepts that, and tells clients to upgrade.
    // we do this upgrade manually.
    if url.scheme() == "http" {
        warn!(
            %url,
            coordinate = %artifact.coordinate,
            source = artifact.source.as_deref().unwrap_or("unknown"),
            repository = %artifact.repository,
            repository_source = artifact.repository_source,
            "upgraded Maven artifact URL from HTTP to HTTPS",
        );

        url.set_scheme("https").map_err(|()| {
            anyhow!("cannot upgrade Maven repository URL to HTTPS")
        })?;
    }
    let path = artifact.coordinate.to_maven_path();
    let relative = path
        .strip_prefix("maven/")
        .context("missing Maven path prefix")?;
    url.path_segments_mut()
        .map_err(|()| {
            anyhow!("repository URL cannot be a base: {}", artifact.repository)
        })?
        .pop_if_empty()
        .extend(relative.split('/'));
    Ok(url)
}

/// Downloads a concrete dependency and stores it in the public Maven mirror.
pub async fn mirror_artifact(app: &AppState, artifact: Artifact) -> Result<()> {
    let url = artifact_url(&artifact)?;
    let bytes = async {
		app.http
			.get(url.clone())
			.send()
			.await?
			.error_for_status_ext()
			.await?
			.bytes()
			.await
			.map_err(anyhow::Error::from)
	}
	.context(info_span!("fetching Maven artifact", %url, coordinate = %artifact.coordinate, source = artifact.source.as_deref().unwrap_or("unknown"), repository = %artifact.repository, repository_source = artifact.repository_source, scheme = url.scheme()))
	.await?;

    app.maven
		.put(&artifact.coordinate, &bytes)
		.context(
			info_span!("storing Maven artifact", coordinate = %artifact.coordinate),
		)
		.await
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn normalize(
        mut profile: Value,
        games: &[String],
    ) -> Result<(Value, Vec<Artifact>)> {
        if let Some(object) = profile.as_object_mut() {
            object.insert("releaseTime".into(), json!("2026-01-01T00:00:00Z"));
            object.insert("time".into(), json!("2026-01-01T00:00:00Z"));
            object.insert("type".into(), json!("release"));
        }
        let (profile, artifacts) = normalize_profile(
            serde_json::from_value(profile)?,
            "GAME",
            games,
            &Url::parse("https://maven.fabricmc.net/").unwrap(),
            &Url::parse("https://example.com/maven/").unwrap(),
        )?;
        Ok((serde_json::to_value(profile)?, artifacts))
    }

    #[test]
    fn substitutes_templates_and_expands_concrete_artifacts() {
        let (profile, artifacts) = normalize(
            json!({
                "id": "fabric-GAME-loader",
                "inheritsFrom": "GAME",
                "libraries": [{"name": "net.fabricmc:intermediary:GAME"}]
            }),
            &["1.21.1".to_owned(), "1.14.2 Pre-Release 4".to_owned()],
        )
        .unwrap();
        assert_eq!(
            profile["id"],
            format!("fabric-{GAME_VERSION_PLACEHOLDER}-loader")
        );
        assert_eq!(profile["inheritsFrom"], GAME_VERSION_PLACEHOLDER);
        assert_eq!(
            profile["libraries"][0]["name"],
            format!("net.fabricmc:intermediary:{GAME_VERSION_PLACEHOLDER}")
        );

        assert_eq!(artifacts.len(), 2);
        assert_eq!(
            artifacts[0].coordinate.to_string(),
            "net.fabricmc:intermediary:1.21.1"
        );
        assert_eq!(
            artifacts[1].coordinate.to_string(),
            "net.fabricmc:intermediary:1.14.2 Pre-Release 4"
        );
        assert_eq!(
            artifacts[0].repository.as_str(),
            "https://maven.fabricmc.net/"
        );
    }

    #[test]
    fn only_mapping_coordinates_are_templated() {
        let (profile, artifacts) = normalize(
            json!({
                "id": "GAME", "inheritsFrom": "GAME",
                "libraries": [
                    {"name": "net.fabricmc:sponge-mixin:0.7.11.21"},
                    {"name": "net.fabricmc:sponge-mixin:GAME"},
                    {"name": "other:intermediary:GAME"},
                    {"name": "net.fabricmc:intermediary:GAME:GAME@jar"},
                    {"name": "org.quiltmc:hashed:GAME"}
                ]
            }),
            &["1.21.1".into(), "26.4-snapshot-3".into()],
        )
        .unwrap();
        assert_eq!(artifacts.len(), 7);
        for (index, name) in [
            "net.fabricmc:sponge-mixin:0.7.11.21",
            "net.fabricmc:sponge-mixin:GAME",
            "other:intermediary:GAME",
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(profile["libraries"][index]["name"], *name);
            assert_eq!(artifacts[index].coordinate.to_string(), *name);
        }
        assert_eq!(
            profile["libraries"][3]["name"],
            format!(
                "net.fabricmc:intermediary:{GAME_VERSION_PLACEHOLDER}:GAME@jar"
            )
        );
        assert_eq!(
            artifacts[3].coordinate.to_string(),
            "net.fabricmc:intermediary:1.21.1:GAME@jar"
        );
        assert_eq!(
            artifacts[4].coordinate.to_string(),
            "net.fabricmc:intermediary:26.4-snapshot-3:GAME@jar"
        );
        assert_eq!(
            profile["libraries"][4]["name"],
            format!("org.quiltmc:hashed:{GAME_VERSION_PLACEHOLDER}")
        );
    }

    #[test]
    fn rejects_mapping_versions_that_do_not_match_template() {
        assert!(
            normalize(
                json!({
                    "id": "GAME", "inheritsFrom": "GAME",
                    "libraries": [{"name": "net.fabricmc:intermediary:OTHER"}]
                }),
                &["1.21.1".into()]
            )
            .is_err()
        );
    }

    #[test]
    fn launchwrapper_uses_minecraft_repository() {
        let (profile, artifacts) = normalize(
            json!({
                "id": "GAME", "inheritsFrom": "GAME",
                "libraries": [{
                    "name": "net.minecraft:launchwrapper:1.12",
                    "url": "https://maven.fabricmc.net/"
                }]
            }),
            &[],
        )
        .unwrap();
        assert_eq!(
            artifacts[0].repository.as_str(),
            "https://libraries.minecraft.net/"
        );
        assert_eq!(
            profile["libraries"][0]["url"],
            "https://example.com/maven/"
        );
    }

    #[test]
    fn preserves_concrete_libraries_and_emits_daedalus_defaults() {
        let (profile, artifacts) = normalize(json!({
			"id": "loader", "inheritsFrom": "1.21.1",
			"libraries": [
				{"name": "org.quiltmc:quilt-loader:1", "url": "https://maven.quiltmc.org/repository/release/", "include_in_classpath": false, "downloadable": false, "rules": []},
				{"name": "net.fabricmc:fabric-loader:1"}
			]
		}), &["1.21.1".to_owned(), "1.21.2".to_owned()]).unwrap();
        assert_eq!(artifacts.len(), 2);
        assert_eq!(
            artifacts[0].repository.as_str(),
            "https://maven.quiltmc.org/repository/release/"
        );
        assert_eq!(
            artifacts[1].repository.as_str(),
            "https://maven.fabricmc.net/"
        );
        assert_eq!(
            profile["libraries"][0]["name"],
            "org.quiltmc:quilt-loader:1"
        );
        assert_eq!(profile["libraries"][0]["include_in_classpath"], false);
        assert_eq!(profile["libraries"][0]["downloadable"], false);
        assert_eq!(profile["libraries"][0]["rules"], json!([]));
        assert_eq!(profile["libraries"][1]["include_in_classpath"], true);
        assert_eq!(profile["libraries"][1]["downloadable"], true);
        assert!(profile.get("arguments").is_none());
        assert!(profile.get("data").is_none());
    }

    #[test]
    fn rejects_invalid_profiles_and_names() {
        for profile in [
            json!(null),
            json!({"id": 1, "inheritsFrom": "GAME", "libraries": []}),
            json!({"id": "GAME", "libraries": []}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": {}}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": [null]}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": [{"name": 1}]}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": [{"name": "bad/GAME:a:1"}]}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": [{"name": "g:a:${unknown}"}]}),
        ] {
            assert!(normalize(profile, &[]).is_err());
        }
    }

    #[test]
    fn typed_profiles_round_trip_templates_and_require_metadata() {
        let value = json!({
            "id": "fabric-${modrinth.gameVersion}",
            "inheritsFrom": "${modrinth.gameVersion}",
            "releaseTime": "2026-01-01T00:00:00Z",
            "time": "2026-01-01T00:00:00Z",
            "type": "release",
            "libraries": [{"name": "net.fabricmc:intermediary:${modrinth.gameVersion}"}]
        });
        let profile: ProfileMetadata =
            serde_json::from_value(value.clone()).unwrap();
        let output = serde_json::to_value(&profile).unwrap();
        assert_eq!(
            output["libraries"][0]["name"],
            value["libraries"][0]["name"]
        );
        assert!(profile.libraries[0].include_in_classpath);
        assert!(profile.libraries[0].downloadable);
        assert!(output.get("arguments").is_none());
        assert!(output["libraries"][0].get("downloads").is_none());
        let mut missing = value;
        missing.as_object_mut().unwrap().remove("releaseTime");
        assert!(serde_json::from_value::<ProfileMetadata>(missing).is_err());
    }

    #[test]
    fn records_profile_repository_selection_and_upgrades_http() {
        let (_, artifacts) = normalize(
            json!({
                "id": "GAME", "inheritsFrom": "GAME",
                "libraries": [{
                    "name": "com.github.zafarkhaja:java-semver:0.9.0",
                    "url": "http://repo.maven.apache.org/maven2/"
                }]
            }),
            &[],
        )
        .unwrap();
        assert_eq!(artifacts[0].repository_source, "profile library.url");
        assert_eq!(
            artifact_url(&artifacts[0]).unwrap().as_str(),
            "https://repo.maven.apache.org/maven2/com/github/zafarkhaja/java-semver/0.9.0/java-semver-0.9.0.jar"
        );
    }

    #[test]
    fn artifact_urls_append_encoded_segments_without_store_prefix() {
        let artifact = Artifact {
            coordinate: "net.fabricmc:intermediary:1.14.2 Pre-Release 4"
                .parse()
                .unwrap(),
            repository: Url::parse("https://example.com/repository/release")
                .unwrap(),
            source: None,
            repository_source: "test",
        };
        assert_eq!(
            artifact_url(&artifact).unwrap().as_str(),
            "https://example.com/repository/release/net/fabricmc/intermediary/1.14.2%20Pre-Release%204/intermediary-1.14.2%20Pre-Release%204.jar"
        );
    }
}
