pub mod meilisearch;

pub use self::meilisearch::{
    BatchClient, MeilisearchBackend, MeilisearchReadClient, SearchConfig,
};

pub use super::{
    ResultSearchProject, SearchBackend, SearchResults, UploadSearchProject,
};
