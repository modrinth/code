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
mod config;
mod error;
mod launcher;
mod state;
mod model;

pub use api::*;
pub use error::*;
pub use state::State;
