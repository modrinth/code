//! API for interacting with Theseus
pub mod auth;
pub mod process;
pub mod profile;
pub mod profile_create;
pub mod tags;
pub mod settings;

pub mod data {
    pub use crate::state::{
        DirectoryInfo, Hooks, JavaSettings, MemorySettings, ModLoader,
        ProfileMetadata, Settings, WindowSize,
    };
}

pub mod prelude {
    pub use crate::{
        auth::{self, Credentials},
        data::*,
        process,
        profile::{self, Profile},
        profile_create, settings, State,
    };
}
