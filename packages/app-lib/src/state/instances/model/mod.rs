#![allow(dead_code)]

mod content_entry;
pub use self::content_entry::*;

mod content_set;
pub use self::content_set::*;

mod content_set_remote_ref;
pub use self::content_set_remote_ref::*;

mod content_set_sync_state;
pub use self::content_set_sync_state::*;

mod file;
pub use self::file::*;

mod game_options;
pub use self::game_options::{CanonicalValue, GameOptionKind};
pub(crate) use self::game_options::{
    GameOptionsProjection, ProjectedField, ProjectionOrigin, StoredOption,
    StoredPreference,
};

mod instance;
pub use self::instance::*;

mod install_candidate;
pub use self::install_candidate::*;

mod launch;
pub use self::launch::*;

mod link;
pub use self::link::*;

mod manifest;

mod update_check;
pub use self::update_check::*;

fn unknown_value(kind: &str, value: &str) -> crate::Error {
    crate::ErrorKind::InputError(format!("Unknown {kind} {value}")).into()
}
