//! All integration tests compile into this single binary, since linking a separate
//! test binary per file against all of labrinth's dependencies is slow.

#[macro_use]
pub mod common;

mod error;
mod games;
mod limits;
mod loader_fields;
mod moderation_lock;
mod moderation_notes;
mod notifications;
mod oauth;
mod oauth_clients;
mod organizations;
mod pats;
mod project;
mod redis;
mod scopes;
mod search;
mod tags;
mod teams;
mod user;
mod version;

// Not all tests expect exactly the same functionality in v2 and v3.
// For example, though we expect the /GET version to return the corresponding project,
// we may want to do different checks for each.
// (such as checking client_side in v2, but loader fields on v3- which are model-exclusive)

// Such V2 tests are exported here
mod v2 {
    mod error;
    mod notifications;
    mod project;
    mod scopes;
    mod search;
    mod tags;
    mod teams;
    mod version;
}
