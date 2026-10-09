//! Logic for tasks which read from our upstreams, interact with the database,
//! and export artifacts.

mod extract_installers;

pub use extract_installers::*;
