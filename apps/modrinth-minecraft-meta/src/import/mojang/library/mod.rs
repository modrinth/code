use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::{
    upstream::mojang::{
        Library, LibraryDownloads, LibraryExtract, OperatingSystem, Rule,
    },
    util::from_json_slice,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPatch {
    // #[serde(rename = "_comment")]
    // pub _comment: String,
    #[serde(rename = "match")]
    pub matches: Vec<String>,
    pub additional_libraries: Option<Vec<Library>>,
    #[serde(rename = "override")]
    pub override_library: Option<PartialLibrary>,
    pub patch_additional_libraries: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialLibrary {
    pub downloads: Option<LibraryDownloads>,
    pub extract: Option<LibraryExtract>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub natives: Option<HashMap<OperatingSystem, String>>,
    pub rules: Option<Vec<Rule>>,
    pub checksums: Option<Vec<String>>,
    pub include_in_classpath: Option<bool>,
}

/// Bundled copy of PrismLauncher's library patches.
///
/// Sourced from <https://github.com/PrismLauncher/meta/blob/main/meta/common/mojang-library-patches.json>
///
/// The upstream repository is licensed under the Microsoft Public License (Ms-PL).
static LIBRARY_PATCHES: LazyLock<Vec<LibraryPatch>> = LazyLock::new(|| {
    from_json_slice(include_bytes!("library-patches.json"))
        .expect("shouldn't fail to deserialize")
});

pub fn patch_library(mut library: Library) -> Vec<Library> {
    let patches = LIBRARY_PATCHES.as_slice();

    let mut libraries = Vec::new();
    let matching_patches = patches
        .iter()
        .filter(|patch| patch.matches.contains(&library.name))
        .collect::<Vec<_>>();
    for patch in matching_patches {
        if let Some(override_library) = &patch.override_library {
            library = merge_partial_library(override_library.clone(), library);
        }
        if let Some(additional_libraries) = &patch.additional_libraries {
            for additional_library in additional_libraries {
                if patch.patch_additional_libraries.unwrap_or(false) {
                    libraries.extend(patch_library(additional_library.clone()));
                } else {
                    libraries.push(additional_library.clone());
                }
            }
        }
    }
    libraries.push(library);
    libraries
}

pub fn merge_partial_library(
    partial: PartialLibrary,
    mut library: Library,
) -> Library {
    if let Some(downloads) = partial.downloads {
        if let Some(existing) = &mut library.downloads {
            if let Some(artifact) = downloads.artifact {
                existing.artifact = Some(artifact);
            }
            if let Some(classifiers) = downloads.classifiers {
                if let Some(existing_classifiers) = &mut existing.classifiers {
                    existing_classifiers.extend(classifiers);
                } else {
                    existing.classifiers = Some(classifiers);
                }
            }
        } else {
            library.downloads = Some(downloads);
        }
    }
    if let Some(extract) = partial.extract {
        library.extract = Some(extract);
    }
    if let Some(name) = partial.name {
        library.name = name;
    }
    if let Some(url) = partial.url {
        library.url = Some(url);
    }
    if let Some(natives) = partial.natives {
        if let Some(existing) = &mut library.natives {
            existing.extend(natives);
        } else {
            library.natives = Some(natives);
        }
    }
    if let Some(rules) = partial.rules {
        if let Some(existing) = &mut library.rules {
            existing.extend(rules);
        } else {
            library.rules = Some(rules);
        }
    }
    if let Some(checksums) = partial.checksums {
        library.checksums = Some(checksums);
    }
    if let Some(include_in_classpath) = partial.include_in_classpath {
        library.include_in_classpath = include_in_classpath;
    }
    library
}

#[cfg(test)]
mod tests {
    use crate::upstream::mojang::{Library, patch_library};

    #[test]
    fn bundled_patches_load_and_add_missing_dependencies() {
        let library: Library = serde_json::from_value(serde_json::json!({
            "name": "org.lwjgl:lwjgl:3.2.2"
        }))
        .unwrap();
        let patched = patch_library(library);
        assert!(
            patched
                .iter()
                .any(|library| library.name == "org.lwjgl:lwjgl-tinyfd:3.2.2")
        );
        assert_eq!(patched.last().unwrap().name, "org.lwjgl:lwjgl:3.2.2");
    }

    #[test]
    fn unmatched_library_is_unchanged() {
        let library: Library = serde_json::from_value(serde_json::json!({
            "name": "com.example:unmatched:1"
        }))
        .unwrap();
        let before = serde_json::to_value(&library).unwrap();
        let patched = patch_library(library);
        assert_eq!(patched.len(), 1);
        assert_eq!(serde_json::to_value(&patched[0]).unwrap(), before);
    }
}
