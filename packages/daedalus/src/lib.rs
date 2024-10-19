//! # Daedalus
//!
//! Daedalus is a library which provides models and methods to fetch metadata about games

#![warn(missing_docs, unused_import_braces, missing_debug_implementations)]

/// Models and methods for fetching metadata for Minecraft
pub mod minecraft;
/// Models and methods for fetching metadata for Minecraft mod loaders
pub mod modded;

#[derive(thiserror::Error, Debug)]
/// An error type representing possible errors when fetching metadata
pub enum Error {
    /// Error while parsing input
    #[error("{0}")]
    ParseError(String),
}

/// Converts a maven artifact to a path
pub fn get_path_from_artifact(artifact: &str) -> Result<String, Error> {
    let name_items = artifact.split(':').collect::<Vec<&str>>();

    let package = name_items.first().ok_or_else(|| {
        Error::ParseError(format!(
            "Unable to find package for library {}",
            &artifact
        ))
    })?;
    let name = name_items.get(1).ok_or_else(|| {
        Error::ParseError(format!(
            "Unable to find name for library {}",
            &artifact
        ))
    })?;

    if name_items.len() == 3 {
        let version_ext = name_items
            .get(2)
            .ok_or_else(|| {
                Error::ParseError(format!(
                    "Unable to find version for library {}",
                    &artifact
                ))
            })?
            .split('@')
            .collect::<Vec<&str>>();
        let version = version_ext.first().ok_or_else(|| {
            Error::ParseError(format!(
                "Unable to find version for library {}",
                &artifact
            ))
        })?;
        let ext = version_ext.get(1);

        Ok(format!(
            "{}/{}/{}/{}-{}.{}",
            package.replace('.', "/"),
            name,
            version,
            name,
            version,
            ext.unwrap_or(&"jar")
        ))
    } else {
        let version = name_items.get(2).ok_or_else(|| {
            Error::ParseError(format!(
                "Unable to find version for library {}",
                &artifact
            ))
        })?;

        let data_ext = name_items
            .get(3)
            .ok_or_else(|| {
                Error::ParseError(format!(
                    "Unable to find data for library {}",
                    &artifact
                ))
            })?
            .split('@')
            .collect::<Vec<&str>>();
        let data = data_ext.first().ok_or_else(|| {
            Error::ParseError(format!(
                "Unable to find data for library {}",
                &artifact
            ))
        })?;
        let ext = data_ext.get(1);

        Ok(format!(
            "{}/{}/{}/{}-{}-{}.{}",
            package.replace('.', "/"),
            name,
            version,
            name,
            version,
            data,
            ext.unwrap_or(&"jar")
        ))
    }
}
