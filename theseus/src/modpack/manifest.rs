use std::collections::HashSet;
use std::path::{Path, PathBuf};

use std::convert::TryFrom;

use crate::launcher::ModLoader;

use super::pack::ModpackGame;
use super::{pack, ModpackError, ModpackResult};
use daedalus::modded::LoaderType;
use serde::{Deserialize, Serialize};

pub const DEFAULT_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Manifest<'a> {
    pub format_version: u32,
    pub game: &'a str,
    pub version_id: &'a str,
    pub name: &'a str,
    #[serde(borrow)]
    pub summary: Option<&'a str>,
    pub files: Vec<ManifestFile<'a>>,
    pub dependencies: ManifestDeps<'a>,
}

impl TryFrom<Manifest<'_>> for pack::Modpack {
    type Error = ModpackError;

    fn try_from(manifest: Manifest<'_>) -> Result<Self, Self::Error> {
        let files = manifest
            .files
            .into_iter()
            .map(pack::ModpackFile::try_from)
            .collect::<ModpackResult<HashSet<pack::ModpackFile>>>()?;

        Ok(Self {
            name: String::from(manifest.name),
            version: String::from(manifest.version_id),
            summary: manifest.summary.map(String::from),
            game: ModpackGame::from(manifest.dependencies),
            files,
        })
    }
}

const MODRINTH_GAMEDATA_URL: &'static str = "https://staging-cdn.modrinth.com/gamedata";
fn get_loader_version(loader: ModLoader, version: &str) -> ModpackResult<String> {
    let source = match loader {
        ModLoader::Vanilla => Err(ModpackError::VersionError(String::from(
            "Attempted to get mod loader version of Vanilla",
        ))),
        ModLoader::Forge => Ok(format!("{}/forge/v0/manifest.json", MODRINTH_GAMEDATA_URL)),
        ModLoader::Fabric => Ok(format!("{}/fabric/v0/manifest.json", MODRINTH_GAMEDATA_URL)),
    }?;
    let manifest = futures::executor::block_on(daedalus::modded::fetch_manifest(&source))?;

    Ok(manifest
        .game_versions
        .iter()
        .find(|&it| it.id == version)
        .ok_or(ModpackError::VersionError(format!(
            "No versions of modloader {:?} exist for Minecraft {}",
            loader, version
        )))?
        .loaders[&LoaderType::Latest]
        .id
        .clone())
}

impl<'a> TryFrom<&'a pack::Modpack> for Manifest<'a> {
    type Error = ModpackError;

    fn try_from(pack: &'a pack::Modpack) -> Result<Self, Self::Error> {
        let game_field: &'a str = match pack.game {
            ModpackGame::Minecraft(..) => "minecraft",
        };

        let files = pack
            .files
            .iter()
            .map(ManifestFile::from)
            .collect::<Vec<ManifestFile>>();

        Ok(Manifest {
            format_version: DEFAULT_FORMAT_VERSION,
            game: game_field,
            version_id: &pack.version,
            name: &pack.name,
            summary: pack.summary.as_ref().map(String::as_str),
            files,
            dependencies: ManifestDeps::try_from(&pack.game)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFile<'a> {
    #[serde(borrow)]
    pub path: &'a Path,
    pub hashes: Option<ManifestHashes<'a>>,
    #[serde(default)]
    pub env: ManifestEnvs,
    #[serde(borrow)]
    pub downloads: Vec<&'a str>,
}

impl TryFrom<ManifestFile<'_>> for pack::ModpackFile {
    type Error = ModpackError;

    fn try_from(file: ManifestFile<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            path: PathBuf::from(file.path),
            hashes: file.hashes.map(pack::ModpackFileHashes::from),
            env: pack::ModpackEnv::try_from(file.env)?,
            downloads: file.downloads.into_iter().map(ToOwned::to_owned).collect(),
        })
    }
}

impl<'a> From<&'a pack::ModpackFile> for ManifestFile<'a> {
    fn from(file: &'a pack::ModpackFile) -> Self {
        Self {
            path: file.path.as_path(),
            hashes: file.hashes.as_ref().map(ManifestHashes::from),
            env: file.env.into(),
            downloads: file
                .downloads
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct ManifestHashes<'a> {
    pub sha1: &'a str,
}

impl From<ManifestHashes<'_>> for pack::ModpackFileHashes {
    fn from(hashes: ManifestHashes<'_>) -> Self {
        Self {
            sha1: String::from(hashes.sha1),
        }
    }
}

impl<'a> From<&'a pack::ModpackFileHashes> for ManifestHashes<'a> {
    fn from(hashes: &'a pack::ModpackFileHashes) -> Self {
        Self { sha1: &hashes.sha1 }
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
// HACK: I've tried for hours to get this working zero-copy, but I'm beat. If someone else wants to
// go through the #<!! of implementing it, be my guest.
pub enum ManifestDeps<'a> {
    MinecraftFabric {
        minecraft: &'a str,
        #[serde(rename = "fabric-loader")]
        fabric_loader: String,
    },
    MinecraftForge {
        minecraft: &'a str,
        forge: String,
    },
    MinecraftVanilla {
        minecraft: &'a str,
    },
}

impl From<ManifestDeps<'_>> for pack::ModpackGame {
    fn from(deps: ManifestDeps<'_>) -> Self {
        use ManifestDeps::*;

        match deps {
            MinecraftVanilla { minecraft } => {
                Self::Minecraft(String::from(minecraft), ModLoader::Vanilla)
            }
            MinecraftFabric { minecraft, .. } => {
                Self::Minecraft(String::from(minecraft), ModLoader::Fabric)
            }
            MinecraftForge { minecraft, .. } => {
                Self::Minecraft(String::from(minecraft), ModLoader::Forge)
            }
        }
    }
}

impl<'a> TryFrom<&'a pack::ModpackGame> for ManifestDeps<'a> {
    type Error = ModpackError;

    fn try_from(game: &'a pack::ModpackGame) -> Result<Self, Self::Error> {
        use super::pack::ModpackGame::*;
        Ok(match game {
            Minecraft(ref ver, ModLoader::Vanilla) => Self::MinecraftVanilla { minecraft: ver },
            Minecraft(ref ver, loader @ ModLoader::Fabric) => Self::MinecraftFabric {
                minecraft: ver,
                fabric_loader: get_loader_version(*loader, ver)?,
            },
            Minecraft(ref ver, loader @ ModLoader::Forge) => Self::MinecraftForge {
                minecraft: ver,
                forge: get_loader_version(*loader, ver)?,
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
            game: "minecraft",
            version_id: "deadbeef",
            name: "Example Pack",
            summary: None,
            files: Vec::new(),
            dependencies: ManifestDeps::MinecraftVanilla {
                minecraft: "1.17.1",
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
            game: "minecraft",
            version_id: "deadbeef",
            name: "Example Pack",
            summary: None,
            files: vec![ManifestFile {
                path: Path::new("mods/testmod.jar"),
                hashes: ManifestHashes {
                    sha1: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                },
                env: ManifestEnvs::default(),
                downloads: vec!["https://example.com/testmod.jar"],
            }],
            dependencies: ManifestDeps::MinecraftForge {
                minecraft: "1.17.1",
                forge: String::from("37.0.110"),
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
            game: "minecraft",
            version_id: "deadbeef",
            name: "Example Pack",
            summary: None,
            files: vec![ManifestFile {
                path: Path::new("mods/testmod.jar"),
                hashes: ManifestHashes {
                    sha1: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                },
                env: ManifestEnvs::default(),
                downloads: vec!["https://example.com/testmod.jar"],
            }],
            dependencies: ManifestDeps::MinecraftFabric {
                minecraft: "1.17.1",
                fabric_loader: String::from("0.9.0"),
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
            game: "minecraft",
            version_id: "deadbeef",
            name: "Example Pack",
            summary: Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."),
            files: vec![ManifestFile {
                path: Path::new("mods/testmod.jar"),
                hashes: ManifestHashes {
                    sha1: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                },
                env: ManifestEnvs {
                    client: ManifestEnv::Required,
                    server: ManifestEnv::Unsupported,
                },
                downloads: vec!["https://example.com/testmod.jar"],
            }],
            dependencies: ManifestDeps::MinecraftForge {
                minecraft: "1.17.1",
                forge: String::from("37.0.110"),
            },
        };
        let manifest: Manifest = serde_json::from_str(PACK_JSON).expect("Error parsing pack JSON");

        assert_eq!(expected_manifest, manifest);
        Ok(())
    }
}
