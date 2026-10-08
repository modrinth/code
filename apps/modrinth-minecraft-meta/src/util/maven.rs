use std::{str::FromStr, sync::LazyLock};

use anyhow::{Context, ensure};
use derive_more::Display;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::util::de_error;

#[derive(Debug, Clone)]
pub struct MavenCoordinate {
    pub group_id: CoordinatePart,
    pub artifact_id: CoordinatePart,
    pub version: CoordinatePart,
    pub classifier: Option<CoordinatePart>,
    pub extension: Option<CoordinatePart>,
}

impl MavenCoordinate {
    pub fn to_maven_path(&self) -> String {
        let group_id = self.group_id.as_str().replace('.', "/");
        let artifact_id = self.artifact_id.as_str();
        let version = self.version.as_str();
        let classifier = self
            .classifier
            .as_ref()
            .map(|classifier| format!("-{classifier}"))
            .unwrap_or_default();
        let extension = self
            .extension
            .as_ref()
            .map(CoordinatePart::as_str)
            .unwrap_or("jar");
        format!(
            "maven/{group_id}/{artifact_id}/{version}/{artifact_id}-{version}{classifier}.{extension}"
        )
    }

    pub fn from_maven_path(path: &str) -> anyhow::Result<Self> {
        let relative = path
            .strip_prefix("maven/")
            .context("missing maven/ prefix")?;
        let (directories, filename) = relative
            .rsplit_once('/')
            .context("missing artifact filename")?;
        let (directories, version) = directories
            .rsplit_once('/')
            .context("missing version directory")?;
        let (group, artifact_id) = directories
            .rsplit_once('/')
            .context("missing artifact directory")?;
        for segment in group.split('/') {
            segment
                .parse::<CoordinatePart>()
                .context("invalid group directory")?;
            ensure!(
                !segment.contains('.'),
                "group directory contains a dot: {segment:?}"
            );
        }
        let group_id = group
            .replace('/', ".")
            .parse()
            .context("invalid group ID")?;
        let artifact_id: CoordinatePart =
            artifact_id.parse().context("invalid artifact ID")?;
        let version: CoordinatePart =
            version.parse().context("invalid version")?;
        let prefix = format!("{artifact_id}-{version}");
        let suffix = filename
            .strip_prefix(&prefix)
            .context("filename does not match artifact and version")?;
        let (classifier, extension) =
            if let Some(extension) = suffix.strip_prefix('.') {
                (None, extension)
            } else {
                let suffix = suffix
                    .strip_prefix('-')
                    .context("invalid artifact filename suffix")?;
                let (classifier, extension) = suffix
                    .rsplit_once('.')
                    .context("missing artifact extension")?;
                (
                    Some(classifier.parse().context("invalid classifier")?),
                    extension,
                )
            };
        Ok(Self {
            group_id,
            artifact_id,
            version,
            classifier,
            extension: Some(extension.parse().context("invalid extension")?),
        })
    }
}

impl std::fmt::Display for MavenCoordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.group_id, self.artifact_id, self.version)?;
        if let Some(classifier) = &self.classifier {
            write!(f, ":{classifier}")?;
        }
        if let Some(extension) = &self.extension {
            write!(f, "@{extension}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
pub struct CoordinatePart(String);

impl CoordinatePart {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for CoordinatePart {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"\A[A-Za-z0-9_+ -]+(?:\.[A-Za-z0-9_+ -]+)*\z")
                .expect("valid coordinate part regex")
        });
        ensure!(PATTERN.is_match(s), "invalid Maven coordinate part: {s:?}");

        Ok(Self(s.to_owned()))
    }
}

impl FromStr for MavenCoordinate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (coordinate, extension) = match s.split_once('@') {
            Some((coordinate, extension)) => (
                coordinate,
                Some(extension.parse().context("invalid extension")?),
            ),
            None => (s, None),
        };
        let (group_id, remaining) = coordinate
            .split_once(':')
            .context("missing group ID separator")?;
        let (artifact_id, remaining) = remaining
            .split_once(':')
            .context("missing artifact ID separator")?;
        let (version, classifier) = match remaining.split_once(':') {
            Some((version, classifier)) => (
                version,
                Some(classifier.parse().context("invalid classifier")?),
            ),
            None => (remaining, None),
        };
        Ok(Self {
            group_id: group_id.parse().context("invalid group ID")?,
            artifact_id: artifact_id.parse().context("invalid artifact ID")?,
            version: version.parse().context("invalid version")?,
            classifier,
            extension,
        })
    }
}

impl Serialize for MavenCoordinate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MavenCoordinate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Self>().map_err(de_error::<D>)
    }
}

#[cfg(test)]
mod tests {
    use super::{CoordinatePart, MavenCoordinate};

    #[test]
    fn maven_paths_round_trip() {
        for (input, path) in [
            (
                "net.fabricmc:intermediary:1.14.2 Pre-Release 4",
                "maven/net/fabricmc/intermediary/1.14.2 Pre-Release 4/intermediary-1.14.2 Pre-Release 4.jar",
            ),
            (
                "net.fabricmc:intermediary:3D Shareware v1.34",
                "maven/net/fabricmc/intermediary/3D Shareware v1.34/intermediary-3D Shareware v1.34.jar",
            ),
            (
                "net.fabricmc:intermediary:1.21.1+build.3",
                "maven/net/fabricmc/intermediary/1.21.1+build.3/intermediary-1.21.1+build.3.jar",
            ),
            (
                "net.minecraftforge:forge:1.21.8-58.1.22",
                "maven/net/minecraftforge/forge/1.21.8-58.1.22/forge-1.21.8-58.1.22.jar",
            ),
            (
                "com.example:tool:1:client@lzma",
                "maven/com/example/tool/1/tool-1-client.lzma",
            ),
            (
                "com.example:tool:1:client.extra@jar",
                "maven/com/example/tool/1/tool-1-client.extra.jar",
            ),
            (
                "com.example:tool:1@zip",
                "maven/com/example/tool/1/tool-1.zip",
            ),
        ] {
            let coordinate: MavenCoordinate = input.parse().unwrap();
            assert_eq!(coordinate.to_maven_path(), path);
            assert_eq!(
                MavenCoordinate::from_maven_path(path)
                    .unwrap()
                    .to_maven_path(),
                path
            );
        }
    }

    #[test]
    fn invalid_maven_paths_are_rejected() {
        for path in [
            "maven/../tool/1/tool-1.jar",
            "maven/com//example/tool/1/tool-1.jar",
            "maven/com.example/tool/1/tool-1.jar",
            "maven/com/example/tool/1/other-1.jar",
            "maven/com/example/tool/1/tool-1-.jar",
            "maven/com/example/tool/1/tool-1",
            "/maven/com/example/tool/1/tool-1.jar",
        ] {
            assert!(
                MavenCoordinate::from_maven_path(path).is_err(),
                "accepted {path:?}"
            );
        }
    }

    #[test]
    fn coordinate_parts_are_validated() {
        for value in [
            "net.minecraftforge",
            "fabric-loader",
            "1.21.8-58.1.22",
            "a_b",
            "1+2",
            "1.21.1+build.3",
            "1.14.2 Pre-Release 4",
            "3D Shareware v1.34",
        ] {
            assert_eq!(
                value.parse::<CoordinatePart>().unwrap().as_str(),
                value
            );
        }
        for value in [
            "", ".", "..", ".foo", "foo.", "foo..bar", "a/b", "a\\b", "a:b",
            "a\t", "a\n",
        ] {
            assert!(
                value.parse::<CoordinatePart>().is_err(),
                "accepted {value:?}"
            );
        }
    }

    #[test]
    fn coordinates_round_trip_and_validate_each_field() {
        for input in [
            "net.minecraftforge:forge:1.21.8-58.1.22",
            "net.minecraftforge:forge:1.21.8-58.1.22:universal",
            "net.minecraftforge:forge:1.21.8-58.1.22@zip",
            "net.minecraftforge:forge:1.21.8-58.1.22:client@lzma",
            "net.fabricmc:intermediary:1.21.1+build.3",
            "net.fabricmc:intermediary:1.14.2 Pre-Release 4",
            "net.fabricmc:intermediary:3D Shareware v1.34",
        ] {
            let coordinate: MavenCoordinate =
                serde_json::from_value(serde_json::json!(input)).unwrap();
            assert_eq!(coordinate.to_string(), input);
            assert_eq!(serde_json::to_value(coordinate).unwrap(), input);
        }
        for value in [
            "bad/group:forge:1",
            "net.minecraftforge:..:1",
            "net.minecraftforge:forge:",
            "g:a:1:",
            "g:a:1@",
            "g:a:1:c:extra",
            "g:a:1@jar@zip",
            "g:a:1:../client",
            "g:a:1@../jar",
        ] {
            assert!(value.parse::<MavenCoordinate>().is_err());
        }
    }
}
