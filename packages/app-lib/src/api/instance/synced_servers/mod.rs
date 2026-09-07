mod codec;
mod modpack;
mod operations;
mod storage;
mod types;

const SERVERS_FILE: &str = "servers.dat";

pub(crate) use self::codec::server_data;
pub use self::modpack::{capture_modpack_servers, clear_modpack_servers};
pub(crate) use self::operations::{
    add_user_server, list_server_records, remove_server_by_index,
    update_server_by_index,
};
pub use self::operations::{
    desync_server, list_synced_servers, remove_synced_server,
    update_synced_server,
};
pub(super) use self::operations::{
    detach_servers, ensure_servers, merge_servers_from_instance,
    reconcile_servers, seed_servers,
};
pub(super) use self::storage::canonical_exists;
pub use self::types::{DesyncServerMode, ServerSource, SyncedServer};
