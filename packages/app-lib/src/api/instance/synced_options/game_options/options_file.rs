//! Reads and edits `options.txt` without changing unrelated lines or file formatting.

use super::super as synced_options;
use super::catalog::{LEGACY_DATA_VERSIONS, release_version};
use super::{
    MAX_KEY_BYTES, MAX_OPTIONS_BYTES, MAX_OPTIONS_LINES, MAX_VALUE_BYTES,
    OPTIONS_FILE,
};
use crate::ErrorKind;
use crate::state::{InstanceMetadata, State};
use crate::util::io;
use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE};
use sha1_smol::Sha1;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(super) struct GameOptionsDocument {
    pub(super) encoding: &'static Encoding,
    pub(super) bom: Bom,
    pub(super) default_line_ending: String,
    pub(super) lines: Vec<GameOptionsLine>,
}

#[derive(Clone, Copy)]
pub(super) enum Bom {
    None,
    Utf8,
    Utf16Le,
    Utf16Be,
}

#[derive(Clone)]
pub(super) struct GameOptionsLine {
    pub(super) raw: String,
    pub(super) ending: String,
    pub(super) entry: Option<GameOptionsEntry>,
}

#[derive(Clone)]
pub(super) struct GameOptionsEntry {
    pub(super) key: String,
    pub(super) value: String,
    pub(super) changed: bool,
}

impl GameOptionsDocument {
    pub(super) fn empty() -> Self {
        Self {
            encoding: UTF_8,
            bom: Bom::None,
            default_line_ending: "\n".to_string(),
            lines: Vec::new(),
        }
    }

    /// Creates settings in the target game's format. Modern key bindings need
    /// its data version so Minecraft does not run legacy migrations on them.
    /// Unknown versions without embedded metadata wait for Minecraft's own file.
    pub(super) async fn for_instance(
        metadata: &InstanceMetadata,
        state: &State,
    ) -> crate::Result<Option<Self>> {
        let content_set = &metadata.applied_content_set;
        let version = &content_set.game_version;
        let version_jar = content_set.loader_version.as_ref().map_or_else(
            || version.clone(),
            |loader| format!("{version}-{loader}"),
        );
        let path = state
            .directories
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));
        let embedded_version = if path.exists() {
            crate::launcher::read_game_version_metadata_from_jar(&path)
                .await?
                .and_then(|metadata| metadata.world_version)
        } else {
            None
        };
        let data_version = embedded_version
            .or_else(|| LEGACY_DATA_VERSIONS.get(version.as_str()).copied());
        if data_version.is_none()
            && !release_version(version)
                .is_some_and(|(major, minor, _)| major == 1 && minor < 10)
        {
            return Ok(None);
        }
        let mut document = Self::empty();
        if let Some(data_version) = data_version {
            document.set("version", &data_version.to_string(), true)?;
        }
        Ok(Some(document))
    }

    pub(super) fn parse(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() > MAX_OPTIONS_BYTES {
            return Err(input_error("options.txt is too large to sync safely"));
        }

        let (bom, encoding, contents) =
            if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                (Bom::Utf8, UTF_8, &bytes[3..])
            } else if bytes.starts_with(&[0xFF, 0xFE]) {
                (Bom::Utf16Le, UTF_16LE, &bytes[2..])
            } else if bytes.starts_with(&[0xFE, 0xFF]) {
                (Bom::Utf16Be, UTF_16BE, &bytes[2..])
            } else {
                let mut detector = chardetng::EncodingDetector::new();
                detector.feed(bytes, true);
                (Bom::None, detector.guess(None, true), bytes)
            };
        let (decoded, decode_errors) =
            encoding.decode_without_bom_handling(contents);
        if decode_errors {
            return Err(input_error(
                "options.txt contains bytes that cannot be decoded losslessly",
            ));
        }
        let roundtrip = encode_without_bom(encoding, &decoded)?;
        if roundtrip.as_slice() != contents {
            return Err(input_error(
                "options.txt encoding cannot be round-tripped safely",
            ));
        }

        let mut lines = Vec::new();
        let mut start = 0;
        let source: &str = decoded.as_ref();
        for (index, byte) in source.as_bytes().iter().enumerate() {
            if *byte != b'\n' {
                continue;
            }
            let raw_end =
                if index > start && source.as_bytes()[index - 1] == b'\r' {
                    index - 1
                } else {
                    index
                };
            let ending = if raw_end == index { "\n" } else { "\r\n" };
            lines.push(Self::parse_line(&source[start..raw_end], ending));
            start = index + 1;
        }
        if start < source.len() {
            lines.push(Self::parse_line(&source[start..], ""));
        }
        if lines.len() > MAX_OPTIONS_LINES {
            return Err(input_error(
                "options.txt has too many lines to sync safely",
            ));
        }
        let default_line_ending = lines
            .iter()
            .find(|line| !line.ending.is_empty())
            .map(|line| line.ending.clone())
            .unwrap_or_else(|| "\n".to_string());
        Ok(Self {
            encoding,
            bom,
            default_line_ending,
            lines,
        })
    }

    pub(super) fn parse_line(raw: &str, ending: &str) -> GameOptionsLine {
        let entry = raw.split_once(':').and_then(|(key, value)| {
            if key.is_empty()
                || key.len() > MAX_KEY_BYTES
                || value.len() > MAX_VALUE_BYTES
            {
                None
            } else {
                Some(GameOptionsEntry {
                    key: key.to_string(),
                    value: value.to_string(),
                    changed: false,
                })
            }
        });
        GameOptionsLine {
            raw: raw.to_string(),
            ending: ending.to_string(),
            entry,
        }
    }

    pub(super) fn effective_entries(&self) -> HashMap<&str, (usize, usize)> {
        let mut entries = HashMap::new();
        for (index, line) in self.lines.iter().enumerate() {
            if let Some(entry) = &line.entry {
                entries
                    .entry(entry.key.as_str())
                    .and_modify(|value: &mut (usize, usize)| {
                        value.0 = index;
                        value.1 += 1;
                    })
                    .or_insert((index, 1));
            }
        }
        entries
    }

    pub(super) fn value(&self, key: &str) -> Option<&str> {
        let (index, _) = self.effective_entries().get(key).copied()?;
        self.lines[index]
            .entry
            .as_ref()
            .map(|entry| entry.value.as_str())
    }

    pub(super) fn set(
        &mut self,
        key: &str,
        value: &str,
        insert: bool,
    ) -> crate::Result<bool> {
        validate_raw_key_value(key, value)?;
        if let Some((index, _)) = self.effective_entries().get(key).copied() {
            let entry = self.lines[index].entry.as_mut().expect("entry index");
            if entry.value == value {
                return Ok(false);
            }
            entry.value = value.to_string();
            entry.changed = true;
            return Ok(true);
        }
        if !insert {
            return Ok(false);
        }

        let ending = if let Some(last) = self.lines.last_mut() {
            if last.ending.is_empty() {
                last.ending = self.default_line_ending.clone();
                String::new()
            } else {
                self.default_line_ending.clone()
            }
        } else {
            self.default_line_ending.clone()
        };
        self.lines.push(GameOptionsLine {
            raw: String::new(),
            ending,
            entry: Some(GameOptionsEntry {
                key: key.to_string(),
                value: value.to_string(),
                changed: true,
            }),
        });
        Ok(true)
    }

    pub(super) fn serialize(&self) -> crate::Result<Vec<u8>> {
        let mut text = String::new();
        for line in &self.lines {
            match &line.entry {
                Some(entry) if entry.changed => {
                    text.push_str(&entry.key);
                    text.push(':');
                    text.push_str(&entry.value);
                }
                _ => text.push_str(&line.raw),
            }
            text.push_str(&line.ending);
        }
        let encoded = encode_without_bom(self.encoding, &text)?;
        let bom: &[u8] = match self.bom {
            Bom::None => &[],
            Bom::Utf8 => &[0xEF, 0xBB, 0xBF],
            Bom::Utf16Le => &[0xFF, 0xFE],
            Bom::Utf16Be => &[0xFE, 0xFF],
        };
        let mut result = Vec::with_capacity(bom.len() + encoded.len());
        result.extend_from_slice(bom);
        result.extend_from_slice(&encoded);
        Ok(result)
    }
}

pub(super) fn encode_without_bom(
    encoding: &'static Encoding,
    text: &str,
) -> crate::Result<Vec<u8>> {
    if encoding == UTF_16LE || encoding == UTF_16BE {
        let mut encoded = Vec::with_capacity(text.len().saturating_mul(2));
        for unit in text.encode_utf16() {
            let bytes = if encoding == UTF_16LE {
                unit.to_le_bytes()
            } else {
                unit.to_be_bytes()
            };
            encoded.extend_from_slice(&bytes);
        }
        return Ok(encoded);
    }

    let (encoded, _, had_errors) = encoding.encode(text);
    if had_errors {
        return Err(input_error(
            "A synced option value cannot be represented in this file's encoding",
        ));
    }
    Ok(encoded.into_owned())
}

pub(super) fn input_error(message: impl Into<String>) -> crate::Error {
    ErrorKind::InputError(message.into()).into()
}

pub(super) fn validate_raw_key_value(
    key: &str,
    value: &str,
) -> crate::Result<()> {
    if key.is_empty()
        || key.len() > MAX_KEY_BYTES
        || key.contains(':')
        || key.chars().any(char::is_control)
    {
        return Err(input_error("Invalid options.txt setting key"));
    }
    if value.len() > MAX_VALUE_BYTES
        || value
            .chars()
            .any(|character| character.is_control() && character != '\t')
    {
        return Err(input_error("Invalid options.txt setting value"));
    }
    Ok(())
}

pub(super) fn options_path(
    metadata: &InstanceMetadata,
    state: &State,
) -> PathBuf {
    synced_options::instance_dir(metadata, state).join(OPTIONS_FILE)
}

pub(super) fn sha1_bytes(bytes: &[u8]) -> String {
    Sha1::from(bytes).digest().to_string()
}

/// Reads an `options.txt` only when it is safe to rewrite.
///
/// The parser keeps its text encoding, line endings, comments, and duplicate keys.
pub(super) async fn read_document(
    path: &Path,
) -> crate::Result<(GameOptionsDocument, Vec<u8>)> {
    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|error| io::IOError::with_path(error, path))?;
    if metadata.file_type().is_symlink() {
        return Err(input_error("Refusing to sync a symlinked options.txt"));
    }
    let bytes = io::read(path).await?;
    let document = GameOptionsDocument::parse(&bytes)?;
    Ok((document, bytes))
}
