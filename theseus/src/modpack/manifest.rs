use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
    str::FromStr,
};

use daedalus::download_file_mirrors;
use futures::future;
use tokio::fs;

use crate::launcher::ModLoader;

use super::ModpackError;

#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    format_version: u64,
    game: ModpackGame,
    version_id: String,

    name: String,
    summary: Option<String>,

    files: Vec<ModpackFile>,
}

impl Manifest {
    /// Download a modpack's files for a given side to a given destination
    /// Assumes the destination exists and is a directory
    pub async fn download_files(&self, dest: &Path, side: ModpackSide) -> Result<(), ModpackError> {
        let handles = self.files.clone().into_iter().map(move |file| {
            let (dest, side) = (dest.to_owned(), side);
            tokio::spawn(async move { file.fetch(&dest, side).await })
        });
        future::try_join_all(handles)
            .await?
            .into_iter()
            .collect::<Result<_, ModpackError>>()?;

        // TODO Integrate instance format to save other metadata
        Ok(())
    }
}

fn try_get<'r, F, T>(
    manifest: &'r serde_json::Map<String, serde_json::Value>,
    field: &str,
    caster: F,
) -> Result<T, ModpackError>
where
    F: Fn(&'r serde_json::Value) -> Option<T>,
{
    manifest
        .get(field)
        .and_then(caster)
        .ok_or(ModpackError::ManifestError(format!(
            "Invalid or missing field: {}",
            field
        )))
}

impl TryFrom<serde_json::Value> for Manifest {
    type Error = ModpackError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        use ModpackError::ManifestError;

        let value = value.as_object().ok_or(ManifestError(String::from(
            "Manifest is not a JSON object!",
        )))?;

        let game = ModpackGame::new(
            try_get(value, "game", serde_json::Value::as_str)?,
            try_get(value, "dependencies", serde_json::Value::as_object)?,
        )?;

        let files = try_get(value, "files", serde_json::Value::as_array)?
            .iter()
            .map(|it| -> Result<ModpackFile, ModpackError> {
                let file = it
                    .as_object()
                    .ok_or(ManifestError(String::from("Malformed file: not an object")))?;

                let path = Path::new(try_get(file, "path", serde_json::Value::as_str)?);
                let hashes = ModpackFileHashes::try_from(try_get(
                    file,
                    "hashes",
                    serde_json::Value::as_object,
                )?)?;
                let downloads = try_get(file, "downloads", serde_json::Value::as_array)?
                    .iter()
                    .map(serde_json::Value::as_str)
                    .map(|it| it.map(String::from))
                    .collect::<Option<Vec<String>>>()
                    .ok_or(ManifestError(format!(
                        "Invalid source for path {}",
                        path.to_str().unwrap_or("?")
                    )))?;
                let env: Option<[ModpackEnv; 2]> = if let Some(env) = file.get("env") {
                    if !env.is_object() {
                        return Err(ManifestError(String::from(
                            "Env is provided, but is not an object!",
                        )));
                    }
                    Some([
                        ModpackEnv::from_str(
                            env.get("client")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or_default(),
                        )?,
                        ModpackEnv::from_str(
                            env.get("server")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or_default(),
                        )?,
                    ])
                } else {
                    None
                };

                ModpackFile::new(path, hashes, env, &downloads)
            })
            .collect::<Result<Vec<ModpackFile>, ModpackError>>()?;

        Ok(Self {
            format_version: try_get(value, "formatVersion", serde_json::Value::as_u64)?,
            game,
            version_id: String::from(try_get(value, "versionId", serde_json::Value::as_str)?),
            name: String::from(try_get(value, "name", serde_json::Value::as_str)?),
            summary: value
                .get("summary")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            files,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModpackGame {
    // TODO: Currently, the launcher does not support specifying mod loader versions, so I just
    // store the loader here.
    Minecraft(String, ModLoader),
}

impl ModpackGame {
    pub fn new(
        game: &str,
        deps: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Self, ModpackError> {
        match game {
            "minecraft" => {
                let game_version = String::from(
                    deps.get("minecraft")
                        .ok_or(ModpackError::ManifestError(String::from(
                            "No version of minecraft given",
                        )))?
                        .as_str()
                        .unwrap(),
                );

                // TODO: See comment in ModpackGame, this code was designed specifically to be
                // easily adapted for versioned modloaders
                let loader = if let Some(_) = deps.get("fabric-loader") {
                    ModLoader::Fabric
                } else if let Some(_) = deps.get("forge") {
                    ModLoader::Forge
                } else {
                    ModLoader::Vanilla
                };

                Ok(ModpackGame::Minecraft(game_version, loader))
            }
            _ => Err(ModpackError::ManifestError(format!(
                "Invalid game: {}",
                game
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModpackFile {
    path: PathBuf,
    hashes: ModpackFileHashes,
    envs: Option<[ModpackEnv; 2]>,
    downloads: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModpackSide {
    Client = 0,
    Server = 1,
}

impl ModpackFile {
    pub fn new(
        path: &Path,
        hashes: ModpackFileHashes,
        envs: Option<[ModpackEnv; 2]>,
        downloads: &[String],
    ) -> Result<Self, ModpackError> {
        if path.is_dir() {
            return Err(ModpackError::ManifestError(format!(
                "Modpack file {} is a directory!",
                path.to_str().unwrap_or("?")
            )));
        }

        Ok(Self {
            path: PathBuf::from(path),
            hashes,
            envs,
            downloads: Vec::from(downloads),
        })
    }

    pub async fn fetch(&self, dest: &Path, side: ModpackSide) -> Result<(), ModpackError> {
        if let Some(envs) = &self.envs {
            if envs[side as usize] == ModpackEnv::Unsupported
                || envs[(side as usize + 1) % 2] == ModpackEnv::Required
            {
                return Ok(());
            }
        }

        let output = dest.join(&self.path);

        // HACK: Since Daedalus appends a file name to all mirrors and the manifest supplies full
        // URLs, I'm supplying it with an empty string to avoid reinventing the wheel.
        let bytes = download_file_mirrors(
            "",
            &self
                .downloads
                .iter()
                .map(|it| it.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
            Some(&self.hashes.sha1),
        )
        .await?;
        fs::create_dir_all(output.parent().unwrap()).await?;
        fs::write(output, bytes).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModpackFileHashes {
    sha1: String,
}

impl TryFrom<&serde_json::Map<String, serde_json::Value>> for ModpackFileHashes {
    type Error = ModpackError;

    fn try_from(value: &serde_json::Map<String, serde_json::Value>) -> Result<Self, Self::Error> {
        let sha1 = String::from(try_get(&value, "sha1", serde_json::Value::as_str)?);
        Ok(Self { sha1 })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModpackEnv {
    Required,
    Optional,
    Unsupported,
}

impl FromStr for ModpackEnv {
    type Err = ModpackError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ModpackEnv::*;
        match s {
            "required" => Ok(Required),
            "optional" => Ok(Optional),
            "unsupported" => Ok(Unsupported),
            _ => Err(ModpackError::ManifestError(format!(
                "Invalid environment support: {}",
                s
            ))),
        }
    }
}

impl Default for ModpackEnv {
    fn default() -> Self {
        Self::Optional
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() -> Result<(), ModpackError> {
        const PACK_JSON: &'static str = r#"
            {
                "formatVersion": 1,
                "game": "minecraft",
                "versionId": "deadbeef",
                "name": "Example Pack",
                "files": [],
                "dependencies": {
                    "minecraft": "1.17.1"
                }
            }
        "#;
        let expected_manifest = Manifest {
            format_version: 1,
            game: ModpackGame::Minecraft(String::from("1.17.1"), ModLoader::Vanilla),
            version_id: String::from("deadbeef"),
            name: String::from("Example Pack"),
            summary: None,
            files: Vec::new(),
        };
        let manifest_json: serde_json::Value =
            serde_json::from_str(PACK_JSON).expect("Error parsing pack JSON");
        let manifest = Manifest::try_from(manifest_json)?;

        assert_eq!(expected_manifest, manifest);
        Ok(())
    }

    #[test]
    fn parse_forge() -> Result<(), ModpackError> {
        const PACK_JSON: &'static str = r#"
            {
                "formatVersion": 1,
                "game": "minecraft",
                "versionId": "deadbeef",
                "name": "Example Pack",
                "files": [
                    {
                        "path": "mods/testmod.jar",
                        "hashes": {
                            "sha1": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        },
                        "downloads": [
                            "https://example.com/testmod.jar"
                        ]
                    }
                ],
                "dependencies": {
                    "minecraft": "1.17.1",
                    "forge": "37.0.110"
                }
            }
        "#;

        let expected_manifest = Manifest {
            format_version: 1,
            game: ModpackGame::Minecraft(String::from("1.17.1"), ModLoader::Forge),
            version_id: String::from("deadbeef"),
            name: String::from("Example Pack"),
            summary: None,
            files: vec![ModpackFile::new(
                Path::new("mods/testmod.jar"),
                ModpackFileHashes {
                    sha1: String::from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                },
                None,
                &[String::from("https://example.com/testmod.jar")],
            )?],
        };

        let manifest_json: serde_json::Value =
            serde_json::from_str(PACK_JSON).expect("Error parsing pack JSON");
        let manifest = Manifest::try_from(manifest_json)?;

        assert_eq!(expected_manifest, manifest);
        Ok(())
    }

    #[test]
    fn parse_fabric() -> Result<(), ModpackError> {
        const PACK_JSON: &'static str = r#"
            {
                "formatVersion": 1,
                "game": "minecraft",
                "versionId": "deadbeef",
                "name": "Example Pack",
                "files": [
                    {
                        "path": "mods/testmod.jar",
                        "hashes": {
                            "sha1": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        },
                        "downloads": [
                            "https://example.com/testmod.jar"
                        ]
                    }
                ],
                "dependencies": {
                    "minecraft": "1.17.1",
                    "fabric-loader": "0.9.0"
                }
            }
        "#;

        let expected_manifest = Manifest {
            format_version: 1,
            game: ModpackGame::Minecraft(String::from("1.17.1"), ModLoader::Fabric),
            version_id: String::from("deadbeef"),
            name: String::from("Example Pack"),
            summary: None,
            files: vec![ModpackFile::new(
                Path::new("mods/testmod.jar"),
                ModpackFileHashes {
                    sha1: String::from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                },
                None,
                &[String::from("https://example.com/testmod.jar")],
            )?],
        };

        let manifest_json: serde_json::Value =
            serde_json::from_str(PACK_JSON).expect("Error parsing pack JSON");
        let manifest = Manifest::try_from(manifest_json)?;

        assert_eq!(expected_manifest, manifest);
        Ok(())
    }

    #[test]
    fn parse_complete() -> Result<(), ModpackError> {
        const PACK_JSON: &'static str = r#"
            {
                "formatVersion": 1,
                "game": "minecraft",
                "versionId": "deadbeef",
                "name": "Example Pack",
                "summary": "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                "files": [
                    {
                        "path": "mods/testmod.jar",
                        "hashes": {
                            "sha1": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        },
                        "env": {
                            "client": "required",
                            "server": "unsupported"
                        },
                        "downloads": [
                            "https://example.com/testmod.jar"
                        ]
                    }
                ],
                "dependencies": {
                    "minecraft": "1.17.1",
                    "forge": "37.0.110"
                }
            }
        "#;

        let expected_manifest = Manifest {
            format_version: 1,
            game: ModpackGame::Minecraft(String::from("1.17.1"), ModLoader::Forge),
            version_id: String::from("deadbeef"),
            name: String::from("Example Pack"),
            summary: Some(String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.")),
            files: vec![
                ModpackFile::new(
                    Path::new("mods/testmod.jar"), 
                    ModpackFileHashes {
                        sha1: String::from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                    },
                    Some([ ModpackEnv::Required, ModpackEnv::Unsupported ]),
                    &[ String::from("https://example.com/testmod.jar") ],
                )?
            ],
        };

        let manifest_json: serde_json::Value =
            serde_json::from_str(PACK_JSON).expect("Error parsing pack JSON");
        let manifest = Manifest::try_from(manifest_json)?;

        assert_eq!(expected_manifest, manifest);
        Ok(())
    }
}
