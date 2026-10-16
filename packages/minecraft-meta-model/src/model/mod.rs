//! Models for tables stored in the database.
//!
//! Use this when interacting with a Postgres database which was populated by
//! `meta-generator`.

mod download;
mod fabric;
mod fabriclike;
mod forge;
mod forgelike;
mod minecraft;
mod mojang;
mod neoforge;
mod profile;
mod quilt;
pub use download::*;
pub use fabric::*;
pub use fabriclike::*;
pub use forge::*;
pub use forgelike::*;
pub use minecraft::*;
pub use mojang::*;
pub use neoforge::*;
pub use profile::*;
pub use quilt::*;
