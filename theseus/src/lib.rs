//! # Theseus
//!
//! Theseus is a library which provides utilities for launching minecraft, creating Modrinth mod packs,
//! and launching Modrinth mod packs

#![warn(unused_import_braces, missing_debug_implementations)]

static LAUNCHER_WORK_DIR: &'static str = "./launcher";

pub mod data;
pub mod launcher;
pub mod modpack;
mod util;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Launcher error: {0}")]
    LauncherError(#[from] launcher::LauncherError),

    #[error("Modpack error: {0}")]
    ModpackError(#[from] modpack::ModpackError),

    #[error("Data error: {0}")]
    DaedalusError(#[from] data::DataError),
}

pub async fn init() -> Result<(), Error> {
    std::fs::create_dir_all(LAUNCHER_WORK_DIR)
        .expect("Unable to create launcher root directory!");

    use crate::data::*;
    Metadata::init().await?;

    Settings::init().await?;

    tokio::try_join! {
        launcher::init_download_semaphore(),
        Profiles::init(),
    }?;

    Ok(())
}

pub async fn save() -> Result<(), Error> {
    use crate::data::*;

    tokio::try_join! {
        Settings::save(),
        Profiles::save(),
    }?;

    Ok(())
}
