use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, ensure};

use crate::{
    model::{ForgelikeInstallerName, ForgelikeLoader, ProfileMetadata},
    upstream::mojang::Library,
    util::MavenCoordinate,
};

/// Rewrites installer-relative data into libraries without storing their bytes.
/// All paths and coordinates are validated before the profile is changed.
pub(super) fn normalize(
    profile: &mut ProfileMetadata,
    loader: ForgelikeLoader,
    name: &ForgelikeInstallerName,
    entries: &HashSet<&str>,
) -> Result<Vec<(MavenCoordinate, String)>> {
    let Some(mut data) = profile.data.clone() else {
        return Ok(Vec::new());
    };
    let loader = match loader {
        ForgelikeLoader::Forge => "forge",
        ForgelikeLoader::Neoforge => "neoforge",
    };
    let mut artifacts = Vec::new();
    let mut paths = HashMap::new();
    let mut coordinates = HashMap::new();
    let mut libraries = Vec::new();
    let mut keys = data.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    for key in keys {
        let entry = data.get_mut(&key).expect("existing data key");
        for (side, value) in
            [("client", &mut entry.client), ("server", &mut entry.server)]
        {
            let Some(path) = value.strip_prefix('/') else {
                continue;
            };
            ensure!(
                entries.contains(path),
                "missing installer data for {key} ({side}): {value}"
            );
            if let Some(reference) = paths.get(path) {
                *value = String::clone(reference);
                continue;
            }
            let filename =
                path.rsplit('/').next().context("missing data filename")?;
            let mut parts = filename.split('.');
            let stem = parts.next().context("missing data filename stem")?;
            let extension =
                parts.next().context("missing data filename extension")?;
            let coordinate: MavenCoordinate = format!(
                "com.modrinth.daedalus:{loader}-installer-extracts:{}:{stem}@{extension}",
                name.as_str()
            )
            .parse()
            .with_context(|| format!("invalid installer data coordinate for {key} ({side}): {value}"))?;
            let maven_path = coordinate.to_maven_path();
            if let Some(existing) =
                coordinates.insert(maven_path.clone(), path.to_owned())
            {
                ensure!(
                    existing == path,
                    "installer data coordinate collision: {existing} and {path}"
                );
            }
            ensure!(
                !profile.libraries.iter().any(|library| library
                    .name
                    .parse::<MavenCoordinate>()
                    .is_ok_and(
                        |existing| existing.to_maven_path() == maven_path
                    )),
                "installer data coordinate conflicts with existing library: {coordinate}"
            );
            let reference = format!("[{coordinate}]");
            paths.insert(path.to_owned(), reference.clone());
            libraries.push(Library {
                name: coordinate.to_string(),
                downloads: None,
                extract: None,
                url: None,
                natives: None,
                rules: None,
                checksums: None,
                include_in_classpath: false,
                downloadable: true,
            });
            artifacts.push((coordinate, path.to_owned()));
            *value = reference;
        }
    }
    profile.data = Some(data);
    profile.libraries.extend(libraries);
    Ok(artifacts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn profile(client: &str, server: &str) -> ProfileMetadata {
        serde_json::from_value(json!({
            "id": "test", "inheritsFrom": "1.20.3", "type": "release",
            "time": "2024-01-01T00:00:00Z", "releaseTime": "2024-01-01T00:00:00Z",
            "libraries": [], "data": {"BINPATCH": {"client": client, "server": server}}
        })).unwrap()
    }

    fn normalize_forge(
        profile: &mut ProfileMetadata,
        entries: &[&str],
    ) -> Result<Vec<(MavenCoordinate, String)>> {
        normalize(
            profile,
            ForgelikeLoader::Forge,
            &"1.20.3-49.0.1".parse::<ForgelikeInstallerName>().unwrap(),
            &entries.iter().copied().collect(),
        )
    }

    #[test]
    fn sides_and_coordinate_names_preserve_legacy_namespace() {
        let mut profile = profile("/data/client.lzma", "/data/server.lzma");
        let artifacts = normalize_forge(
            &mut profile,
            &["data/client.lzma", "data/server.lzma"],
        )
        .unwrap();
        assert_eq!(artifacts.len(), 2);
        let data = &profile.data.as_ref().unwrap()["BINPATCH"];
        assert_eq!(
            data.client,
            "[com.modrinth.daedalus:forge-installer-extracts:1.20.3-49.0.1:client@lzma]"
        );
        assert_eq!(
            data.server,
            "[com.modrinth.daedalus:forge-installer-extracts:1.20.3-49.0.1:server@lzma]"
        );
        assert_eq!(artifacts[0].1, "data/client.lzma");
        assert!(profile.libraries.iter().all(|lib| lib.downloadable
            && !lib.include_in_classpath
            && lib.url.is_none()
            && lib.downloads.is_none()));
    }

    #[test]
    fn repeated_files_are_deduplicated_across_sides_and_keys() {
        let mut profile = profile("/data/client.lzma", "/data/client.lzma");
        let entry = profile.data.as_ref().unwrap()["BINPATCH"].clone();
        profile.data.as_mut().unwrap().insert("OTHER".into(), entry);
        assert_eq!(
            normalize_forge(&mut profile, &["data/client.lzma"])
                .unwrap()
                .len(),
            1
        );
        assert_eq!(profile.libraries.len(), 1);
        let data = profile.data.as_ref().unwrap();
        assert_eq!(data["BINPATCH"].client, data["OTHER"].server);
    }

    #[test]
    fn nonpaths_and_absent_data_are_preserved() {
        for value in [
            "[net.example:artifact:1]",
            "'literal'",
            "{MINECRAFT_JAR}",
            "123",
            "",
            "relative/file.jar",
        ] {
            let mut profile = profile(value, value);
            let before = serde_json::to_value(&profile).unwrap();
            assert!(normalize_forge(&mut profile, &[]).unwrap().is_empty());
            assert_eq!(serde_json::to_value(profile).unwrap(), before);
        }
        let mut profile = profile("", "");
        profile.data = None;
        assert!(normalize_forge(&mut profile, &[]).unwrap().is_empty());
        assert!(profile.data.is_none());
    }

    #[test]
    fn neoforge_and_multidot_names_preserve_legacy_namespace() {
        let mut profile = profile("/data/client.extra.lzma", "'scalar'");
        let artifacts = normalize(
            &mut profile,
            ForgelikeLoader::Neoforge,
            &"21.0.1-beta".parse::<ForgelikeInstallerName>().unwrap(),
            &HashSet::from(["data/client.extra.lzma"]),
        )
        .unwrap();
        assert_eq!(
            artifacts[0].0.to_string(),
            "com.modrinth.daedalus:neoforge-installer-extracts:21.0.1-beta:client@extra"
        );
        assert_eq!(profile.data.unwrap()["BINPATCH"].server, "'scalar'");
    }

    #[test]
    fn invalid_or_missing_data_does_not_change_profile() {
        for (client, server, entries) in [
            (
                "/data/client.lzma",
                "/missing.lzma",
                vec!["data/client.lzma"],
            ),
            ("/data/noextension", "", vec!["data/noextension"]),
            ("/data/.lzma", "", vec!["data/.lzma"]),
            ("/data/client.", "", vec!["data/client."]),
            ("/", "", vec![]),
            (
                "/a/client.lzma",
                "/b/client.lzma",
                vec!["a/client.lzma", "b/client.lzma"],
            ),
        ] {
            let mut profile = profile(client, server);
            let before = serde_json::to_value(&profile).unwrap();
            assert!(normalize_forge(&mut profile, &entries).is_err());
            assert_eq!(serde_json::to_value(profile).unwrap(), before);
        }
    }
}
