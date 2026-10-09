use derive_more::Display;
use serde::{Deserialize, Serialize};
use toasty::{Embed, Model};

use crate::{upstream::mojang, util::Sha256};

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Embed,
)]
pub enum MinecraftLoader {
	Fabric,
	Forge,
	Neoforge,
	Quilt,
}

#[derive(
	Debug,
	Display,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Serialize,
	Deserialize,
	Embed,
)]
pub struct MinecraftVersionName(pub String);

/// Selected Minecraft metadata, with library patches applied for publication.
#[derive(Debug, Clone, Model)]
pub struct MinecraftVersion {
	#[key]
	pub name: MinecraftVersionName,
	#[column(type = json)]
	pub catalog_entry: toasty::Json<mojang::Version>,
	#[column(type = json)]
	pub manifest: toasty::Json<mojang::VersionManifest>,
	pub source_catalog_sha256: Sha256,
	pub source_manifest_sha256: Sha256,
	#[unique]
	pub is_latest_release: Option<bool>,
	#[unique]
	pub is_latest_snapshot: Option<bool>,
}
