use indexmap::IndexMap;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::minecraft::{Argument, ArgumentType, Library, VersionType};

/// Launch metadata combined with optional Forge-style installation metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMetadata {
	pub id: String,
	pub inherits_from: String,
	#[serde(deserialize_with = "crate::util::deserialize_timestamp")]
	pub release_time: Timestamp,
	#[serde(deserialize_with = "crate::util::deserialize_timestamp")]
	pub time: Timestamp,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub main_class: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub minecraft_arguments: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub arguments: Option<IndexMap<ArgumentType, Vec<Argument>>>,
	pub libraries: Vec<Library>,
	#[serde(rename = "type")]
	pub ty: VersionType,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub data: Option<IndexMap<String, SidedDataEntry>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub processors: Option<Vec<Processor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidedDataEntry {
	pub client: String,
	pub server: String,
}

/// Installation step executed by the launcher after downloading libraries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Processor {
	pub jar: String,
	pub classpath: Vec<String>,
	pub args: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub outputs: Option<IndexMap<String, String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub sides: Option<Vec<String>>,
}
