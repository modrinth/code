use std::path::PathBuf;

use std::convert::TryFrom;

use crate::launcher::ModLoader;

use super::pack::ModpackGame;
use super::{pack, ModpackError, ModpackResult};
use serde::{Deserialize, Serialize};

pub const DEFAULT_FORMAT_VERSION: u32 = 1;
const MODRINTH_GAMEDATA_URL: &str = "https://staging-cdn.modrinth.com/gamedata";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<ManifestFile>,
    pub dependencies: ManifestDeps,
}

impl TryFrom<Manifest> for pack::Modpack {
    type Error = ModpackError;

    fn try_from(manifest: Manifest) -> Result<Self, Self::Error> {
        let files = manifest
            .files
            .into_iter()
            .map(pack::ModpackFile::try_from)
            .collect::<ModpackResult<_>>()?;

        Ok(Self {
            name: manifest.name,
            version: manifest.version_id,
            summary: manifest.summary,
            game: ModpackGame::from(manifest.dependencies),
            files,
        })
    }
}

fn get_loader_version(loader: ModLoader, version: &str) -> ModpackResult<String> {
    let source = match loader {
        ModLoader::Vanilla => Err(ModpackError::VersionError(String::from(
            "Attempted to get mod loader version of Vanilla",
        ))),
        ModLoader::Forge => Ok(format!("{MODRINTH_GAMEDATA_URL}/forge/v0/manifest.json")),
        ModLoader::Fabric => Ok(format!("{MODRINTH_GAMEDATA_URL}/fabric/v0/manifest.json")),
    }?;
    let manifest = futures::executor::block_on(daedalus::modded::fetch_manifest(&source))?;

    let version = manifest
        .game_versions
        .iter()
        .find(|&it| it.id == version)
        .map(|x| x.loaders.first())
        .flatten()
        .ok_or_else(|| {
            ModpackError::VersionError(format!(
                "No versions of modloader {loader:?} exist for Minecraft {version}",
            ))
        })?;
    Ok(version.id.clone())
}

impl TryFrom<pack::Modpack> for Manifest {
    type Error = ModpackError;

    fn try_from(pack: pack::Modpack) -> Result<Self, Self::Error> {
        let pack::Modpack {
            game,
            version,
            name,
            summary,
            files,
        } = pack;

        let game_name = match &game {
            ModpackGame::Minecraft(..) => "minecraft".into(),
        };

        let files: Vec<_> = files.into_iter().map(ManifestFile::from).collect();

        Ok(Manifest {
            format_version: DEFAULT_FORMAT_VERSION,
            game: game_name,
            version_id: version,
            name,
            summary,
            files,
            dependencies: ManifestDeps::try_from(game)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFile {
    pub path: PathBuf,
    pub hashes: Option<ManifestHashes>,
    #[serde(default)]
    pub env: ManifestEnvs,
    pub downloads: Vec<String>,
}

impl TryFrom<ManifestFile> for pack::ModpackFile {
    type Error = ModpackError;

    fn try_from(file: ManifestFile) -> Result<Self, Self::Error> {
        Ok(Self {
            path: file.path,
            hashes: file.hashes.map(pack::ModpackFileHashes::from),
            env: pack::ModpackEnv::try_from(file.env)?,
            downloads: file.downloads.into_iter().collect(),
        })
    }
}

impl From<pack::ModpackFile> for ManifestFile {
    fn from(file: pack::ModpackFile) -> Self {
        Self {
            path: file.path,
            hashes: file.hashes.map(ManifestHashes::from),
            env: file.env.into(),
            downloads: file.downloads.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ManifestHashes {
    pub sha1: String,
}

impl From<ManifestHashes> for pack::ModpackFileHashes {
    fn from(hashes: ManifestHashes) -> Self {
        Self { sha1: hashes.sha1 }
    }
}

impl From<pack::ModpackFileHashes> for ManifestHashes {
    fn from(hashes: pack::ModpackFileHashes) -> Self {
        Self { sha1: hashes.sha1 }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct ManifestEnvs {
    pub client: ManifestEnv,
    pub server: ManifestEnv,
}

impl Default for ManifestEnvs {
    fn default() -> Self {
        Self {
            client: ManifestEnv::Optional,
            server: ManifestEnv::Optional,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ManifestEnv {
    Required,
    Optional,
    Unsupported,
}

impl TryFrom<ManifestEnvs> for pack::ModpackEnv {
    type Error = ModpackError;

    fn try_from(envs: ManifestEnvs) -> Result<Self, Self::Error> {
        use ManifestEnv::*;

        match (envs.client, envs.server) {
            (Required, Unsupported) => Ok(Self::ClientOnly),
            (Unsupported, Required) => Ok(Self::ServerOnly),
            (Optional, Optional) => Ok(Self::Both),
            _ => Err(ModpackError::FormatError(format!(
                "Invalid environment specification: {:?}",
                envs
            ))),
        }
    }
}

impl From<pack::ModpackEnv> for ManifestEnvs {
    fn from(envs: pack::ModpackEnv) -> Self {
        use super::pack::ModpackEnv::*;

        let (client, server) = match envs {
            ClientOnly => (ManifestEnv::Required, ManifestEnv::Unsupported),
            ServerOnly => (ManifestEnv::Unsupported, ManifestEnv::Required),
            Both => (ManifestEnv::Optional, ManifestEnv::Optional),
        };

        Self { client, server }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ManifestDeps {
    MinecraftFabric {
        minecraft: String,
        #[serde(rename = "fabric-loader")]
        fabric_loader: String,
    },
    MinecraftForge {
        minecraft: String,
        forge: String,
    },
    MinecraftVanilla {
        minecraft: String,
    },
}

impl From<ManifestDeps> for pack::ModpackGame {
    fn from(deps: ManifestDeps) -> Self {
        use ManifestDeps::*;

        match deps {
            MinecraftVanilla { minecraft } => Self::Minecraft(minecraft, ModLoader::Vanilla),
            MinecraftFabric { minecraft, .. } => Self::Minecraft(minecraft, ModLoader::Fabric),
            MinecraftForge { minecraft, .. } => Self::Minecraft(minecraft, ModLoader::Forge),
        }
    }
}

impl TryFrom<pack::ModpackGame> for ManifestDeps {
    type Error = ModpackError;

    fn try_from(game: pack::ModpackGame) -> Result<Self, Self::Error> {
        use super::pack::ModpackGame::*;
        Ok(match game {
            Minecraft(minecraft, ModLoader::Vanilla) => Self::MinecraftVanilla { minecraft },
            Minecraft(minecraft, ModLoader::Fabric) => Self::MinecraftFabric {
                fabric_loader: get_loader_version(ModLoader::Fabric, &minecraft)?,
                minecraft,
            },
            Minecraft(minecraft, ModLoader::Forge) => Self::MinecraftForge {
                forge: get_loader_version(ModLoader::Fabric, &minecraft)?,
                minecraft,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() -> ModpackResult<()> {
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
            game: "minecraft".into(),
            version_id: "deadbeef".into(),
            name: "Example Pack".into(),
            summary: None,
            files: vec![],
            dependencies: ManifestDeps::MinecraftVanilla {
                minecraft: "1.17.1".into(),
            },
        };
        let manifest: Manifest = serde_json::from_str(PACK_JSON).expect("Error parsing pack JSON");

        assert_eq!(expected_manifest, manifest);
        Ok(())
    }

    #[test]
    fn parse_forge() -> ModpackResult<()> {
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
            game: "minecraft".into(),
            version_id: "deadbeef".into(),
            name: "Example Pack".into(),
            summary: None,
            files: vec![ManifestFile {
                path: "mods/testmod.jar".into(),
                hashes: Some(ManifestHashes {
                    sha1: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                }),
                env: ManifestEnvs::default(),
                downloads: vec!["https://example.com/testmod.jar".into()],
            }],
            dependencies: ManifestDeps::MinecraftForge {
                minecraft: "1.17.1".into(),
                forge: "37.0.110".into(),
            },
        };
        let manifest: Manifest = serde_json::from_str(PACK_JSON).expect("Error parsing pack JSON");

        assert_eq!(expected_manifest, manifest);
        Ok(())
    }

    #[test]
    fn parse_fabric() -> ModpackResult<()> {
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
            game: "minecraft".into(),
            version_id: "deadbeef".into(),
            name: "Example Pack".into(),
            summary: None,
            files: vec![ManifestFile {
                path: "mods/testmod.jar".into(),
                hashes: Some(ManifestHashes {
                    sha1: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                }),
                env: ManifestEnvs::default(),
                downloads: vec!["https://example.com/testmod.jar".into()],
            }],
            dependencies: ManifestDeps::MinecraftFabric {
                minecraft: "1.17.1".into(),
                fabric_loader: "0.9.0".into(),
            },
        };
        let manifest: Manifest = serde_json::from_str(PACK_JSON).expect("Error parsing pack JSON");

        assert_eq!(expected_manifest, manifest);
        Ok(())
    }

    #[test]
    fn parse_complete() -> ModpackResult<()> {
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
            game: "minecraft".into(),
            version_id: "deadbeef".into(),
            name: "Example Pack".into(),
            summary: Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".into()),
            files: vec![ManifestFile {
                path: "mods/testmod.jar".into(),
                hashes: Some(ManifestHashes {
                    sha1: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                }),
                env: ManifestEnvs {
                    client: ManifestEnv::Required,
                    server: ManifestEnv::Unsupported,
                },
                downloads: vec!["https://example.com/testmod.jar".into()],
            }],
            dependencies: ManifestDeps::MinecraftForge {
                minecraft: "1.17.1".into(),
                forge: "37.0.110".into(),
            },
        };
        let manifest: Manifest = serde_json::from_str(PACK_JSON).expect("Error parsing pack JSON");

        assert_eq!(expected_manifest, manifest);
        Ok(())
    }
}
