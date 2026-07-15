//! API for interacting with Theseus
pub mod cache;
pub mod friends;
pub mod handler;
pub mod instance;
pub mod jre;
pub mod logs;
pub mod metadata;
pub mod minecraft_auth;
pub mod minecraft_skins;
pub mod mr_auth;
pub mod onboarding_checklist;
pub mod pack;
pub mod process;
pub mod server_address;
pub mod settings;
pub mod tags;
pub mod worlds;

pub mod data {
    pub use crate::state::{
        AppliedContentSetPatch, CacheBehaviour, CacheValueType, ContentFile,
        ContentItem, ContentItemOwner, ContentItemProject, ContentItemVersion,
        CreateInstance, Credentials, Dependency, DirectoryInfo, EditInstance,
        Hooks, InstanceInstallCandidate, InstanceInstallTarget,
        InstanceLaunchOverridesPatch, InstanceLink, InstanceMetadata,
        JavaVersion, LinkedModpackInfo, MemorySettings, ModLoader,
        ModrinthCredentials, OnboardingChecklist, Organization, OwnerType,
        ProcessMetadata, Project, ProjectType, ProjectV3, SearchResult,
        SearchResults, SearchResultsV3, Settings, TeamMember, Theme, User,
        UserFriend, Version, WindowSize,
    };
    pub use ariadne::users::UserStatus;
    pub use modrinth_content_management::{
        ContentType, ResolutionPreferences, ResolveContentPlan,
        ResolveContentRequest,
    };
}

pub mod prelude {
    pub use crate::{
        State,
        data::*,
        event::CommandPayload,
        install, instance, jre, metadata, minecraft_auth, mr_auth,
        onboarding_checklist, pack, process, settings,
        state::{ReleaseChannel, db_backup::app_db_backup_dir},
        util::{
            io::{IOError, canonicalize},
            network::{is_network_metered, tcp_listen_any_loopback},
        },
    };
}
