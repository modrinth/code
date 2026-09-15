use serde::{Deserialize, Serialize};
use serde_json::Value;

const MOJANG_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const FABRIC_GAME: &str = "https://meta.fabricmc.net/v2/versions/game";
const FABRIC_LOADER: &str = "https://meta.fabricmc.net/v2/versions/loader";
const QUILT_LOADER: &str = "https://meta.quiltmc.org/v3/versions/loader";
const FORGE_META: &str =
    "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
const NEOFORGE_META: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftVersion {
    pub id: String,
    pub version_type: String,
    pub release_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderKind {
    pub id: String,
    pub name: String,
}

fn http() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(concat!("Owyx/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))
}

pub fn list_loaders() -> Vec<LoaderKind> {
    vec![
        LoaderKind {
            id: "vanilla".into(),
            name: "Vanilla".into(),
        },
        LoaderKind {
            id: "fabric".into(),
            name: "Fabric".into(),
        },
        LoaderKind {
            id: "forge".into(),
            name: "Forge".into(),
        },
        LoaderKind {
            id: "neoforge".into(),
            name: "NeoForge".into(),
        },
        LoaderKind {
            id: "quilt".into(),
            name: "Quilt".into(),
        },
    ]
}

pub async fn list_minecraft_versions(
    loader: &str,
    include_snapshots: bool,
) -> Result<Vec<MinecraftVersion>, String> {
    let loader = loader.to_ascii_lowercase();
    match loader.as_str() {
        "vanilla" | "forge" | "neoforge" => list_mojang_versions(include_snapshots).await,
        "fabric" => list_fabric_game_versions(include_snapshots).await,
        "quilt" => {
            // Quilt tracks Mojang games; filter by quilt loaders availability lazily in UI.
            list_mojang_versions(include_snapshots).await
        }
        other => Err(format!("Unknown loader: {other}")),
    }
}

pub async fn list_loader_versions(
    loader: &str,
    minecraft: &str,
) -> Result<Vec<String>, String> {
    let loader = loader.to_ascii_lowercase();
    let minecraft = minecraft.trim();
    if minecraft.is_empty() {
        return Err("Minecraft version is required".into());
    }
    match loader.as_str() {
        "vanilla" => Ok(vec![]),
        "fabric" => list_fabric_loaders(minecraft).await,
        "quilt" => list_quilt_loaders(minecraft).await,
        "forge" => list_forge_like_versions(FORGE_META, minecraft, "forge").await,
        "neoforge" => list_neoforge_versions(minecraft).await,
        other => Err(format!("Unknown loader: {other}")),
    }
}

async fn list_mojang_versions(include_snapshots: bool) -> Result<Vec<MinecraftVersion>, String> {
    let client = http()?;
    let manifest: Value = client
        .get(MOJANG_MANIFEST)
        .send()
        .await
        .map_err(|e| format!("Mojang manifest fetch failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Mojang manifest HTTP error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Mojang manifest JSON error: {e}"))?;

    let versions = manifest
        .get("versions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Mojang manifest missing versions[]".to_string())?;

    let mut out = Vec::new();
    for v in versions {
        let id = v.get("id").and_then(|x| x.as_str()).unwrap_or_default();
        let ty = v.get("type").and_then(|x| x.as_str()).unwrap_or_default();
        if id.is_empty() {
            continue;
        }
        if !include_snapshots && ty != "release" {
            continue;
        }
        out.push(MinecraftVersion {
            id: id.to_string(),
            version_type: ty.to_string(),
            release_time: v
                .get("releaseTime")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    Ok(out)
}

async fn list_fabric_game_versions(include_snapshots: bool) -> Result<Vec<MinecraftVersion>, String> {
    let client = http()?;
    let games: Value = client
        .get(FABRIC_GAME)
        .send()
        .await
        .map_err(|e| format!("Fabric game list failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Fabric game list HTTP error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Fabric game list JSON error: {e}"))?;

    let arr = games
        .as_array()
        .ok_or_else(|| "Fabric game list is not an array".to_string())?;
    let mut out = Vec::new();
    for v in arr {
        let id = v.get("version").and_then(|x| x.as_str()).unwrap_or_default();
        let stable = v.get("stable").and_then(|x| x.as_bool()).unwrap_or(false);
        if id.is_empty() {
            continue;
        }
        if !include_snapshots && !stable {
            continue;
        }
        out.push(MinecraftVersion {
            id: id.to_string(),
            version_type: if stable {
                "release".into()
            } else {
                "snapshot".into()
            },
            release_time: String::new(),
        });
    }
    Ok(out)
}

async fn list_fabric_loaders(minecraft: &str) -> Result<Vec<String>, String> {
    let client = http()?;
    let url = format!("{FABRIC_LOADER}/{minecraft}");
    let rows: Value = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Fabric loader list failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Fabric loader HTTP error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Fabric loader JSON error: {e}"))?;

    let arr = rows
        .as_array()
        .ok_or_else(|| "Fabric loader list is not an array".to_string())?;
    Ok(arr
        .iter()
        .filter_map(|row| {
            row.get("loader")
                .and_then(|l| l.get("version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect())
}

async fn list_quilt_loaders(minecraft: &str) -> Result<Vec<String>, String> {
    let client = http()?;
    let url = format!("{QUILT_LOADER}/{minecraft}");
    let rows: Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Quilt loader list failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Quilt loader HTTP error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Quilt loader JSON error: {e}"))?;

    let arr = rows
        .as_array()
        .ok_or_else(|| "Quilt loader list is not an array".to_string())?;
    Ok(arr
        .iter()
        .filter_map(|row| {
            row.get("loader")
                .and_then(|l| l.get("version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect())
}

async fn list_forge_like_versions(
    meta_url: &str,
    minecraft: &str,
    _kind: &str,
) -> Result<Vec<String>, String> {
    let client = http()?;
    let xml = client
        .get(meta_url)
        .send()
        .await
        .map_err(|e| format!("Forge metadata fetch failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Forge metadata HTTP error: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Forge metadata body error: {e}"))?;

    // Versions look like: 1.20.1-47.2.0
    let prefix = format!("{minecraft}-");
    let mut out = Vec::new();
    for line in xml.lines() {
        let line = line.trim();
        let Some(inner) = line
            .strip_prefix("<version>")
            .and_then(|s| s.strip_suffix("</version>"))
        else {
            continue;
        };
        if let Some(rest) = inner.strip_prefix(&prefix) {
            if !rest.is_empty() {
                out.push(rest.to_string());
            }
        }
    }
    out.reverse(); // maven metadata is usually oldest→newest
    Ok(out)
}

async fn list_neoforge_versions(minecraft: &str) -> Result<Vec<String>, String> {
    // NeoForge versions often encode MC as maj.min in the version string (e.g. 21.1.x for 1.21.1).
    let client = http()?;
    let xml = client
        .get(NEOFORGE_META)
        .send()
        .await
        .map_err(|e| format!("NeoForge metadata fetch failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("NeoForge metadata HTTP error: {e}"))?
        .text()
        .await
        .map_err(|e| format!("NeoForge metadata body error: {e}"))?;

    let mc_parts: Vec<&str> = minecraft.split('.').collect();
    let filter = if mc_parts.len() >= 2 {
        // 1.21.1 -> 21.1
        format!("{}.{}", mc_parts[1], mc_parts.get(2).copied().unwrap_or("0"))
    } else {
        minecraft.to_string()
    };

    let mut out = Vec::new();
    for line in xml.lines() {
        let line = line.trim();
        let Some(ver) = line
            .strip_prefix("<version>")
            .and_then(|s| s.strip_suffix("</version>"))
        else {
            continue;
        };
        // Match "21.1" as a prefix only at a version segment boundary:
        // 21.1.77 ok, 21.10.x must not match filter "21.1".
        let matched = ver == filter
            || ver.starts_with(&format!("{filter}."))
            || ver.starts_with(&format!("{filter}-"))
            || ver.contains(&format!("-{minecraft}"));
        if matched {
            out.push(ver.to_string());
        }
    }
    out.reverse();
    Ok(out)
}
