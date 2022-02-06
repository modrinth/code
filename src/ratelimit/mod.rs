use std::future::Future;
use std::marker::Send;
use std::pin::Pin;
use std::time::Duration;

use crate::ratelimit::errors::ARError;
use actix::dev::*;

pub mod errors;
pub mod memory;
/// The code for this module was directly taken from https://github.com/TerminalWitchcraft/actix-ratelimit
/// with some modifications including upgrading it to Actix 4!
pub mod middleware;

/// Represents message that can be handled by a `StoreActor`
pub enum ActorMessage {
    /// Get the remaining count based on the provided identifier
    Get(String),
    /// Set the count of the client identified by `key` to `value` valid for `expiry`
    Set {
        key: String,
        value: usize,
        expiry: Duration,
    },
    /// Change the value of count for the client identified by `key` by `value`
    Update { key: String, value: usize },
    /// Get the expiration time for the client.
    Expire(String),
    /// Remove the client from the store
    Remove(String),
}

impl Message for ActorMessage {
    type Result = ActorResponse;
}

/// Wrapper type for `Pin<Box<dyn Future>>` type
pub type Output<T> = Pin<Box<dyn Future<Output = Result<T, ARError>> + Send>>;

/// Represents data returned in response to `Messages` by a `StoreActor`
pub enum ActorResponse {
    /// Returned in response to [Messages::Get](enum.Messages.html)
    Get(Output<Option<usize>>),
    /// Returned in response to [Messages::Set](enum.Messages.html)
    Set(Output<()>),
    /// Returned in response to [Messages::Update](enum.Messages.html)
    Update(Output<usize>),
    /// Returned in response to [Messages::Expire](enum.Messages.html)
    Expire(Output<Duration>),
    /// Returned in response to [Messages::Remove](enum.Messages.html)
    Remove(Output<usize>),
}

impl<A, M> MessageResponse<A, M> for ActorResponse
where
    A: Actor,
    M: actix::Message<Result = ActorResponse>,
{
    fn handle(self, _: &mut A::Context, tx: Option<OneshotSender<Self>>) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}
