/*!
# Theseus

Theseus is a library which provides utilities for launching minecraft, creating Modrinth mod packs,
and launching Modrinth mod packs
*/
#![warn(unused_import_braces, missing_debug_implementations)]
#![deny(unused_must_use)]

mod api;
mod config;
mod error;
mod launcher;
mod state;
mod util;

pub use api::*;
pub use error::*;
pub use state::State;
