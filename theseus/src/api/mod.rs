//! API for interacting with Theseus
pub mod auth;
pub mod pack;
pub mod profile;
pub mod profile_create;

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
        pack,
        profile::{self, Profile},
        profile_create, State,
    };
}
