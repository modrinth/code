//! API for interacting with Theseus
pub mod auth;
pub mod handler;
pub mod jre;
pub mod logs;
pub mod metadata;
pub mod pack;
pub mod process;
pub mod profile;
pub mod safety;
pub mod settings;
pub mod tags;

pub mod data {
    pub use crate::state::{
        DirectoryInfo, Hooks, JavaSettings, LinkedData, MemorySettings,
        ModLoader, ModrinthProject, ModrinthTeamMember, ModrinthUser,
        ModrinthVersion, ProfileMetadata, ProjectMetadata, Settings, Theme,
        WindowSize,
    };
}

pub mod prelude {
    pub use crate::{
        auth::{self, Credentials},
        data::*,
        event::CommandPayload,
        jre, metadata, pack, process,
        profile::{self, create, Profile},
        settings,
        state::JavaGlobals,
        state::{ProfilePathId, ProjectPathId},
        util::{
            io::{canonicalize, IOError},
            jre::JavaVersion,
        },
        State,
    };
}
