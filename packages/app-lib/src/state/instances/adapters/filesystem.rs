use crate::state::ProjectType;
use crate::util::io::{self, IOError};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct ScannedContentFile {
    pub relative_path: String,
    pub file_name: String,
    pub enabled: bool,
    pub size: u64,
    pub hash_cache_key: String,
}

pub(crate) fn scan_content_files(
    instances_dir: &Path,
    instance_path: &str,
) -> crate::Result<Vec<ScannedContentFile>> {
    let instance_dir = io::canonicalize(instances_dir.join(instance_path))?;
    let mut files = Vec::new();

    for project_type in ProjectType::iterator() {
        let folder = project_type.get_folder();
        let folder_path = instance_dir.join(folder);

        if !folder_path.exists() {
            continue;
        }

        for entry in std::fs::read_dir(&folder_path)
            .map_err(|err| IOError::with_path(err, &folder_path))?
        {
            let path = entry.map_err(IOError::from)?.path();
            if !path.is_file() {
                continue;
            }

            let Some(file_name) =
                path.file_name().and_then(|value| value.to_str())
            else {
                continue;
            };

            if !is_scannable_project_file(project_type, file_name) {
                continue;
            }

            let size = path.metadata().map_err(IOError::from)?.len();
            let relative_path = format!("{folder}/{file_name}");

            files.push(ScannedContentFile {
                relative_path,
                file_name: file_name.to_string(),
                enabled: !file_name.ends_with(".disabled"),
                size,
                hash_cache_key: format!(
                    "{size}-{instance_path}/{folder}/{file_name}"
                ),
            });
        }
    }

    Ok(files)
}

pub(crate) fn project_type_from_relative_path(
    relative_path: &str,
) -> Option<ProjectType> {
    ProjectType::get_from_parent_folder(PathBuf::from(relative_path))
}

fn is_scannable_project_file(
    project_type: ProjectType,
    file_name: &str,
) -> bool {
    let Some(extension) = Path::new(file_name.trim_end_matches(".disabled"))
        .extension()
        .and_then(|ext| ext.to_str())
    else {
        return false;
    };

    match project_type {
        ProjectType::Mod => extension.eq_ignore_ascii_case("jar"),
        ProjectType::DataPack
        | ProjectType::ResourcePack
        | ProjectType::ShaderPack => extension.eq_ignore_ascii_case("zip"),
    }
}
