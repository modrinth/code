use thiserror::Error;

pub mod categories;
pub mod charge_item;
pub mod collection_item;
pub mod flow_item;
pub mod friend_item;
pub mod ids;
pub mod image_item;
pub mod legacy_loader_fields;
pub mod loader_fields;
pub mod notification_item;
pub mod oauth_client_authorization_item;
pub mod oauth_client_item;
pub mod oauth_token_item;
pub mod organization_item;
pub mod pat_item;
pub mod payout_item;
pub mod product_item;
pub mod project_item;
pub mod report_item;
pub mod session_item;
pub mod team_item;
pub mod thread_item;
pub mod user_item;
pub mod user_subscription_item;
pub mod version_item;

pub use collection_item::DBCollection;
pub use ids::*;
pub use image_item::DBImage;
pub use oauth_client_item::DBOAuthClient;
pub use organization_item::DBOrganization;
pub use project_item::DBProject;
pub use team_item::DBTeam;
pub use team_item::DBTeamMember;
pub use thread_item::{DBThread, DBThreadMessage};
pub use user_item::DBUser;
pub use version_item::DBVersion;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error while interacting with the database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Error while trying to generate random ID")]
    RandomId,
    #[error("Error while interacting with the cache: {0}")]
    CacheError(#[from] redis::RedisError),
    #[error("Redis Pool Error: {0}")]
    RedisPool(#[from] deadpool_redis::PoolError),
    #[error("Error while serializing with the cache: {0}")]
    SerdeCacheError(#[from] serde_json::Error),
    #[error("Schema error: {0}")]
    SchemaError(String),
    #[error("Timeout when waiting for cache subscriber")]
    CacheTimeout,
}
