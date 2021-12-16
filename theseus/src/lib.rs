//! # Theseus
//!
//! Theseus is a library which provides utilities for launching minecraft, creating Modrinth mod packs,
//! and launching Modrinth mod packs

#![warn(missing_docs, unused_import_braces, missing_debug_implementations)]

use std::path::Path;
use std::time::Duration;

lazy_static::lazy_static! {
    pub static ref LAUNCHER_WORK_DIR: &'static Path = Path::new("./launcher");
}

pub mod launcher;
mod meta;
pub mod modpack;
mod util;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Launcher error: {0}")]
    LauncherError(#[from] launcher::LauncherError),

    #[error("Modpack error: {0}")]
    ModpackError(#[from] modpack::ModpackError),

    #[error("Meta error: {0}")]
    DaedalusError(#[from] meta::MetaError),
}

pub async fn init() -> Result<(), Error> {
    std::fs::create_dir_all(*LAUNCHER_WORK_DIR).expect("Unable to create launcher root directory!");
    crate::meta::Metadata::init().await?;

    Ok(())
}
