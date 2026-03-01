//! Highly experimental and unstable API endpoint models.
//!
//! These are used for testing new API patterns and exploring future endpoints,
//! which may or may not make it into an official release.
//!
//! # Projects and versions
//!
//! Projects and versions work in an ECS-like architecture, where each project
//! is an entity (project ID), and components can be attached to that project to
//! determine the project's type, like a Minecraft mod, data pack, etc. Project
//! components *may* store extra data (like a server listing which stores the
//! server address), but typically, the version will store this data in *version
//! components*.

pub mod base;
pub mod compat;
pub mod component;
pub mod minecraft;
pub mod project;
pub mod version;

pub use project::{
    PROJECT_COMPONENT_RELATIONS, ProjectComponentKind, ProjectEdit,
    ProjectQuery, ProjectSerial,
};
pub use version::{
    VersionComponentKind, VersionCreate, VersionEdit, VersionQuery,
    VersionSerial,
};
