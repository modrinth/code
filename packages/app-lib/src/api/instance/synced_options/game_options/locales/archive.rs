use super::super::options_file::{input_error, sha1_bytes};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use zip::ZipArchive;

pub(super) const MAX_LANGUAGE_BYTES: usize = 4 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 100_000;
const MAX_NESTED_BYTES: usize = 64 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 128 * 1024 * 1024;
const MAX_NESTED_DEPTH: usize = 4;
pub(super) type Translations = BTreeMap<String, String>;

#[derive(Default, Serialize, Deserialize)]
pub(super) struct ArchiveIndex {
	#[serde(default)]
	pub version: u32,
	pub bundles: Vec<LanguageBundle>,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct LanguageBundle {
	pub nested_path: String,
	pub locales: BTreeMap<String, Translations>,
	#[serde(default)]
	pub deprecated: DeprecatedTranslations,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct DeprecatedTranslations {
	#[serde(default)]
	removed: Vec<String>,
	#[serde(default)]
	renamed: BTreeMap<String, String>,
}

impl DeprecatedTranslations {
	pub fn apply(&self, translations: &mut Translations) {
		for key in &self.removed {
			translations.remove(key);
		}
		for (from, to) in &self.renamed {
			if let Some(value) = translations.remove(from) {
				translations.insert(to.clone(), value);
			} else {
				translations.remove(to);
			}
		}
	}
}

pub(super) fn parse_language(bytes: &[u8], legacy: bool) -> crate::Result<Translations> {
	if bytes.len() > MAX_LANGUAGE_BYTES {
		return Err(input_error("Minecraft language file exceeds size limit"));
	}
	let values: Translations = if legacy {
		String::from_utf8_lossy(bytes)
			.lines()
			.filter(|line| !line.starts_with('#'))
			.filter_map(|line| line.split_once('='))
			.map(|(key, value)| (key.to_owned(), value.to_owned()))
			.collect()
	} else {
		serde_json::from_slice::<BTreeMap<String, serde_json::Value>>(bytes)?
			.into_iter()
			.filter_map(|(key, value)| value.as_str().map(|s| (key, s.to_owned())))
			.collect()
	};
	Ok(values.into_iter().filter(|(key, value)| {
		!key.is_empty() && key.len() <= 1024 && value.len() <= 32 * 1024
	}).collect())
}

pub(super) fn inspect(path: &Path, expected_hash: &str) -> crate::Result<ArchiveIndex> {
	let mut file = std::fs::File::open(path).map_err(crate::util::io::IOError::from)?;
	let mut hasher = sha1_smol::Sha1::new();
	let mut buffer = [0; 64 * 1024];
	loop {
		let count = file.read(&mut buffer).map_err(crate::util::io::IOError::from)?;
		if count == 0 { break; }
		hasher.update(&buffer[..count]);
	}
	if hasher.digest().to_string() != expected_hash {
		return Err(input_error("Locale source changed before it could be indexed"));
	}
	file.rewind().map_err(crate::util::io::IOError::from)?;
	let mut index = ArchiveIndex { version: 2, ..Default::default() };
	let mut budget = MAX_TOTAL_BYTES;
	let mut entries = MAX_ARCHIVE_ENTRIES;
	inspect_archive(file, "", 0, &mut budget, &mut entries, &mut index)?;
	Ok(index)
}

fn read_entry<R: Read + Seek>(
	archive: &mut ZipArchive<R>, name: &str, limit: usize, budget: &mut usize,
) -> crate::Result<Vec<u8>> {
	let mut entry = archive.by_name(name).map_err(|e| input_error(e.to_string()))?;
	let limit = limit.min(*budget);
	if entry.size() > limit as u64 {
		return Err(input_error("Locale archive entry exceeds size limit"));
	}
	let mut bytes = Vec::new();
	entry.by_ref().take(limit as u64 + 1).read_to_end(&mut bytes)
		.map_err(crate::util::io::IOError::from)?;
	if bytes.len() > limit {
		return Err(input_error("Locale archive entry exceeds size limit"));
	}
	*budget -= bytes.len();
	Ok(bytes)
}

fn inspect_archive<R: Read + Seek>(
	reader: R, nested_path: &str, depth: usize, budget: &mut usize,
	entries: &mut usize, index: &mut ArchiveIndex,
) -> crate::Result<()> {
	let mut archive = ZipArchive::new(reader).map_err(|e| input_error(e.to_string()))?;
	if archive.len() > *entries {
		return Err(input_error("Too many entries in locale archive"));
	}
	*entries -= archive.len();
	let names: BTreeSet<String> = archive.file_names().map(str::to_owned).collect();
	let mut bundle = LanguageBundle { nested_path: nested_path.to_owned(), ..Default::default() };
	for name in &names {
		let parts: Vec<_> = name.split('/').collect();
		let (namespace, filename) = match parts.as_slice() {
			["assets", namespace, "lang", filename] => (*namespace, *filename),
			["lang", filename] => ("minecraft", *filename),
			_ => continue,
		};
		if filename == "deprecated.json" {
			if namespace == "minecraft" {
				let bytes = read_entry(&mut archive, name, MAX_LANGUAGE_BYTES, budget)?;
				bundle.deprecated = serde_json::from_slice(&bytes)?;
			}
			continue;
		}
		let legacy = filename.ends_with(".lang");
		let Some(locale) = filename.strip_suffix(if legacy { ".lang" } else { ".json" }) else { continue; };
		if !valid_locale(locale) { continue; }
		let bytes = read_entry(&mut archive, name, MAX_LANGUAGE_BYTES, budget)?;
		match parse_language(&bytes, legacy) {
			Ok(values) => bundle.locales.entry(locale.to_ascii_lowercase()).or_default().extend(values),
			Err(error) => tracing::debug!(%name, %error, "Skipping malformed mod language file"),
		}
	}
	index.bundles.push(bundle);
	let mut nested = BTreeSet::new();
	for manifest in ["fabric.mod.json", "quilt.mod.json", "META-INF/jarjar/metadata.json"] {
		if !names.contains(manifest) { continue; }
		let bytes = read_entry(&mut archive, manifest, MAX_LANGUAGE_BYTES, budget)?;
		let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else { continue; };
		let jars = value.get("jars").or_else(|| value.pointer("/quilt_loader/jars"));
		for jar in jars.and_then(|v| v.as_array()).into_iter().flatten() {
			if let Some(path) = jar.as_str().or_else(|| jar.get("file").and_then(|v| v.as_str()))
				.or_else(|| jar.get("path").and_then(|v| v.as_str())) {
				nested.insert(path.to_owned());
			}
		}
	}
	if !nested.is_empty() && depth >= MAX_NESTED_DEPTH {
		return Err(input_error("Locale archive nesting exceeds limit"));
	}
	for name in nested {
		if !names.contains(&name) || !name.ends_with(".jar") { continue; }
		let bytes = read_entry(&mut archive, &name, MAX_NESTED_BYTES, budget)?;
		let path = if nested_path.is_empty() { name } else { format!("{nested_path}!{name}") };
		inspect_archive(Cursor::new(bytes), &path, depth + 1, budget, entries, index)?;
	}
	Ok(())
}

pub(super) fn valid_locale(locale: &str) -> bool {
	!locale.is_empty() && locale.len() <= 32
		&& locale.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

pub(super) fn valid_hash(hash: &str) -> bool {
	hash.len() == 40 && hash.bytes().all(|c| c.is_ascii_hexdigit())
}

pub(super) fn checked_bytes(bytes: Vec<u8>, hash: &str) -> crate::Result<Vec<u8>> {
	if sha1_bytes(&bytes) != hash {
		return Err(input_error("Minecraft locale resource checksum mismatch"));
	}
	Ok(bytes)
}
