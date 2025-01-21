use super::message::{ClientToServerMessage, ServerToClientMessage};
use actix_ws::{Closed, Message, Session};
use either::Either;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeserializationError {
    #[error("Failed to deserialize message: {0}")]
    DeserializationFailed(#[from] serde_json::Error),

    #[error("Failed to deserialize binary message: {0}")]
    BinaryDeserializationFailed(#[from] serde_cbor::Error),
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error("Failed to serialize message: {0}")]
    SerializationFailed(#[from] serde_json::Error),

    #[error("Failed to serialize binary message: {0}")]
    BinarySerializationFailed(#[from] serde_cbor::Error),

    #[error("Websocket closed")]
    Closed,
}

impl From<Closed> for SendError {
    fn from(_: Closed) -> Self {
        SendError::Closed
    }
}

macro_rules! message_wire {
    ($message_enum:ty, $binary_pattern:pat) => {
        impl $message_enum {
            pub fn deserialize(
                msg: Message,
            ) -> Result<Either<Self, Message>, DeserializationError> {
                Ok(match msg {
                    Message::Text(text) => {
                        Either::Left(serde_json::from_str(&text)?)
                    }
                    Message::Binary(bytes) => {
                        Either::Left(serde_cbor::from_slice(&bytes)?)
                    }
                    other => Either::Right(other),
                })
            }

            pub async fn send(
                &self,
                session: &mut Session,
            ) -> Result<(), SendError> {
                Ok(match self {
                    $binary_pattern => {
                        session.binary(serde_cbor::to_vec(self)?).await?
                    }
                    _ => session.text(serde_json::to_string(self)?).await?,
                })
            }
        }
    };
}

message_wire!(
    ClientToServerMessage,
    ClientToServerMessage::SocketSend { .. }
);
message_wire!(
    ServerToClientMessage,
    ServerToClientMessage::SocketData { .. }
);
