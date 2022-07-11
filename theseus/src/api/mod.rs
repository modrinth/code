//! API for interacting with Theseus
pub mod auth;
pub mod profile;

pub mod data {
    pub use crate::state::{
        DirectoryInfo, Hooks, JavaSettings, MemorySettings, ModLoader,
        ProfileMetadata, Settings, WindowSize,
    };
}

pub mod prelude {
    pub use crate::{
        auth::{authenticate, Credentials},
        data::*,
        profile::{self, Profile},
        State,
    };
}
