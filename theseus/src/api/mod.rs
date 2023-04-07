//! API for interacting with Theseus
pub mod auth;
pub mod jre;
pub mod pack;
pub mod process;
pub mod profile;
pub mod profile_create;
pub mod settings;
pub mod tags;

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
        jre, pack, process,
        profile::{self, Profile},
        profile_create, settings,
        state::JavaGlobals,
        util::jre::JavaVersion,
        State,
    };
}
