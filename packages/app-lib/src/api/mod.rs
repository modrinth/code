//! API for interacting with Theseus
pub mod cache;
pub mod friends;
pub mod handler;
pub mod jre;
pub mod logs;
pub mod metadata;
pub mod minecraft_auth;
pub mod mr_auth;
pub mod pack;
pub mod process;
pub mod profile;
pub mod settings;
pub mod tags;
pub mod worlds;

pub mod data {
    pub use crate::state::{
        CacheBehaviour, CacheValueType, Credentials, Dependency, DirectoryInfo,
        Hooks, JavaVersion, LinkedData, MemorySettings, ModLoader,
        ModrinthCredentials, Organization, ProcessMetadata, ProfileFile,
        Project, ProjectType, SearchResult, SearchResults, Settings,
        TeamMember, Theme, User, UserFriend, Version, WindowSize,
    };
    pub use ariadne::users::UserStatus;
}

pub mod prelude {
    pub use crate::{
        data::*,
        event::CommandPayload,
        jre, metadata, minecraft_auth, mr_auth, pack, process,
        profile::{self, create, Profile},
        settings,
        util::io::{canonicalize, IOError},
        State,
    };
}
