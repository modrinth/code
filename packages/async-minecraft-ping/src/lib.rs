mod protocol;
mod server;
pub use server::{
    connect, ConnectionConfig, ServerError, ServerPlayer, ServerPlayers, ServerVersion,
    StatusConnection, StatusResponse,
};
