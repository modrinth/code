//! API for interacting with Theseus
pub mod profile;

pub mod data {
    pub use crate::{
        launcher::Credentials,
        state::{
            DirectoryInfo, Hooks, JavaSettings, MemorySettings, ModLoader,
            ProfileMetadata, Settings, WindowSize,
        },
    };
}

pub mod prelude {
    pub use crate::{
        data::*,
        profile::{self, Profile},
        State,
    };
}
