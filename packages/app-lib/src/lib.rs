/*!
# Theseus

Theseus is a library which provides utilities for launching minecraft, creating Modrinth mod packs,
and launching Modrinth mod packs
*/
#![warn(unused_import_braces)]
#![deny(unused_must_use)]

#[macro_use]
mod util;

mod api;
mod error;
mod event;
pub mod install;
mod launcher;
mod logger;
mod state;
mod updater;

pub use api::*;
pub use error::*;
pub use event::{
    EventState, LoadingBar, LoadingBarType, emit::emit_loading,
    emit::init_loading,
};
pub use logger::start_logger;
pub use state::State;
pub use updater::cleanup_updater_temp_folders;
pub use util::fetch::DownloadReason;

pub fn launcher_user_agent() -> String {
    const LAUNCHER_BASE_USER_AGENT: &str =
        concat!("modrinth/theseus/", env!("CARGO_PKG_VERSION"),);

    format!(
        "{} ({}; support@modrinth.com)",
        LAUNCHER_BASE_USER_AGENT,
        std::env::consts::OS
    )
}
