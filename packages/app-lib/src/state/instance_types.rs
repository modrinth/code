use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceInstallStage {
	Installed,
	MinecraftInstalling,
	PackInstalled,
	PackInstalling,
	NotInstalled,
}

impl InstanceInstallStage {
	pub fn as_str(&self) -> &'static str {
		match *self {
			Self::Installed => "installed",
			Self::MinecraftInstalling => "minecraft_installing",
			Self::PackInstalled => "pack_installed",
			Self::PackInstalling => "pack_installing",
			Self::NotInstalled => "not_installed",
		}
	}

	pub fn from_str(val: &str) -> Self {
		match val {
			"installed" => Self::Installed,
			"minecraft_installing" => Self::MinecraftInstalling,
			"installing" => Self::MinecraftInstalling,
			"pack_installed" => Self::PackInstalled,
			"pack_installing" => Self::PackInstalling,
			"not_installed" => Self::NotInstalled,
			_ => Self::NotInstalled,
		}
	}
}

#[derive(
	Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd,
)]
#[serde(rename_all = "snake_case")]
pub enum LauncherFeatureVersion {
	None,
	MigratedServerLastPlayTime,
	MigratedLaunchHooks,
}

impl LauncherFeatureVersion {
	pub const MOST_RECENT: Self = Self::MigratedLaunchHooks;

	pub fn as_str(&self) -> &'static str {
		match *self {
			Self::None => "none",
			Self::MigratedServerLastPlayTime => {
				"migrated_server_last_play_time"
			}
			Self::MigratedLaunchHooks => "migrated_launch_hooks",
		}
	}

	pub fn from_str(val: &str) -> Self {
		match val {
			"none" => Self::None,
			"migrated_server_last_play_time" => {
				Self::MigratedServerLastPlayTime
			}
			"migrated_launch_hooks" => Self::MigratedLaunchHooks,
			_ => Self::None,
		}
	}
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
	Vanilla,
	Forge,
	Fabric,
	Quilt,
	NeoForge,
}

impl ModLoader {
	pub fn as_str(&self) -> &'static str {
		match *self {
			Self::Vanilla => "vanilla",
			Self::Forge => "forge",
			Self::Fabric => "fabric",
			Self::Quilt => "quilt",
			Self::NeoForge => "neoforge",
		}
	}

	pub fn as_meta_str(&self) -> &'static str {
		match *self {
			Self::Vanilla => "vanilla",
			Self::Forge => "forge",
			Self::Fabric => "fabric",
			Self::Quilt => "quilt",
			Self::NeoForge => "neo",
		}
	}

	pub fn from_string(val: &str) -> Self {
		match val {
			"vanilla" => Self::Vanilla,
			"forge" => Self::Forge,
			"fabric" => Self::Fabric,
			"quilt" => Self::Quilt,
			"neoforge" => Self::NeoForge,
			_ => Self::Vanilla,
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentFile {
	pub hash: String,
	pub file_name: String,
	pub size: u64,
	pub metadata: Option<FileMetadata>,
	pub update_version_id: Option<String>,
	pub project_type: ProjectType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
	pub project_id: String,
	pub version_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
	Mod,
	DataPack,
	ResourcePack,
	ShaderPack,
}

impl ProjectType {
	pub fn get_from_loaders(loaders: Vec<String>) -> Option<Self> {
		if loaders
			.iter()
			.any(|x| ["fabric", "forge", "quilt", "neoforge"].contains(&&**x))
		{
			Some(ProjectType::Mod)
		} else if loaders.iter().any(|x| x == "datapack") {
			Some(ProjectType::DataPack)
		} else if loaders.iter().any(|x| ["iris", "optifine"].contains(&&**x)) {
			Some(ProjectType::ShaderPack)
		} else if loaders
			.iter()
			.any(|x| ["vanilla", "canvas", "minecraft"].contains(&&**x))
		{
			Some(ProjectType::ResourcePack)
		} else {
			None
		}
	}

	pub fn get_from_parent_folder(path: impl AsRef<Path>) -> Option<Self> {
		match path
			.as_ref()
			.parent()?
			.file_name()?
			.to_str()
			.unwrap_or_default()
		{
			"mods" => Some(ProjectType::Mod),
			"datapacks" => Some(ProjectType::DataPack),
			"resourcepacks" => Some(ProjectType::ResourcePack),
			"shaderpacks" => Some(ProjectType::ShaderPack),
			_ => None,
		}
	}

	pub fn get_name(&self) -> &'static str {
		match self {
			ProjectType::Mod => "mod",
			ProjectType::DataPack => "datapack",
			ProjectType::ResourcePack => "resourcepack",
			ProjectType::ShaderPack => "shader",
		}
	}

	pub fn get_folder(&self) -> &'static str {
		match self {
			ProjectType::Mod => "mods",
			ProjectType::DataPack => "datapacks",
			ProjectType::ResourcePack => "resourcepacks",
			ProjectType::ShaderPack => "shaderpacks",
		}
	}

	pub fn get_loaders(&self) -> &'static [&'static str] {
		match self {
			ProjectType::Mod => &["fabric", "forge", "quilt", "neoforge"],
			ProjectType::DataPack => &["datapack"],
			ProjectType::ResourcePack => &["vanilla", "canvas", "minecraft"],
			ProjectType::ShaderPack => &["iris", "optifine"],
		}
	}

	pub fn iterator() -> impl Iterator<Item = ProjectType> {
		[
			ProjectType::Mod,
			ProjectType::DataPack,
			ProjectType::ResourcePack,
			ProjectType::ShaderPack,
		]
		.iter()
		.copied()
	}
}
