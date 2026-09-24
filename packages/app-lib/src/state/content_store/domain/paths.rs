use crate::state::content_store::input;
use itertools::Itertools;
use std::path::{Component, Path, PathBuf};

pub(crate) fn validate_digest(hash: &str, length: usize) -> crate::Result<()> {
    if hash.len() != length
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(input("Invalid content hash"));
    }
    Ok(())
}

pub(crate) fn object_relative_path(sha512: &str) -> crate::Result<String> {
    validate_digest(sha512, 128)?;
    Ok(format!("objects/{}/{}", &sha512[..2], sha512))
}

pub(crate) fn validate_relative(path: &str) -> crate::Result<()> {
    if path.is_empty()
        || path.contains('\\')
        || path.contains(':')
        || path.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.ends_with('.')
                || part.ends_with(' ')
        })
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(input(format!("Invalid instance-relative path: {path:?}")));
    }
    Ok(())
}

pub(crate) fn validate_instance_path(path: &str) -> crate::Result<()> {
    // Legacy instance names can end in dots or spaces; preserve their on-disk path.
    let normalized = path
        .split('/')
        .map(|part| part.trim_end_matches(['.', ' ']))
        .join("/");
    validate_relative(&normalized)
        .map_err(|_| input(format!("Invalid instance folder path: {path:?}")))
}

pub(crate) fn is_managed_content_path(path: &str) -> bool {
    if validate_relative(path).is_err() {
        return false;
    }
    let Some((directory, filename)) = path.split('/').collect_tuple() else {
        return false;
    };
    let extension = Path::new(filename.trim_end_matches(".disabled"))
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    match directory {
        "mods" => extension.eq_ignore_ascii_case("jar"),
        "resourcepacks" | "shaderpacks" | "datapacks" => {
            extension.eq_ignore_ascii_case("zip")
                || extension.eq_ignore_ascii_case("jar")
        }
        _ => false,
    }
}

pub(crate) fn relative_link(source: &Path, parent: &Path) -> PathBuf {
    if source.components().next() != parent.components().next() {
        return source.to_path_buf();
    }
    pathdiff::diff_paths(source, parent).unwrap_or_else(|| source.to_path_buf())
}

pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            part => normalized.push(part.as_os_str()),
        }
    }
    normalized
}

use crate::state::InstanceFile;

pub(crate) fn content_file_path(file: &InstanceFile) -> String {
    file_path_on_disk(&file.relative_path, file.enabled)
}

pub(crate) fn file_path_on_disk(relative_path: &str, enabled: bool) -> String {
    let canonical = relative_path.trim_end_matches(".disabled");
    if enabled {
        canonical.to_string()
    } else {
        format!("{canonical}.disabled")
    }
}
