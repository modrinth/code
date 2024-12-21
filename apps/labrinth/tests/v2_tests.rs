// importing common module.
mod common;

// Not all tests expect exactly the same functionality in v2 and v3.
// For example, though we expect the /GET version to return the corresponding project,
// we may want to do different checks for each.
// (such as checking client_side in v2, but loader fields on v3- which are model-exclusie)

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
